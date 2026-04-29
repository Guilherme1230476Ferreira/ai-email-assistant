/// Integration tests for the health check and general infrastructure.
mod common;

use axum::http::StatusCode;
use sqlx::PgPool;

use common::*;

// ─── GET /health ──────────────────────────────────────────────────────────────

#[sqlx::test]
async fn health_endpoint_returns_200_ok(pool: PgPool) {
    let (_, app) = build_test_app(pool).await;
    let res = app_get(&app, "/health", None).await;
    assert_eq!(res.status(), StatusCode::OK);
}

// ─── Unknown route ────────────────────────────────────────────────────────────

#[sqlx::test]
async fn unknown_route_returns_404(pool: PgPool) {
    let (_, app) = build_test_app(pool).await;
    let res = app_get(&app, "/api/does/not/exist", None).await;
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}
