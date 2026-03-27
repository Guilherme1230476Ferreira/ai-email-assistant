/// Database entity structs — mapped from table rows via sqlx::FromRow.
/// Real fields will be added when implementing Sprint 3.
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, utoipa::ToSchema)]
pub struct Role {
    pub id: Uuid,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub password_hash: String,
    pub role_id: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Settings {
    pub id: Uuid,
    pub user_id: Uuid,
    pub llm_provider: String,
    pub api_key: Option<String>,
    pub max_tokens: i32,
    pub temperature: f64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Email {
    pub id: Uuid,
    pub user_id: Uuid,
    pub sender: String,
    pub subject: Option<String>,
    pub body: String,
    pub generated_reply: Option<String>,
    pub is_archived: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct EmailEmbedding {
    pub id: Uuid,
    pub email_id: Uuid,
    pub created_at: DateTime<Utc>,
    // Note: embedding (vector) field handled separately via pgvector
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AuditLog {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub action: String,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}
