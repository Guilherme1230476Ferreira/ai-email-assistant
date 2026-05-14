/// Application state — shared across all handlers via Axum's State extractor.
/// Database pool and service initialization added in US-2.3.
use std::sync::Arc;

use crate::infrastructure::config::Config;
use crate::infrastructure::crypto::CryptoService;
use crate::repositories::{
    audit_repo::AuditRepository, email_repo::EmailRepository, knowledge_repo::KnowledgeRepository,
    role_repo::RoleRepository, settings_repo::SettingsRepository, user_repo::UserRepository,
};
use crate::services::llm_service::{LlmService, UnifiedLlmService};
use crate::services::rig_service::RigRagService;
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
    pub knowledge_repo: Arc<KnowledgeRepository>,
    pub settings_repo: Arc<SettingsRepository>,
    pub audit_repo: Arc<AuditRepository>,
    pub crypto_service: Arc<CryptoService>,
    pub llm_service: Arc<dyn LlmService + Send + Sync>,
    pub config: Arc<Config>,
    pub rig_service: Arc<RigRagService>,
    pub rate_limiters: Arc<
        tokio::sync::RwLock<std::collections::HashMap<std::net::IpAddr, Vec<std::time::Instant>>>,
    >,
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

impl FromRef<AppState> for Arc<AuditRepository> {
    fn from_ref(state: &AppState) -> Self {
        state.audit_repo.clone()
    }
}

impl FromRef<AppState> for Arc<KnowledgeRepository> {
    fn from_ref(state: &AppState) -> Self {
        state.knowledge_repo.clone()
    }
}

impl FromRef<AppState> for Arc<Config> {
    fn from_ref(state: &AppState) -> Self {
        state.config.clone()
    }
}

impl FromRef<AppState> for Arc<RigRagService> {
    fn from_ref(state: &AppState) -> Self {
        state.rig_service.clone()
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
        let audit_repo = Arc::new(AuditRepository::new((*pool).clone()));
        let knowledge_repo = Arc::new(KnowledgeRepository::new(pool.clone()));

        let crypto_service = Arc::new(CryptoService::new(Arc::new(config.clone()))?);
        let settings_repo = Arc::new(SettingsRepository::new(
            pool.clone(),
            (*crypto_service).clone(),
        ));

        let llm_service: Arc<dyn LlmService + Send + Sync> = Arc::new(UnifiedLlmService::new());

        // Initialize the Rig RAG framework service
        let rig_service = Arc::new(RigRagService::new(
            &config.embedding_api_url,  // Rig uses OpenAI-compatible endpoints
            &config.embedding_api_key,
            "unused",                   // generation model comes from DB settings
            &config.embedding_api_url,
            &config.embedding_api_key,
            &config.embedding_model,
            pool.clone(),
        ));

        let app_state = AppState {
            user_repo,
            role_repo,
            email_repo,
            knowledge_repo,
            settings_repo,
            audit_repo,
            crypto_service,
            llm_service,
            config: Arc::new(config),
            rig_service,
            rate_limiters: Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
        };

        Ok(app_state)
    }
}
