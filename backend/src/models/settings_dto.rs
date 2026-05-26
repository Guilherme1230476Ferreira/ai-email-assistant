use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateSettingsRequest {
    pub llm_base_url: String,
    pub llm_model: String,
    pub llm_api_key: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct VerifySettingsRequest {
    pub llm_base_url: Option<String>,
    pub llm_model: Option<String>,
    pub llm_api_key: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SettingsResponse {
    pub id: String,
    pub llm_base_url: String,
    pub llm_model: String,
    pub has_api_key: bool,
    pub masked_api_key: Option<String>,
    /// The active embedding model read from EMBEDDING_MODEL env var.
    pub embedding_model: String,
}
