/// JSON request/response structs — separate from database entities.
/// Will be fleshed out in Sprint 3.
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

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

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateSettingsRequest {
    pub llm_provider: Option<String>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<i32>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SettingsResponse {
    pub llm_provider: String,
    pub temperature: f32,
    pub max_tokens: i32,
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
