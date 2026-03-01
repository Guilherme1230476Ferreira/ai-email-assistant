/// Application state — shared across all handlers via Axum's State extractor.
/// Database pool and service initialization added in US-2.3.
use crate::infrastructure::config::Config;

pub struct AppState {
    pub config: Config,
    // pub db: sqlx::PgPool,                              — added in US-2.3
    // pub llm: Arc<RwLock<Box<dyn LlmService>>>,         — added in US-4.5
}
