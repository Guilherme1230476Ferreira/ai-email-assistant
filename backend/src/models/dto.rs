use chrono::{DateTime, Utc};
/// JSON request/response structs — separate from database entities.
/// Will be fleshed out in Sprint 3.
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

#[derive(Debug, Serialize, ToSchema)]
pub struct TelemetryData {
    pub context_retrieval_rate: f64,
    pub avg_similarity_score: f64,
    pub tokens_saved: i64,
    pub knowledge_matches: i64,
}

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

/// Response for user-related endpoints (never includes password_hash).
#[derive(Debug, serde::Serialize, ToSchema)]
pub struct UserResponse {
    pub id: Uuid,
    pub email: String,
    pub role_id: Uuid,
    pub created_at: DateTime<Utc>,
}

impl From<super::domain::User> for UserResponse {
    fn from(u: super::domain::User) -> Self {
        Self {
            id: u.id,
            email: u.email,
            role_id: u.role_id,
            created_at: u.created_at,
        }
    }
}

/// Request body for the login endpoint.
#[derive(Debug, serde::Deserialize, ToSchema)]
pub struct LoginRequest {
    #[schema(example = "test.user@example.com")]
    pub email: String,
    #[schema(example = "Str0ngP@ssw0rd!")]
    pub password: String,
}

/// Response for a successful login.
#[derive(Debug, serde::Serialize, ToSchema)]
pub struct LoginResponse {
    pub token: String,
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

use super::domain::{Email, Role, User};

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct GenerateEmailRequest {
    #[schema(example = "Write a follow-up email to a client who missed a meeting.")]
    pub prompt: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct GenerateEmailResponse {
    pub email: Email,
}

/// Query parameters for paginated list endpoints.
#[derive(Debug, Deserialize, IntoParams)]
pub struct PaginationParams {
    /// Page number (1-indexed). Defaults to 1.
    pub page: Option<i64>,
    /// Items per page. Defaults to 20, max 100.
    pub limit: Option<i64>,
}

impl PaginationParams {
    pub fn offset_limit(&self) -> (i64, i64) {
        let limit = self.limit.unwrap_or(20).min(100).max(1);
        let page = self.page.unwrap_or(1).max(1);
        let offset = (page - 1) * limit;
        (offset, limit)
    }

    pub fn page_num(&self) -> i64 {
        self.page.unwrap_or(1).max(1)
    }

    pub fn per_page(&self) -> i64 {
        self.limit.unwrap_or(20).min(100).max(1)
    }
}

/// Generic paginated response wrapper.
#[derive(Debug, Serialize, ToSchema)]
pub struct PaginatedResponse<T: Serialize> {
    pub items: Vec<T>,
    pub total: i64,
    pub page: i64,
    pub limit: i64,
}
