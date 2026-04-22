use std::sync::Arc;

use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};

use crate::{
    app_error::AppError,
    middleware::auth::AuthUser,
    models::{
        domain::Email,
        dto::GenerateEmailRequest,
    },
    repositories::{email_repo::EmailRepository, settings_repo::SettingsRepository},
    services::llm_service::LlmService,
    state::AppState,
    infrastructure::config::Config,
};

#[utoipa::path(
    post,
    path = "/api/emails/generate",
    request_body = GenerateEmailRequest,
    responses(
        (status = 200, description = "Email generated successfully", body = Email),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal Server Error")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Email"
)]
#[axum::debug_handler(state = AppState)]
pub async fn generate_email_handler(
    State(settings_repo): State<Arc<SettingsRepository>>,
    State(llm_service): State<Arc<dyn LlmService + Send + Sync>>,
    State(email_repo): State<Arc<EmailRepository>>,
    State(config): State<Arc<Config>>,
    auth_user: AuthUser,
    Json(request): Json<GenerateEmailRequest>,
) -> Result<impl IntoResponse, AppError> {

    // 1. Chat/Generation Configurations from DB
    let settings = settings_repo.get_settings().await?;
    let api_key = settings_repo.get_decrypted_api_key().await?
        .ok_or_else(|| AppError::new(StatusCode::BAD_REQUEST, "LLM API Key is not configured."))?;

    // 2. Fetch Embeddings and Context for RAG (Uses .env config to separate permanent Emdedding LLM provider from Chat LLM Provider)
    let mut context_str = String::new();
    let embedding_result = llm_service
        .generate_embedding(&request.prompt, &config.embedding_api_url, &config.embedding_model, &config.embedding_api_key)
        .await;

    let mut prompt_embedding = None;
    match embedding_result {
        Ok(vec) => {
            // Retrieve up to 3 similar past emails mapped to the user
            if let Ok(similar_emails) = email_repo.find_similar_emails(auth_user.0.id, &vec, 3).await {
                for past_email in similar_emails {
                    if let Some(resp) = past_email.generated_response {
                        context_str.push_str(&format!(
                            "--- Past User Prompt: {}\n--- Past User Approved Response: {}\n\n",
                            past_email.original_content, resp
                        ));
                    }
                }
            }
            prompt_embedding = Some(vec);
        }
        Err(e) => {
            // Non-fatal error; if embeddings are unsupported by their chosen model Endpoint (e.g. Groq wrapper issue),
            // we will gracefully fail the RAG context injection and just generate normally.
            eprintln!("Warning: Failed to fetch embeddings for RAG Context. Proceeding without context. Error: {:?}", e);
        }
    }

    // 3. Create the dummy Email entity
    let email_to_generate = Email {
        id: uuid::Uuid::new_v4(),
        user_id: auth_user.0.id,
        original_content: request.prompt.clone(),
        generated_response: Some("".to_string()),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let generated_response = llm_service
        .generate_reply(&email_to_generate, &context_str, &settings.llm_base_url, &settings.llm_model, &api_key)
        .await
        .map_err(|e| {
            eprintln!("LLM service error: {:?}", e);
            AppError::new(StatusCode::INTERNAL_SERVER_ERROR, "Failed to generate email response".to_string())
        })?;

    // 4. Fetch Embedding for the LLM's Generated Response
    let response_embedding_result = llm_service
        .generate_embedding(&generated_response, &config.embedding_api_url, &config.embedding_model, &config.embedding_api_key)
        .await;

    let response_embedding = match response_embedding_result {
        Ok(vec) => Some(vec),
        Err(e) => {
            eprintln!("Warning: Failed to fetch embedding for generated response: {:?}", e);
            None
        }
    };

    let email = email_repo
        .create_email(
            auth_user.0.id,
            &request.prompt,
            &generated_response,
            prompt_embedding,
            response_embedding,
        )
        .await
        .map_err(|e| {
            eprintln!("Email repo error: {:?}", e);
            AppError::new(StatusCode::INTERNAL_SERVER_ERROR, "Failed to save generated email".to_string())
        })?;

    Ok((StatusCode::OK, Json(email)))
}

#[utoipa::path(
    get,
    path = "/api/emails",
    responses(
        (status = 200, description = "List of user emails", body = [Email]),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal Server Error")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Email"
)]
#[axum::debug_handler(state = AppState)]
pub async fn get_emails_handler(
    State(email_repo): State<Arc<EmailRepository>>,
    auth_user: AuthUser,
) -> Result<impl IntoResponse, AppError> {
    let emails = email_repo
        .get_emails_by_user(auth_user.0.id)
        .await
        .map_err(|e| {
            eprintln!("Email repo error: {:?}", e);
            AppError::new(StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch emails".to_string())
        })?;

    Ok((StatusCode::OK, Json(emails)))
}