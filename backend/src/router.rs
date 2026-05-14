use axum::{Router, routing::get};
use tower_http::cors::{CorsLayer, Any};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::{
    app_error::AppError,
    handlers::{admin_handler, auth_handler, email_handler, knowledge_handler, role_handler, settings_handler},
    models::{
        domain::{Email, Role, User},
        dto::{
            CreateRoleRequest, CreateUserRequest, GenerateEmailRequest, LoginRequest,
            LoginResponse, UpdateUserRoleRequest, CreateQAPairRequest, KnowledgeEntryResponse,
        },
        settings_dto::{SettingsResponse, UpdateSettingsRequest, VerifySettingsRequest},
    },
    state::AppState,
};

#[derive(OpenApi)]
#[openapi(
    paths(
        auth_handler::login_handler,
        auth_handler::me_handler,
        admin_handler::create_user_handler,
        admin_handler::get_users_handler,
        admin_handler::update_user_role_handler,
        admin_handler::delete_user_handler,
        role_handler::create_role_handler,
        role_handler::get_roles_handler,
        role_handler::delete_role_handler,
        email_handler::generate_email_handler,
        email_handler::generate_email_stream_handler,
        email_handler::get_emails_handler,
        email_handler::delete_email_handler,
        email_handler::get_telemetry_handler,
        settings_handler::get_settings_handler,
        settings_handler::update_settings_handler,
        settings_handler::verify_settings_handler,
        knowledge_handler::create_qa_pair_handler,
        knowledge_handler::upload_document_handler,
        knowledge_handler::get_knowledge_entries_handler,
        knowledge_handler::delete_knowledge_entry_handler,
        health,
    ),
    components(schemas(
        CreateRoleRequest,
        CreateUserRequest,
        GenerateEmailRequest,
        LoginRequest,
        LoginResponse,
        UpdateUserRoleRequest,
        SettingsResponse,
        UpdateSettingsRequest,
        VerifySettingsRequest,
        crate::models::dto::TelemetryData,
        crate::models::dto::UserResponse,
        AppError,
        Role,
        User,
        Email,
        crate::models::domain::KnowledgeEntry,
        CreateQAPairRequest,
        KnowledgeEntryResponse,
    )),
    tags(
        (name = "Admin", description = "Admin management endpoints"),
        (name = "Auth", description = "Authentication endpoints"),
        (name = "Email", description = "Email generation endpoints"),
        (name = "Health", description = "Health check endpoint"),
        (name = "Role", description = "Role management endpoints"),
        (name = "Admin Settings", description = "LLM Settings endpoints"),
        (name = "Knowledge Base", description = "RAG knowledge management endpoints")
    ),
    modifiers(&SecurityAddon)
)]
struct ApiDoc;

struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.get_or_insert_with(Default::default);
        components.add_security_scheme(
            "bearer_auth",
            utoipa::openapi::security::SecurityScheme::Http(
                utoipa::openapi::security::HttpBuilder::new()
                    .scheme(utoipa::openapi::security::HttpAuthScheme::Bearer)
                    .bearer_format("JWT")
                    .build(),
            ),
        )
    }
}

pub async fn create_router(app_state: AppState) -> Router {
    let router = Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .route("/health", get(health))
        .route(
            "/api/auth/login",
            axum::routing::post(auth_handler::login_handler).route_layer(
                axum::middleware::from_fn_with_state(
                    app_state.clone(),
                    crate::middleware::rate_limiter::rate_limit_middleware,
                ),
            ),
        )
        .route(
            "/api/auth/register",
            axum::routing::post(auth_handler::register_handler),
        )
        .route("/api/auth/me", axum::routing::get(auth_handler::me_handler))
        .route("/api/auth/google", get(auth_handler::google_auth_handler))
        .route(
            "/api/auth/google/callback",
            get(auth_handler::google_callback_handler),
        )
        .route(
            "/api/admin/users",
            axum::routing::post(admin_handler::create_user_handler)
                .get(admin_handler::get_users_handler),
        )
        .route(
            "/api/admin/users/:user_id",
            axum::routing::delete(admin_handler::delete_user_handler),
        )
        .route(
            "/api/admin/users/:user_id/role",
            axum::routing::put(admin_handler::update_user_role_handler),
        )
        .route(
            "/api/admin/roles",
            axum::routing::post(role_handler::create_role_handler)
                .get(role_handler::get_roles_handler),
        )
        .route(
            "/api/admin/roles/:role_id",
            axum::routing::delete(role_handler::delete_role_handler),
        )
        .route(
            "/api/admin/audit-logs",
            axum::routing::get(admin_handler::get_audit_logs_handler),
        )
        .route(
            "/api/emails/generate",
            axum::routing::post(email_handler::generate_email_handler),
        )
        .route(
            "/api/emails/generate/stream",
            axum::routing::post(email_handler::generate_email_stream_handler),
        )
        .route(
            "/api/emails",
            axum::routing::get(email_handler::get_emails_handler),
        )
        .route(
            "/api/emails/:id",
            axum::routing::delete(email_handler::delete_email_handler),
        )
        .route(
            "/api/telemetry",
            axum::routing::get(email_handler::get_telemetry_handler),
        )
        .route(
            "/api/admin/settings",
            axum::routing::get(settings_handler::get_settings_handler)
                .put(settings_handler::update_settings_handler),
        )
        .route(
            "/api/admin/settings/verify",
            axum::routing::post(settings_handler::verify_settings_handler),
        )
        .route(
            "/api/knowledge",
            axum::routing::get(knowledge_handler::get_knowledge_entries_handler)
                .post(knowledge_handler::create_qa_pair_handler),
        )
        .route(
            "/api/knowledge/upload",
            axum::routing::post(knowledge_handler::upload_document_handler),
        )
        .route(
            "/api/knowledge/:id",
            axum::routing::delete(knowledge_handler::delete_knowledge_entry_handler),
        )
        .with_state(app_state);

    // CORS: allow the Chrome extension (from mail.google.com) to call the API
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    router.layer(cors)
}

#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "Server is alive", body = String)
    ),
    tag = "Health"
)]
async fn health() -> &'static str {
    "OK"
}
