use axum::{routing::get, Router};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::{
    app_error::AppError,
    handlers::{
        admin_handler, auth_handler, email_handler,
        role_handler, settings_handler,
    },
    models::{
        domain::{Email, Role, User},
        dto::{
            CreateRoleRequest, CreateUserRequest, GenerateEmailRequest, LoginRequest,
            LoginResponse, UpdateUserRoleRequest,
        },
        settings_dto::{SettingsResponse, UpdateSettingsRequest, VerifySettingsRequest},
    },
    state::AppState,
};
use std::sync::Arc;

#[derive(OpenApi)]
#[openapi(
    paths(
        auth_handler::login_handler,
        admin_handler::create_user_handler,
        admin_handler::get_users_handler,
        admin_handler::update_user_role_handler,
        admin_handler::delete_user_handler,
        role_handler::create_role_handler,
        role_handler::get_roles_handler,
        role_handler::delete_role_handler,
        email_handler::generate_email_handler,
        email_handler::get_emails_handler,
        settings_handler::get_settings_handler,
        settings_handler::update_settings_handler,
        settings_handler::verify_settings_handler,
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
        AppError,
        Role,
        User,
        Email,
    )),
    tags(
        (name = "Admin", description = "Admin management endpoints"),
        (name = "Auth", description = "Authentication endpoints"),
        (name = "Email", description = "Email generation endpoints"),
        (name = "Health", description = "Health check endpoint"),
        (name = "Role", description = "Role management endpoints"),
        (name = "Admin Settings", description = "LLM Settings endpoints")
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
    Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .route("/health", get(health))
        .route(
            "/api/auth/login",
            axum::routing::post(auth_handler::login_handler),
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
            "/api/emails/generate",
            axum::routing::post(email_handler::generate_email_handler),
        )
        .route(
            "/api/emails",
            axum::routing::get(email_handler::get_emails_handler),
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
        .with_state(app_state)
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
