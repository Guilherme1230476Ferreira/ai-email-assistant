#![allow(dead_code)]
#[allow(dead_code)]
/// Shared integration test utilities.
/// All integration test files in /tests declare `mod common;` to access this.
use std::{collections::HashMap, net::IpAddr, sync::Arc};

use axum::{
    Router,
    body::Body,
    extract::ConnectInfo,
    http::{Request, header},
};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use sqlx::PgPool;
use std::net::SocketAddr;
use tokio::sync::RwLock;
use tower::ServiceExt;
use uuid::Uuid;

use ai_email_assistant::{
    infrastructure::{config::Config, crypto::CryptoService},
    middleware::auth::create_jwt,
    models::dto::CreateUserRequest,
    repositories::{
        audit_repo::AuditRepository, email_repo::EmailRepository, role_repo::RoleRepository,
        settings_repo::SettingsRepository, user_repo::UserRepository,
    },
    router::create_router,
    services::llm_service::UnifiedLlmService,
    state::AppState,
};

/// Build an AppState wired to the isolated test pool.
pub async fn build_test_state(pool: PgPool) -> AppState {
    let config = Arc::new(Config {
        jwt_secret: "integration_test_jwt_secret_32chars_!!".to_string(),
        jwt_expiration_hours: 24,
        encryption_key: "integration_enc_key_thats_32bytes_!".to_string(),
        ..Config::default()
    });

    let arc_pool = Arc::new(pool.clone());
    let crypto = Arc::new(CryptoService::new(config.clone()).unwrap());

    AppState {
        user_repo: Arc::new(UserRepository::new(arc_pool.clone())),
        role_repo: Arc::new(RoleRepository::new(arc_pool.clone())),
        email_repo: Arc::new(EmailRepository::new(arc_pool.clone())),
        settings_repo: Arc::new(SettingsRepository::new(arc_pool.clone(), (*crypto).clone())),
        audit_repo: Arc::new(AuditRepository::new(pool)),
        crypto_service: crypto,
        llm_service: Arc::new(UnifiedLlmService::new()),
        config,
        rate_limiters: Arc::new(RwLock::new(
            HashMap::<IpAddr, Vec<std::time::Instant>>::new(),
        )),
    }
}

/// Build the full Axum Router for use in tests.
pub async fn build_test_app(pool: PgPool) -> (AppState, Router) {
    let state = build_test_state(pool).await;
    let router = create_router(state.clone()).await;
    (state, router)
}

/// Seed a regular `user`-role user and return their JWT.
pub async fn seed_user(state: &AppState, email: &str) -> (Uuid, String) {
    let user = state
        .user_repo
        .create_user(&CreateUserRequest {
            email: email.to_string(),
            password: "Password123!".to_string(),
        })
        .await
        .unwrap();
    let token = create_jwt(user.id, &state.config).unwrap();
    (user.id, token)
}

/// Seed an `admin`-role user and return their JWT.
pub async fn seed_admin(state: &AppState, email: &str) -> (Uuid, String) {
    let user = state
        .user_repo
        .create_user(&CreateUserRequest {
            email: email.to_string(),
            password: "Password123!".to_string(),
        })
        .await
        .unwrap();
    let admin_role = state
        .role_repo
        .get_role_by_name("admin")
        .await
        .unwrap()
        .unwrap();
    state
        .user_repo
        .update_user_role(user.id, admin_role.id)
        .await
        .unwrap();
    let token = create_jwt(user.id, &state.config).unwrap();
    (user.id, token)
}

// ─── HTTP helpers ─────────────────────────────────────────────────────────────

pub async fn app_post(
    app: &Router,
    path: &str,
    body: Value,
    token: Option<&str>,
) -> axum::response::Response {
    let mut b = Request::builder()
        .method("POST")
        .uri(path)
        .header(header::CONTENT_TYPE, "application/json");
    if let Some(t) = token {
        b = b.header(header::AUTHORIZATION, format!("Bearer {t}"));
    }
    let mut req = b.body(Body::from(body.to_string())).unwrap();
    req.extensions_mut()
        .insert(ConnectInfo(SocketAddr::from(([127, 0, 0, 1], 1234))));
    app.clone().oneshot(req).await.unwrap()
}

pub async fn app_get(app: &Router, path: &str, token: Option<&str>) -> axum::response::Response {
    let mut b = Request::builder().method("GET").uri(path);
    if let Some(t) = token {
        b = b.header(header::AUTHORIZATION, format!("Bearer {t}"));
    }
    let mut req = b.body(Body::empty()).unwrap();
    req.extensions_mut()
        .insert(ConnectInfo(SocketAddr::from(([127, 0, 0, 1], 1234))));
    app.clone().oneshot(req).await.unwrap()
}

pub async fn app_put(
    app: &Router,
    path: &str,
    body: Value,
    token: Option<&str>,
) -> axum::response::Response {
    let mut b = Request::builder()
        .method("PUT")
        .uri(path)
        .header(header::CONTENT_TYPE, "application/json");
    if let Some(t) = token {
        b = b.header(header::AUTHORIZATION, format!("Bearer {t}"));
    }
    let mut req = b.body(Body::from(body.to_string())).unwrap();
    req.extensions_mut()
        .insert(ConnectInfo(SocketAddr::from(([127, 0, 0, 1], 1234))));
    app.clone().oneshot(req).await.unwrap()
}

pub async fn app_delete(app: &Router, path: &str, token: Option<&str>) -> axum::response::Response {
    let mut b = Request::builder().method("DELETE").uri(path);
    if let Some(t) = token {
        b = b.header(header::AUTHORIZATION, format!("Bearer {t}"));
    }
    let mut req = b.body(Body::empty()).unwrap();
    req.extensions_mut()
        .insert(ConnectInfo(SocketAddr::from(([127, 0, 0, 1], 1234))));
    app.clone().oneshot(req).await.unwrap()
}

/// Extract response body as a JSON value.
pub async fn json_body(res: axum::response::Response) -> Value {
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap_or(json!(null))
}
