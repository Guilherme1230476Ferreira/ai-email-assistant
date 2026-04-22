use std::sync::Arc;

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use bcrypt::verify;

use crate::{
    app_error::AppError,
    infrastructure::config::Config,
    middleware::auth::create_jwt,
    models::dto::{LoginRequest, LoginResponse},
    repositories::user_repo::UserRepository,
    state::AppState,
};

#[utoipa::path(
    post,
    path = "/api/auth/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = LoginResponse),
        (status = 401, description = "Invalid credentials"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Auth"
)]
#[axum::debug_handler(state = AppState)]
pub async fn login_handler(
    State(user_repo): State<Arc<UserRepository>>,
    State(config): State<Arc<Config>>,
    Json(payload): Json<LoginRequest>,
) -> Result<impl IntoResponse, AppError> {
    let user = user_repo.get_user_by_email(&payload.email).await?;

    if !verify(&payload.password, &user.password_hash).unwrap_or(false) {
        return Err(AppError::new(
            StatusCode::UNAUTHORIZED,
            "Invalid credentials".to_string(),
        ));
    }

    let token = create_jwt(user.id, &config)?;

    Ok((StatusCode::OK, Json(LoginResponse { token })))
}
