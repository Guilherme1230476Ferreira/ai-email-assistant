use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{StatusCode, request::Parts},
};
use axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Bearer},
};
use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{app_error::AppError, models::domain::User, state::AppState};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid, // Subject (user_id)
    pub exp: i64,  // Expiration time
    pub iat: i64,  // Issued at
}

/// Creates a new JWT for a given user ID.
pub fn create_jwt(
    user_id: Uuid,
    config: &crate::infrastructure::config::Config,
) -> Result<String, AppError> {
    let now = Utc::now();
    let expires_at = now + Duration::hours(config.jwt_expiration_hours);

    let claims = Claims {
        sub: user_id,
        exp: expires_at.timestamp(),
        iat: now.timestamp(),
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config.jwt_secret.as_ref()),
    )
    .map_err(|_| AppError::new(StatusCode::INTERNAL_SERVER_ERROR, "Failed to create token"))
}

/// Axum extractor that validates a JWT and provides the authenticated User.
pub struct AuthUser(pub User);

#[async_trait]
impl FromRequestParts<AppState> for AuthUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        // Extract the token from the authorization header
        let TypedHeader(Authorization(bearer)) =
            TypedHeader::<Authorization<Bearer>>::from_request_parts(parts, state)
                .await
                .map_err(|_| {
                    AppError::new(
                        StatusCode::UNAUTHORIZED,
                        "Missing or invalid token".to_string(),
                    )
                })?;

        // Decode the user claims
        let token_data = decode::<Claims>(
            bearer.token(),
            &DecodingKey::from_secret(state.config.jwt_secret.as_ref()),
            &Validation::default(),
        )
        .map_err(|_| AppError::new(StatusCode::UNAUTHORIZED, "Invalid token"))?;

        // Fetch the user from the database
        let user = state
            .user_repo
            .get_user_by_id(token_data.claims.sub)
            .await?
            .ok_or_else(|| AppError::new(StatusCode::UNAUTHORIZED, "User not found"))?;

        Ok(AuthUser(user))
    }
}

/// Rejection type for the AuthUser extractor.
#[derive(Debug)]
pub enum AuthError {
    InvalidToken,
    InternalError,
}

impl From<AuthError> for AppError {
    fn from(err: AuthError) -> Self {
        let (status, message) = match err {
            AuthError::InvalidToken => (StatusCode::UNAUTHORIZED, "Invalid or missing token"),
            AuthError::InternalError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "An internal error occurred",
            ),
        };
        AppError::new(status, message)
    }
}
