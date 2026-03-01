/// LlmService trait — defines the contract for any LLM provider (Strategy Pattern).
/// Real implementations added in Sprint 4.
use async_trait::async_trait;

#[async_trait]
pub trait LlmService: Send + Sync {
    async fn generate_reply(
        &self,
        email_body: &str,
        context: Option<&str>,
        temperature: f32,
        max_tokens: i32,
    ) -> anyhow::Result<String>;

    fn provider_name(&self) -> &str;
}
