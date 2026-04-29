/// Integration tests for audit log endpoints.
/// Routes: GET /api/admin/audit-logs
mod common;

use axum::http::StatusCode;
use serde_json::json;
use sqlx::PgPool;

use common::*;

// ─── GET /api/admin/audit-logs ───────────────────────────────────────────────

#[sqlx::test]
async fn get_audit_logs_requires_admin(pool: PgPool) {
    let (state, app) = build_test_app(pool).await;
    let (_, user_token) = seed_user(&state, "user@example.com").await;

    let res = app_get(&app, "/api/admin/audit-logs", Some(&user_token)).await;
    assert_eq!(res.status(), StatusCode::FORBIDDEN);
}

#[sqlx::test]
async fn get_audit_logs_requires_authentication(pool: PgPool) {
    let (_, app) = build_test_app(pool).await;
    let res = app_get(&app, "/api/admin/audit-logs", None).await;
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
}

#[sqlx::test]
async fn get_audit_logs_records_admin_actions(pool: PgPool) {
    let (state, app) = build_test_app(pool).await;
    let (_, admin_token) = seed_admin(&state, "admin@example.com").await;

    // Perform an admin action (create a user) to generate an audit log
    app_post(
        &app,
        "/api/admin/users",
        json!({
            "email": "auditme@example.com",
            "password": "Password123!"
        }),
        Some(&admin_token),
    )
    .await;

    let res = app_get(
        &app,
        "/api/admin/audit-logs?page=1&per_page=10",
        Some(&admin_token),
    )
    .await;
    assert_eq!(res.status(), StatusCode::OK);
    let body = json_body(res).await;
    assert!(
        body["total"].as_i64().unwrap_or(0) >= 1,
        "Audit log should have been created"
    );
}

#[sqlx::test]
async fn get_audit_logs_supports_pagination(pool: PgPool) {
    let (state, app) = build_test_app(pool).await;
    let (admin_id, admin_token) = seed_admin(&state, "admin@example.com").await;

    // Seed several logs directly
    for i in 0..6 {
        state
            .audit_repo
            .create_log(Some(admin_id), &format!("action_{}", i), None)
            .await
            .unwrap();
    }

    let res = app_get(
        &app,
        "/api/admin/audit-logs?page=1&per_page=3",
        Some(&admin_token),
    )
    .await;
    let body = json_body(res).await;
    assert_eq!(
        body["items"].as_array().map(|a| a.len()).unwrap_or(0),
        3,
        "Should return only 3 logs per page"
    );
    assert!(
        body["total"].as_i64().unwrap_or(0) >= 6,
        "Total should reflect all logs"
    );
}
