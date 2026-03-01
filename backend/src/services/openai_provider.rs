/// OpenAI provider — calls OpenAI GPT API to generate replies.
/// Real implementation added in Sprint 4 (US-4.2).
use async_trait::async_trait;

use super::llm_service::LlmService;

pub struct OpenAiProvider {
    pub api_key: String,
}

#[async_trait]
impl LlmService for OpenAiProvider {
    async fn generate_reply(
        &self,
        _email_body: &str,
        _context: Option<&str>,
        _temperature: f32,
        _max_tokens: i32,
    ) -> anyhow::Result<String> {
        anyhow::bail!("OpenAI provider not yet implemented")
    }

    fn provider_name(&self) -> &str {
        "OpenAI"
    }
}
