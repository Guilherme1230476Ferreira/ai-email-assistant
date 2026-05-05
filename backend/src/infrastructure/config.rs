use std::env;

use anyhow::{Context, Result, anyhow};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, Default)]
pub enum LlmProviderName {
    #[default]
    OpenAI,
    Gemini,
}

#[derive(Debug, Clone)]
/// Application configuration loaded from environment variables.
pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    pub jwt_expiration_hours: i64,
    pub encryption_key: String,
    pub llm_api_key: String,
    pub llm_provider_name: LlmProviderName,
    pub port: u16,
    pub embedding_api_url: String,
    pub embedding_api_key: String,
    pub embedding_model: String,
    pub google_client_id: Option<String>,
    pub google_client_secret: Option<String>,
    /// The public-facing URL where users access the app (e.g. http://vs224.dei.isep.ipp.pt:2224).
    /// Used for OAuth redirect URIs and post-login redirects.
    /// Defaults to http://localhost:5173 for local development.
    pub public_url: String,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        // Try backend/.env first (cwd), then root .env for current project layout.
        let _ = dotenv::dotenv();
        let _ = dotenv::from_path("../.env");

        let llm_provider_name = env::var("LLM_PROVIDER")
            .ok()
            .and_then(|s| serde_json::from_str(&format!("\"{}\"", s)).ok())
            .unwrap_or(LlmProviderName::Gemini);

        let database_url = env::var("DATABASE_URL")
            .map_err(|_| anyhow!("DATABASE_URL must be set"))
            .context("failed to read DATABASE_URL")?;

        let jwt_secret = env::var("JWT_SECRET")
            .map_err(|_| anyhow!("JWT_SECRET must be set"))
            .context("failed to read JWT_SECRET")?;

        let jwt_expiration_hours = env::var("JWT_EXPIRATION_HOURS")
            .ok()
            .and_then(|raw| raw.parse::<i64>().ok())
            .unwrap_or(24);

        let encryption_key = env::var("ENCRYPTION_KEY")
            .unwrap_or_else(|_| "00000000000000000000000000000000".to_string());

        // Backward compatible: prefer LLM_API_KEY, fallback to provider-specific env vars.
        let llm_api_key = env::var("LLM_API_KEY")
            .ok()
            .filter(|value| !value.trim().is_empty())
            .or_else(|| {
                env::var("OPENAI_API_KEY")
                    .ok()
                    .filter(|value| !value.trim().is_empty())
            })
            .or_else(|| {
                env::var("GEMINI_API_KEY")
                    .ok()
                    .filter(|value| !value.trim().is_empty())
            })
            .unwrap_or_default();

        let embedding_api_url = env::var("EMBEDDING_API_URL")
            .unwrap_or_else(|_| "https://api.openai.com/v1".to_string());

        let embedding_api_key = env::var("EMBEDDING_API_KEY")
            .or_else(|_| env::var("OPENAI_API_KEY"))
            .unwrap_or_default();

        let embedding_model =
            env::var("EMBEDDING_MODEL").unwrap_or_else(|_| "text-embedding-3-small".to_string());

        let port = env::var("PORT")
            .ok()
            .and_then(|raw| raw.parse::<u16>().ok())
            .unwrap_or(3000);

        let google_client_id = env::var("GOOGLE_CLIENT_ID").ok();
        let google_client_secret = env::var("GOOGLE_CLIENT_SECRET").ok();

        let public_url = env::var("PUBLIC_URL")
            .unwrap_or_else(|_| "http://localhost:5173".to_string())
            .trim_end_matches('/')
            .to_string();

        Ok(Self {
            database_url,
            jwt_secret,
            jwt_expiration_hours,
            encryption_key,
            llm_api_key,
            llm_provider_name,
            port,
            google_client_id,
            google_client_secret,
            embedding_api_url,
            embedding_api_key,
            embedding_model,
            public_url,
        })
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            database_url: "postgres://localhost/test".to_string(),
            jwt_secret: "test_secret".to_string(),
            jwt_expiration_hours: 24,
            encryption_key: "00000000000000000000000000000000".to_string(),
            llm_api_key: "".to_string(),
            llm_provider_name: LlmProviderName::OpenAI,
            port: 3000,
            embedding_api_url: "http://localhost".to_string(),
            embedding_api_key: "".to_string(),
            embedding_model: "test-model".to_string(),
            google_client_id: None,
            google_client_secret: None,
            public_url: "http://localhost:5173".to_string(),
        }
    }
}
