use std::sync::Arc;

use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Redirect},
};
use bcrypt::verify;
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, RedirectUrl, Scope,
    TokenResponse, TokenUrl, basic::BasicClient, reqwest::async_http_client,
};
use serde::Deserialize;

use crate::{
    app_error::AppError,
    infrastructure::config::Config,
    middleware::auth::create_jwt,
    models::dto::{CreateUserRequest, LoginRequest, LoginResponse},
    repositories::user_repo::UserRepository,
    state::AppState,
};

#[utoipa::path(
    post,
    path = "/api/auth/register",
    request_body = CreateUserRequest,
    responses(
        (status = 201, description = "User registered successfully"),
        (status = 400, description = "Invalid input"),
        (status = 409, description = "User already exists"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Auth"
)]
#[axum::debug_handler(state = AppState)]
pub async fn register_handler(
    State(user_repo): State<Arc<UserRepository>>,
    Json(payload): Json<CreateUserRequest>,
) -> Result<impl IntoResponse, AppError> {
    let new_user = user_repo.create_user(&payload).await?;
    Ok((StatusCode::CREATED, Json(new_user)))
}

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

#[derive(Deserialize)]
pub struct AuthRequest {
    code: String,
    state: String,
}

#[derive(Deserialize)]
pub struct GoogleAuthQuery {
    intent: Option<String>,
}

#[derive(Deserialize)]
struct GoogleUser {
    email: String,
}

fn get_google_client(config: &Config) -> Result<BasicClient, AppError> {
    let client_id = config.google_client_id.clone().ok_or_else(|| {
        AppError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Google Client ID not configured".to_string(),
        )
    })?;
    let client_secret = config.google_client_secret.clone().ok_or_else(|| {
        AppError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Google Client Secret not configured".to_string(),
        )
    })?;

    let auth_url = AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".to_string())
        .map_err(|_| {
            AppError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Invalid auth URL".to_string(),
            )
        })?;

    let token_url =
        TokenUrl::new("https://oauth2.googleapis.com/token".to_string()).map_err(|_| {
            AppError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Invalid token URL".to_string(),
            )
        })?;

    let redirect_url = format!("{}/api/auth/google/callback", config.public_url);

    Ok(BasicClient::new(
        ClientId::new(client_id),
        Some(ClientSecret::new(client_secret)),
        auth_url,
        Some(token_url),
    )
    .set_redirect_uri(
        RedirectUrl::new(redirect_url).map_err(
            |_| {
                AppError::new(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Invalid redirect URL".to_string(),
                )
            },
        )?,
    ))
}

#[utoipa::path(
    get,
    path = "/api/auth/google",
    responses(
        (status = 303, description = "Redirects to Google OAuth UI")
    ),
    tag = "Auth"
)]
#[axum::debug_handler(state = AppState)]
pub async fn google_auth_handler(
    Query(query): Query<GoogleAuthQuery>,
    State(config): State<Arc<Config>>,
) -> Result<impl IntoResponse, AppError> {
    let client = get_google_client(&config)?;

    let target_intent = query.intent.unwrap_or_else(|| "login".to_string());

    let (auth_url, _csrf_token) = client
        .authorize_url(|| CsrfToken::new(target_intent))
        .add_scope(Scope::new("email".to_string()))
        .add_scope(Scope::new("profile".to_string()))
        .url();

    Ok(Redirect::to(auth_url.as_str()))
}

#[utoipa::path(
    get,
    path = "/api/auth/google/callback",
    responses(
        (status = 303, description = "Redirects to frontend with auth token cookie"),
        (status = 400, description = "Invalid request")
    ),
    tag = "Auth"
)]
#[axum::debug_handler(state = AppState)]
pub async fn google_callback_handler(
    Query(query): Query<AuthRequest>,
    State(user_repo): State<Arc<UserRepository>>,
    State(config): State<Arc<Config>>,
) -> Result<impl IntoResponse, AppError> {
    let client = get_google_client(&config)?;

    // Exchange the code for a token.
    let token_result = client
        .exchange_code(AuthorizationCode::new(query.code))
        .request_async(async_http_client)
        .await
        .map_err(|_| {
            AppError::new(
                StatusCode::UNAUTHORIZED,
                "Failed to validate token with Google".to_string(),
            )
        })?;

    let access_token = token_result.access_token().secret();

    // Use the token to fetch the user's profile info from Google.
    let client = reqwest::Client::new();
    let user_info_response = client
        .get("https://www.googleapis.com/oauth2/v2/userinfo")
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|_| {
            AppError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to request user info from Google".to_string(),
            )
        })?;

    let user_info: GoogleUser = user_info_response.json().await.map_err(|_| {
        AppError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to parse user info".to_string(),
        )
    })?;

    let intent = query.state.clone();

    // Check if user exists
    let user_exists = user_repo.get_user_by_email(&user_info.email).await;

    if intent == "login" && user_exists.is_err() {
        return Ok((
            StatusCode::SEE_OTHER,
            [
                (
                    axum::http::header::SET_COOKIE,
                    "token=; Path=/; Max-Age=0".to_string(),
                ),
                (
                    axum::http::header::LOCATION,
                    format!("{}/login?error=not_registered", config.public_url),
                ),
            ],
        ));
    }

    let user = match user_exists {
        Ok(user) => {
            if intent == "signup" {
                return Ok((
                    StatusCode::SEE_OTHER,
                    [
                        (
                            axum::http::header::SET_COOKIE,
                            "token=; Path=/; Max-Age=0".to_string(),
                        ),
                        (
                            axum::http::header::LOCATION,
                            format!("{}/signup?error=already_registered", config.public_url),
                        ),
                    ],
                ));
            }
            user
        }
        Err(_) => {
            let random_password = uuid::Uuid::new_v4().to_string(); // Inaccessible via normal login
            let payload = CreateUserRequest {
                email: user_info.email.clone(),
                password: random_password,
            };
            user_repo.create_user(&payload).await?
        }
    };

    let token = create_jwt(user.id, &config)?;

    // We set the token in a cookie and redirect back to the home page securely
    let cookie_str = format!(
        "token={}; Path=/; Max-Age={}; SameSite=Lax; HttpOnly",
        token, 86400
    );

    let redirect_url = if intent == "signup" {
        format!("{}/login?success=registered", config.public_url)
    } else {
        format!("{}/", config.public_url)
    };

    Ok((
        StatusCode::SEE_OTHER,
        [
            (axum::http::header::SET_COOKIE, cookie_str),
            (axum::http::header::LOCATION, redirect_url),
        ],
    ))
}

#[utoipa::path(
    get,
    path = "/api/auth/me",
    responses(
        (status = 200, description = "Returns the currently authenticated user", body = UserResponse),
        (status = 401, description = "Unauthorized")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Auth"
)]
#[axum::debug_handler(state = AppState)]
pub async fn me_handler(
    auth_user: crate::middleware::auth::AuthUser,
) -> Result<impl axum::response::IntoResponse, AppError> {
    let response = crate::models::dto::UserResponse::from(auth_user.0);
    Ok((axum::http::StatusCode::OK, axum::Json(response)))
}
