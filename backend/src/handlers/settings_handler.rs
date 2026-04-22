use std::sync::Arc;

use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use reqwest::Client;
use serde_json::json;

use crate::{
    app_error::AppError,
    models::settings_dto::{SettingsResponse, UpdateSettingsRequest, VerifySettingsRequest},
    repositories::settings_repo::SettingsRepository,
    state::AppState,
};

#[utoipa::path(
    get,
    path = "/api/admin/settings",
    responses(
        (status = 200, description = "Settings retrieved successfully", body = SettingsResponse),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Admin Settings"
)]
#[axum::debug_handler(state = AppState)]
pub async fn get_settings_handler(
    State(settings_repo): State<Arc<SettingsRepository>>,
) -> Result<impl IntoResponse, AppError> {
    let settings = settings_repo.get_settings().await?;

    let response = SettingsResponse {
        id: settings.id,
        llm_base_url: settings.llm_base_url,
        llm_model: settings.llm_model,
        has_api_key: settings.llm_api_key_encrypted.is_some(),
        masked_api_key: if settings.llm_api_key_encrypted.is_some() {
            Some("sk-...****".to_string())
        } else {
            None
        },
    };

    Ok((StatusCode::OK, Json(response)))
}

#[utoipa::path(
    put,
    path = "/api/admin/settings",
    request_body = UpdateSettingsRequest,
    responses(
        (status = 200, description = "Settings updated successfully", body = SettingsResponse),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Admin Settings"
)]
#[axum::debug_handler(state = AppState)]
pub async fn update_settings_handler(
    State(settings_repo): State<Arc<SettingsRepository>>,
    Json(request): Json<UpdateSettingsRequest>,
) -> Result<impl IntoResponse, AppError> {
    let settings = settings_repo
        .update_settings(
            &request.llm_base_url,
            &request.llm_model,
            request.llm_api_key.as_deref(),
        )
        .await?;

    let response = SettingsResponse {
        id: settings.id,
        llm_base_url: settings.llm_base_url,
        llm_model: settings.llm_model,
        has_api_key: settings.llm_api_key_encrypted.is_some(),
        masked_api_key: if settings.llm_api_key_encrypted.is_some() {
            Some("sk-...****".to_string())
        } else {
            None
        },
    };

    Ok((StatusCode::OK, Json(response)))
}

#[utoipa::path(
    post,
    path = "/api/admin/settings/verify",
    request_body = VerifySettingsRequest,
    responses(
        (status = 200, description = "API Key is valid and working"),
        (status = 400, description = "Verification failed or invalid models"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Admin Settings"
)]
#[axum::debug_handler(state = AppState)]
pub async fn verify_settings_handler(
    State(settings_repo): State<Arc<SettingsRepository>>,
    Json(request): Json<VerifySettingsRequest>,
) -> Result<impl IntoResponse, AppError> {
    
    // Resolve what to test
    let settings = settings_repo.get_settings().await?;
    let base_url = request.llm_base_url.unwrap_or(settings.llm_base_url);
    let model = request.llm_model.unwrap_or(settings.llm_model);
    
    let mut api_key = request.llm_api_key;
    if api_key.is_none() || api_key.as_deref() == Some("sk-...****") {
        let dec_key = settings_repo.get_decrypted_api_key().await?;
        if dec_key.is_none() {
            return Err(AppError::new(StatusCode::BAD_REQUEST, "No API key to verify"));
        }
        api_key = dec_key;
    }

    let url = format!("{}/chat/completions", base_url.trim_end_matches('/'));
    let client = Client::new();

    let payload = json!({
        "model": model,
        "messages": [
            {
                "role": "user",
                "content": "Respond with 'OK' if you receive this."
            }
        ],
        "max_tokens": 10
    });

    let res = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", api_key.unwrap_or_default()))
        .json(&payload)
        .send()
        .await;

    match res {
        Ok(response) => {
            if response.status().is_success() {
                Ok((StatusCode::OK, Json(json!({"status": "Success", "message": "API key and connection validated successfully"}))))
            } else {
                let status = response.status();
                let body = response.text().await.unwrap_or_default();
                Err(AppError::new(
                    StatusCode::BAD_REQUEST,
                    format!("API verification failed with status: {}. Body: {}", status, body),
                ))
            }
        }
        Err(e) => Err(AppError::new(
            StatusCode::BAD_REQUEST,
            format!("Failed to reach LLM API: {}", e),
        )),
    }
}
