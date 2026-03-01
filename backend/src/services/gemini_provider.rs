/// Gemini provider — stub for Google Gemini API.
/// Real implementation added in a future sprint.
use async_trait::async_trait;

use super::llm_service::LlmService;

pub struct GeminiProvider {
    pub api_key: String,
}

#[async_trait]
impl LlmService for GeminiProvider {
    async fn generate_reply(
        &self,
        _email_body: &str,
        _context: Option<&str>,
        _temperature: f32,
        _max_tokens: i32,
    ) -> anyhow::Result<String> {
        anyhow::bail!("Gemini provider not yet implemented")
    }

    fn provider_name(&self) -> &str {
        "Gemini"
    }
}
