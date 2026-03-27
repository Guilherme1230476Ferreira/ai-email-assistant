/// All route mappings in a single place.
use std::sync::Arc;

use axum::{routing::get, routing::post, Router};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::handlers::{admin_handler, role_handler};
use crate::models::dto;
use crate::state::AppState;

#[derive(OpenApi)]
#[openapi(
    paths(
        admin_handler::get_api_key,
        admin_handler::update_api_key,
        admin_handler::create_user,
        admin_handler::update_user_role,
        admin_handler::get_users,
        role_handler::create_role,
        role_handler::get_roles,
    ),
    components(schemas(
        dto::UpdateApiKeyRequest,
        dto::ApiKeyResponse,
        dto::CreateUserRequest,
        dto::UpdateUserRoleRequest,
        dto::UserResponse,
        dto::CreateRoleRequest,
        dto::RoleResponse,
        dto::ErrorResponse,
        crate::models::domain::Role,
    )),
    tags(
        (name = "Admin", description = "Backoffice administration endpoints"),
        (name = "Roles", description = "Role management endpoints"),
        (name = "Health", description = "Health check")
    ),
    info(
        title = "AI Email Assistant API",
        version = "0.1.0",
        description = "REST API for the AI Email Assistant — generates intelligent email replies using LLM providers."
    )
)]
struct ApiDoc;

pub fn build_router(state: Arc<AppState>) -> Router {
    Router::new()
        // Health check
        .route("/health", get(health))
        // Admin endpoints
        .route(
            "/api/admin/llm-provider-api-key",
            get(admin_handler::get_api_key).post(admin_handler::update_api_key),
        )
        .route("/api/admin/users", post(admin_handler::create_user).get(admin_handler::get_users))
        .route("/api/admin/roles", post(role_handler::create_role).get(role_handler::get_roles))
        .route(
            "/api/admin/users/{id}/role",
            post(admin_handler::update_user_role),
        )
        // Swagger UI
        .merge(SwaggerUi::new("/swagger").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .with_state(state)
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
