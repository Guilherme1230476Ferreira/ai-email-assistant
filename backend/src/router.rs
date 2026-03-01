/// All route mappings in a single place.
use axum::{routing::get, routing::post, Router};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::handlers::{admin_handler, email_handler};
use crate::models::dto;

#[derive(OpenApi)]
#[openapi(
    paths(
        email_handler::generate_reply,
        email_handler::get_history,
        admin_handler::get_settings,
        admin_handler::update_settings,
        admin_handler::get_logs,
    ),
    components(schemas(
        dto::GenerateReplyRequest,
        dto::GenerateReplyResponse,
        dto::UpdateSettingsRequest,
        dto::SettingsResponse,
        dto::HistoryQuery,
        dto::LogsQuery,
    )),
    tags(
        (name = "Emails", description = "Email processing endpoints"),
        (name = "Admin", description = "Backoffice administration endpoints"),
        (name = "Health", description = "Health check")
    ),
    info(
        title = "AI Email Assistant API",
        version = "0.1.0",
        description = "REST API for the AI Email Assistant — generates intelligent email replies using LLM providers."
    )
)]
struct ApiDoc;

pub fn build_router() -> Router {
    Router::new()
        // Health check
        .route("/health", get(health))
        // Email endpoints
        .route("/api/emails/generate-reply", post(email_handler::generate_reply))
        .route("/api/emails/history", get(email_handler::get_history))
        // Admin endpoints
        .route("/api/admin/settings", get(admin_handler::get_settings))
        .route("/api/admin/settings", post(admin_handler::update_settings))
        .route("/api/admin/logs", get(admin_handler::get_logs))
        // Swagger UI
        .merge(SwaggerUi::new("/swagger").url("/api-docs/openapi.json", ApiDoc::openapi()))
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
