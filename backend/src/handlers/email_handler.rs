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
    repositories::email_repo::EmailRepository,
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
    State(llm_service): State<Arc<dyn LlmService + Send + Sync>>,
    State(email_repo): State<Arc<EmailRepository>>,
    auth_user: AuthUser,
    Json(request): Json<GenerateEmailRequest>,
) -> Result<impl IntoResponse, AppError> {
    let email_to_generate = Email {
        id: uuid::Uuid::new_v4(),
        user_id: auth_user.0.id,
        original_content: request.prompt.clone(),
        generated_response: Some("".to_string()),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let generated_response = llm_service
        .generate_reply(&email_to_generate)
        .await
        .map_err(|e| {
            eprintln!("LLM service error: {:?}", e);
            AppError::new(StatusCode::INTERNAL_SERVER_ERROR, "Failed to generate email response".to_string())
        })?;

    let email = email_repo
        .create_email(
            auth_user.0.id,
            &request.prompt,
            &generated_response,
        )
        .await
        .map_err(|e| {
            eprintln!("Email repo error: {:?}", e);
            AppError::new(StatusCode::INTERNAL_SERVER_ERROR, "Failed to save generated email".to_string())
        })?;

    Ok((StatusCode::OK, Json(email)))
}