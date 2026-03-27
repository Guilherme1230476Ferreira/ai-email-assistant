/// JSON request/response structs — separate from database entities.
/// Will be fleshed out in Sprint 3.
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

#[derive(Debug, Deserialize, ToSchema)]
pub struct GenerateReplyRequest {
    pub email_id: String,
    pub email_body: String,
    pub context: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct GenerateReplyResponse {
    pub email_id: String,
    pub reply: String,
}

/// Response for the get_api_key endpoint.
#[derive(Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct ApiKeyResponse {
    /// Indicates whether an API key is currently configured.
    pub has_api_key: bool,
}

/// Request body for updating the global LLM API key.
#[derive(Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct UpdateApiKeyRequest {
    /// The new LLM API key. If `None` or an empty string, the existing key will be removed.
    pub api_key: Option<String>,
}

/// Request body for creating a new user.
#[derive(Debug, serde::Deserialize, ToSchema)]
pub struct CreateUserRequest {
    #[schema(example = "test.user@example.com")]
    pub email: String,
    #[schema(example = "Str0ngP@ssw0rd!")]
    pub password: String,
}

/// Request body for updating a user's role.
#[derive(Debug, serde::Deserialize, ToSchema)]
pub struct UpdateUserRoleRequest {
    pub role_id: Uuid,
}

/// Request body for creating a new role.
#[derive(Debug, serde::Deserialize, ToSchema)]
pub struct CreateRoleRequest {
    #[schema(example = "editor")]
    pub name: String,
}

/// Response for role-related endpoints.
#[derive(Debug, serde::Serialize, ToSchema)]
pub struct RoleResponse {
    pub id: Uuid,
    pub name: String,
}

/// Response for user-related endpoints.
#[derive(Debug, serde::Serialize, ToSchema)]
pub struct UserResponse {
    pub id: Uuid,
    pub email: String,
    pub role_id: Uuid,
}

/// Generic error response.
#[derive(Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct ErrorResponse {
    pub error: String,
}

#[derive(Debug, Deserialize, ToSchema, IntoParams)]
pub struct HistoryQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Deserialize, ToSchema, IntoParams)]
pub struct LogsQuery {
    pub action: Option<String>,
    pub from_date: Option<String>,
    pub to_date: Option<String>,
}
