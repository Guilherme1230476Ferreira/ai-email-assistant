use async_trait::async_trait;

use crate::{infrastructure::config::Config, models::domain::Email};

use super::provider::LlmProvider;

pub struct OpenAiProvider {
    _config: Config,
}

impl OpenAiProvider {
    pub fn new(config: Config) -> Self {
        Self { _config: config }
    }
}

#[async_trait]
impl LlmProvider for OpenAiProvider {
    async fn generate_reply(&self, _email: &Email) -> Result<String, anyhow::Error> {
        // TODO: Implement OpenAI provider
        Ok("Not implemented".to_string())
    }
}