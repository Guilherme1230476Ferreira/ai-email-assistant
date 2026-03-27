use std::env;

use anyhow::{Context, Result, anyhow};

/// Application configuration loaded from environment variables.
pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    pub llm_api_key: String,
    pub port: u16,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        // Try backend/.env first (cwd), then root .env for current project layout.
        let _ = dotenv::dotenv();
        let _ = dotenv::from_path("../.env");

        let database_url = env::var("DATABASE_URL")
            .map_err(|_| anyhow!("DATABASE_URL must be set"))
            .context("failed to read DATABASE_URL")?;

        let jwt_secret = env::var("JWT_SECRET")
            .map_err(|_| anyhow!("JWT_SECRET must be set"))
            .context("failed to read JWT_SECRET")?;

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
        let port = env::var("PORT")
            .ok()
            .and_then(|raw| raw.parse::<u16>().ok())
            .unwrap_or(3000);

        Ok(Self {
            database_url,
            jwt_secret,
            llm_api_key,
            port,
        })
    }
}
