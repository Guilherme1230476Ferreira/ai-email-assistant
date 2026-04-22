use async_trait::async_trait;

use crate::models::domain::Email;

#[async_trait]
pub trait LlmProvider {
    async fn generate_reply(&self, email: &Email) -> Result<String, anyhow::Error>;
}
