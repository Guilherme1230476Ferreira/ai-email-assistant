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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::domain::User;
    use chrono::Utc;
    use uuid::Uuid;

    #[test]
    fn test_user_response_from_user_strips_password() {
        let user_id = Uuid::new_v4();
        let role_id = Uuid::new_v4();
        let now = Utc::now();

        let user = User {
            id: user_id,
            email: "test@example.com".to_string(),
            role_id,
            password_hash: "hashed_secret_password_that_must_never_leak".to_string(),
            created_at: now,
        };

        let response = UserResponse::from(user);

        assert_eq!(response.id, user_id);
        assert_eq!(response.email, "test@example.com");
        assert_eq!(response.role_id, role_id);
        assert_eq!(response.created_at, now);

        let serialized = serde_json::to_value(response).unwrap();
        assert!(
            serialized.get("password_hash").is_none(),
            "password_hash must be absent from serialized JSON"
        );
    }

    #[test]
    fn test_pagination_params_defaults() {
        let empty = PaginationParams {
            page: None,
            limit: None,
        };
        assert_eq!(empty.page_num(), 1);
        assert_eq!(empty.per_page(), 20);
        let (offset, limit) = empty.offset_limit();
        assert_eq!(offset, 0);
        assert_eq!(limit, 20);
    }

    #[test]
    fn test_pagination_params_custom() {
        let params = PaginationParams {
            page: Some(3),
            limit: Some(50),
        };
        assert_eq!(params.page_num(), 3);
        assert_eq!(params.per_page(), 50);
        let (offset, limit) = params.offset_limit();
        assert_eq!(offset, 100);
        assert_eq!(limit, 50);
    }

    #[test]
    fn test_pagination_params_limits() {
        let params = PaginationParams {
            page: Some(-5),
            limit: Some(500),
        };
        assert_eq!(params.page_num(), 1); // Capped by max(1)
        assert_eq!(params.per_page(), 100); // Capped by min(100)
        let (offset, limit) = params.offset_limit();
        assert_eq!(offset, 0);
        assert_eq!(limit, 100);
    }
}
