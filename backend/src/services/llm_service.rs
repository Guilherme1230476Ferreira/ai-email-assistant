use async_trait::async_trait;

use crate::{
    infrastructure::config::{Config, LlmProviderName},
    models::domain::Email,
};

use super::{
    gemini_provider::GeminiProvider,
    openai_provider::OpenAiProvider,
    provider::LlmProvider,
};

#[async_trait]
pub trait LlmService: Send + Sync {
    async fn generate_reply(&self, email: &Email) -> Result<String, anyhow::Error>;
}

pub struct LlmServiceImpl {
    provider: Box<dyn LlmProvider + Send + Sync>,
}

impl LlmServiceImpl {
    pub fn new(config: Config) -> Self {
        let provider: Box<dyn LlmProvider + Send + Sync> = match config.llm_provider_name {
            LlmProviderName::OpenAI => Box::new(OpenAiProvider::new(config)),
            LlmProviderName::Gemini => Box::new(GeminiProvider::new(config)),
        };
        Self { provider }
    }
}

#[async_trait]
impl LlmService for LlmServiceImpl {
    async fn generate_reply(&self, email: &Email) -> Result<String, anyhow::Error> {
        self.provider.generate_reply(email).await
    }
}
