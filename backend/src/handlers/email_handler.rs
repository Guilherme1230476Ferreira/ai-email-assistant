use std::sync::Arc;

use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use futures_util::StreamExt;

use crate::{
    app_error::AppError,
    infrastructure::config::Config,
    middleware::auth::AuthUser,
    models::{
        domain::Email,
        dto::{GenerateEmailRequest, PaginatedResponse, PaginationParams},
    },
    repositories::{email_repo::EmailRepository, settings_repo::SettingsRepository},
    services::llm_service::LlmService,
    state::AppState,
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
    let api_key = settings_repo
        .get_decrypted_api_key()
        .await?
        .ok_or_else(|| AppError::new(StatusCode::BAD_REQUEST, "LLM API Key is not configured."))?;

    // 2. Fetch Embeddings and Context for RAG
    let mut context_str = String::new();
    let embedding_result = llm_service
        .generate_embedding(
            &request.prompt,
            &config.embedding_api_url,
            &config.embedding_model,
            &config.embedding_api_key,
        )
        .await;

    let mut prompt_embedding = None;
    match embedding_result {
        Ok(vec) => {
            if let Ok(similar_emails) = email_repo
                .find_similar_emails(auth_user.0.id, &vec, 3)
                .await
            {
                for past_email in similar_emails {
                    if let Some(resp) = past_email.generated_response {
                        context_str.push_str(&format!(
                            "--- Past Received Email: {}\n--- How You Replied: {}\n\n",
                            past_email.original_content, resp
                        ));
                    }
                }
            }
            prompt_embedding = Some(vec);
        }
        Err(e) => {
            eprintln!(
                "Warning: Failed to fetch embeddings for RAG Context. Proceeding without context. Error: {:?}",
                e
            );
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
        .generate_reply(
            &email_to_generate,
            &context_str,
            &settings.llm_base_url,
            &settings.llm_model,
            &api_key,
        )
        .await
        .map_err(|e| {
            eprintln!("LLM service error: {:?}", e);
            AppError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to generate email response".to_string(),
            )
        })?;

    // 4. Fetch Embedding for the LLM's Generated Response
    let response_embedding_result = llm_service
        .generate_embedding(
            &generated_response,
            &config.embedding_api_url,
            &config.embedding_model,
            &config.embedding_api_key,
        )
        .await;

    let response_embedding = match response_embedding_result {
        Ok(vec) => Some(vec),
        Err(e) => {
            eprintln!(
                "Warning: Failed to fetch embedding for generated response: {:?}",
                e
            );
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
            AppError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to save generated email".to_string(),
            )
        })?;

    Ok((StatusCode::OK, Json(email)))
}

// ─── SSE Streaming Endpoint ──────────────────────────────────────────────────

#[utoipa::path(
    post,
    path = "/api/emails/generate/stream",
    request_body = GenerateEmailRequest,
    responses(
        (status = 200, description = "SSE stream of generated email tokens"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal Server Error")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Email"
)]
#[axum::debug_handler(state = AppState)]
pub async fn generate_email_stream_handler(
    State(settings_repo): State<Arc<SettingsRepository>>,
    State(llm_service): State<Arc<dyn LlmService + Send + Sync>>,
    State(email_repo): State<Arc<EmailRepository>>,
    State(config): State<Arc<Config>>,
    auth_user: AuthUser,
    Json(request): Json<GenerateEmailRequest>,
) -> Result<
    axum::response::sse::Sse<
        impl futures_util::Stream<Item = Result<axum::response::sse::Event, std::convert::Infallible>>,
    >,
    AppError,
