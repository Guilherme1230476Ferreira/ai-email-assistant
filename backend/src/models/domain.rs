/// Database entity structs — mapped from table rows via sqlx::FromRow.
/// Real fields will be added when implementing Sprint 3.
use chrono::{DateTime, Utc};
use pgvector::Vector;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, utoipa::ToSchema)]
pub struct Role {
    pub id: Uuid,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, utoipa::ToSchema)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub role_id: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, utoipa::ToSchema)]
pub struct AppSetting {
    pub id: String,
    pub llm_base_url: String,
    pub llm_model: String,
    pub llm_api_key_encrypted: Option<String>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize, utoipa::ToSchema)]
pub struct Email {
    pub id: Uuid,
    pub user_id: Uuid,
    pub original_content: String,
    pub generated_response: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize, utoipa::ToSchema)]
pub struct EmailEmbedding {
    pub id: Uuid,
    pub email_id: Uuid,
    #[serde(skip)]
    pub content_embedding: Option<Vector>,
    #[serde(skip)]
    pub response_embedding: Option<Vector>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AuditLog {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub action: String,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}
