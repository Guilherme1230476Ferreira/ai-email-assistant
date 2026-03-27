/// Application state — shared across all handlers via Axum's State extractor.
/// Database pool and service initialization added in US-2.3.
use tokio::sync::RwLock;

use crate::infrastructure::config::Config;

#[derive(Debug, Clone, Default)]
pub struct RuntimeSettings {
    pub api_key: Option<String>,
}

pub struct AppState {
    pub config: Config,
    pub settings: RwLock<RuntimeSettings>,
    pub http_client: reqwest::Client,
    pub db: sqlx::PgPool,
    // pub llm: Arc<RwLock<Box<dyn LlmService>>>,         — added in US-4.5
}

impl AppState {
    pub async fn new(config: Config) -> Self {
        let db = sqlx::PgPool::connect(&config.database_url)
            .await
            .expect("failed to connect to database");
        tracing::info!("Database connection established.");

        let default_key = config.llm_api_key.clone();

        let settings = RuntimeSettings {
            api_key: if default_key.trim().is_empty() {
                None
            } else {
                Some(default_key)
            },
        };

        Self {
            config,
            settings: RwLock::new(settings),
            http_client: reqwest::Client::new(),
            db,
        }
    }
}
