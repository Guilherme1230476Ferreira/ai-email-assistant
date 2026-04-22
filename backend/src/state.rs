/// Application state — shared across all handlers via Axum's State extractor.
/// Database pool and service initialization added in US-2.3.
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::infrastructure::config::Config;
use crate::infrastructure::crypto::CryptoService;
use crate::repositories::{user_repo::UserRepository, role_repo::RoleRepository, email_repo::EmailRepository, settings_repo::SettingsRepository};
use crate::services::llm_service::{LlmService, UnifiedLlmService};
use axum::extract::FromRef;

#[derive(Debug, Clone, Default)]
pub struct RuntimeSettings {
    pub api_key: Option<String>,
}

#[derive(Clone)]
pub struct AppState {
    pub user_repo: Arc<UserRepository>,
    pub role_repo: Arc<RoleRepository>,
    pub email_repo: Arc<EmailRepository>,
    pub settings_repo: Arc<SettingsRepository>,
    pub crypto_service: Arc<CryptoService>,
    pub llm_service: Arc<dyn LlmService + Send + Sync>,
    pub config: Arc<Config>,
}

impl FromRef<AppState> for Arc<UserRepository> {
    fn from_ref(state: &AppState) -> Self {
        state.user_repo.clone()
    }
}

impl FromRef<AppState> for Arc<RoleRepository> {
    fn from_ref(state: &AppState) -> Self {
        state.role_repo.clone()
    }
}

impl FromRef<AppState> for Arc<EmailRepository> {
    fn from_ref(state: &AppState) -> Self {
        state.email_repo.clone()
    }
}

impl FromRef<AppState> for Arc<SettingsRepository> {
    fn from_ref(state: &AppState) -> Self {
        state.settings_repo.clone()
    }
}

impl FromRef<AppState> for Arc<CryptoService> {
    fn from_ref(state: &AppState) -> Self {
        state.crypto_service.clone()
    }
}

impl FromRef<AppState> for Arc<dyn LlmService + Send + Sync> {
    fn from_ref(state: &AppState) -> Self {
        state.llm_service.clone()
    }
}

impl FromRef<AppState> for Arc<Config> {
    fn from_ref(state: &AppState) -> Self {
        state.config.clone()
    }
}

impl AppState {
    pub async fn new(config: Config) -> Result<Self, anyhow::Error> {
        let pool = Arc::new(
            sqlx::PgPool::connect(&config.database_url)
                .await
                .expect("failed to connect to database"),
        );
        tracing::info!("Database connection established.");

        let user_repo = Arc::new(UserRepository::new(pool.clone()));
        let role_repo = Arc::new(RoleRepository::new(pool.clone()));
        let email_repo = Arc::new(EmailRepository::new(pool.clone()));

        let crypto_service = Arc::new(CryptoService::new(Arc::new(config.clone()))?);
        let settings_repo = Arc::new(SettingsRepository::new(pool.clone(), (*crypto_service).clone()));

        let llm_service: Arc<dyn LlmService + Send + Sync> =
            Arc::new(UnifiedLlmService::new());

        let app_state = AppState {
            user_repo,
            role_repo,
            email_repo,
            settings_repo,
            crypto_service,
            llm_service,
            config: Arc::new(config),
        };

        Ok(app_state)
    }
}
