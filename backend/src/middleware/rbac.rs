use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{StatusCode, request::Parts},
};

use crate::{
    app_error::AppError, middleware::auth::AuthUser, models::domain::User, state::AppState,
};

/// Axum extractor that validates the authenticated user has the "admin" role.
/// Use this instead of `AuthUser` on any handler that requires admin privileges.
///
/// Returns 403 Forbidden if the user's role is not "admin".
pub struct AdminUser(pub User);

#[async_trait]
impl FromRequestParts<AppState> for AdminUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        // First, authenticate the user (reuses existing JWT validation)
        let auth_user = AuthUser::from_request_parts(parts, state).await?;
        let user = auth_user.0;

        // Fetch the role to check name
        let role = state
            .role_repo
            .get_role_by_id(user.role_id)
            .await?
            .ok_or_else(|| {
                AppError::new(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "User role not found in database",
                )
            })?;

        if role.name != "admin" {
            return Err(AppError::new(
                StatusCode::FORBIDDEN,
                "Admin privileges required",
            ));
        }

        Ok(AdminUser(user))
    }
}