> {
    // 1. Settings + API key
    let settings = settings_repo.get_settings().await?;
    let api_key = settings_repo
        .get_decrypted_api_key()
        .await?
        .ok_or_else(|| AppError::new(StatusCode::BAD_REQUEST, "LLM API Key is not configured."))?;

    // 2. RAG context
    let mut context_str = String::new();
    let embedding_result = llm_service
        .generate_embedding(
            &request.prompt,
            &config.embedding_api_url,
            &config.embedding_model,
            &config.embedding_api_key,
        )
        .await;

    let mut prompt_embedding = None;
    match embedding_result {
        Ok(vec) => {
            if let Ok(similar_emails) = email_repo
                .find_similar_emails(auth_user.0.id, &vec, 3)
                .await
            {
                for past_email in similar_emails {
                    if let Some(resp) = past_email.generated_response {
                        context_str.push_str(&format!(
                            "--- Past Received Email: {}\n--- How You Replied: {}\n\n",
                            past_email.original_content, resp
                        ));
                    }
                }
            }
            prompt_embedding = Some(vec);
        }
        Err(e) => {
            eprintln!(
                "Warning: Embedding failed, proceeding without context: {:?}",
                e
            );
        }
    }

    // 3. Build the email entity for streaming
    let email_to_generate = Email {
        id: uuid::Uuid::new_v4(),
        user_id: auth_user.0.id,
        original_content: request.prompt.clone(),
        generated_response: Some("".to_string()),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    // 4. Get the token stream from the LLM
    let token_stream = llm_service
        .generate_reply_stream(
            &email_to_generate,
            &context_str,
            &settings.llm_base_url,
            &settings.llm_model,
            &api_key,
        )
        .await
        .map_err(|e| {
            eprintln!("LLM stream error: {:?}", e);
            AppError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to start LLM stream",
            )
        })?;

    // 5. Build the SSE stream: yield tokens, then save to DB on completion
    let user_id = auth_user.0.id;
    let prompt = request.prompt.clone();
    let emb_api_url = config.embedding_api_url.clone();
    let emb_model = config.embedding_model.clone();
    let emb_key = config.embedding_api_key.clone();

    let sse_stream = async_stream::stream! {
        let mut full_response = String::new();

        futures_util::pin_mut!(token_stream);
        while let Some(chunk_result) = token_stream.next().await {
            match chunk_result {
                Ok(token) => {
                    full_response.push_str(&token);
                    let event = axum::response::sse::Event::default()
                        .event("token")
                        .data(token);
                    yield Ok::<_, std::convert::Infallible>(event);
                }
                Err(e) => {
                    let event = axum::response::sse::Event::default()
                        .event("error")
                        .data(format!("{}", e.message()));
                    yield Ok(event);
                    return;
                }
            }
        }

        // Stream done — save the email to DB
        // Generate response embedding
        let resp_embedding = match llm_service.generate_embedding(&full_response, &emb_api_url, &emb_model, &emb_key).await {
            Ok(v) => Some(v),
            Err(_) => None,
        };

        match email_repo.create_email(user_id, &prompt, &full_response, prompt_embedding, resp_embedding).await {
            Ok(email) => {
                let done_data = serde_json::json!({ "id": email.id.to_string() }).to_string();
                let event = axum::response::sse::Event::default()
                    .event("done")
                    .data(done_data);
                yield Ok(event);
            }
            Err(e) => {
                let event = axum::response::sse::Event::default()
                    .event("error")
                    .data(format!("Failed to save email: {:?}", e));
                yield Ok(event);
            }
        }
    };

    Ok(axum::response::sse::Sse::new(sse_stream))
}

// ─── Telemetry ───────────────────────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/api/telemetry",
    responses(
        (status = 200, description = "RAG execution telemetry", body = TelemetryData),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal Server Error")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Email"
)]
#[axum::debug_handler(state = AppState)]
pub async fn get_telemetry_handler(
    State(email_repo): State<Arc<EmailRepository>>,
    auth_user: AuthUser,
) -> Result<impl IntoResponse, AppError> {
    let telemetry = email_repo
        .get_telemetry_for_user(auth_user.0.id)
        .await
        .map_err(|e| {
            eprintln!("Telemetry error: {:?}", e);
            AppError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to compute telemetry".to_string(),
            )
        })?;

    Ok((StatusCode::OK, Json(telemetry)))
}

// ─── Paginated Email List ────────────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/api/emails",
    params(PaginationParams),
    responses(
        (status = 200, description = "Paginated list of user emails"),
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
    Query(pagination): Query<PaginationParams>,
) -> Result<impl IntoResponse, AppError> {
    let (offset, limit) = pagination.offset_limit();

    let (emails, total) = tokio::try_join!(
        async {
            email_repo
                .get_emails_by_user_paginated(auth_user.0.id, offset, limit)
                .await
                .map_err(|e| {
                    eprintln!("Email repo error: {:?}", e);
                    AppError::new(StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch emails")
                })
        },
        async {
            email_repo
                .get_email_count_by_user(auth_user.0.id)
                .await
                .map_err(|e| {
                    eprintln!("Email count error: {:?}", e);
                    AppError::new(StatusCode::INTERNAL_SERVER_ERROR, "Failed to count emails")
                })
        }
    )?;

    let response = PaginatedResponse {
        items: emails,
        total,
        page: pagination.page_num(),
        limit: pagination.per_page(),
    };

    Ok((StatusCode::OK, Json(response)))
}
