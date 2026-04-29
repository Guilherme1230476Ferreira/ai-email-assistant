/// Integration tests for role management endpoints.
/// Routes: POST/GET /api/admin/roles, DELETE /api/admin/roles/:id
mod common;

use axum::http::StatusCode;
use serde_json::json;
use sqlx::PgPool;

use common::*;

// ─── GET /api/admin/roles ─────────────────────────────────────────────────────

#[sqlx::test]
async fn get_roles_returns_seeded_roles(pool: PgPool) {
    let (state, app) = build_test_app(pool).await;
    let (_, admin_token) = seed_admin(&state, "admin@example.com").await;

    let res = app_get(&app, "/api/admin/roles", Some(&admin_token)).await;

    assert_eq!(res.status(), StatusCode::OK);
    let body = json_body(res).await;
    let roles = body.as_array().unwrap();
    let role_names: Vec<&str> = roles.iter().filter_map(|r| r["name"].as_str()).collect();
    assert!(role_names.contains(&"admin"), "admin role must be seeded");
    assert!(role_names.contains(&"user"), "user role must be seeded");
}

#[sqlx::test]
async fn get_roles_requires_authentication(pool: PgPool) {
    let (_, app) = build_test_app(pool).await;
    let res = app_get(&app, "/api/admin/roles", None).await;
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
}

#[sqlx::test]
async fn get_roles_forbidden_for_regular_user(pool: PgPool) {
    let (state, app) = build_test_app(pool).await;
    let (_, user_token) = seed_user(&state, "user@example.com").await;
    let res = app_get(&app, "/api/admin/roles", Some(&user_token)).await;
    assert_eq!(res.status(), StatusCode::FORBIDDEN);
}

// ─── POST /api/admin/roles ────────────────────────────────────────────────────

#[sqlx::test]
async fn create_role_success(pool: PgPool) {
    let (state, app) = build_test_app(pool).await;
    let (_, admin_token) = seed_admin(&state, "admin@example.com").await;

    let res = app_post(
        &app,
        "/api/admin/roles",
        json!({
            "name": "moderator"
        }),
        Some(&admin_token),
    )
    .await;

    assert_eq!(res.status(), StatusCode::CREATED);
    let body = json_body(res).await;
    assert_eq!(body["name"], "moderator");
}

#[sqlx::test]
async fn create_role_conflict_on_duplicate_name(pool: PgPool) {
    let (state, app) = build_test_app(pool).await;
    let (_, admin_token) = seed_admin(&state, "admin@example.com").await;

    app_post(
        &app,
        "/api/admin/roles",
        json!({ "name": "moderator" }),
        Some(&admin_token),
    )
    .await;
    let res = app_post(
        &app,
        "/api/admin/roles",
        json!({ "name": "moderator" }),
        Some(&admin_token),
    )
    .await;

    assert_eq!(res.status(), StatusCode::CONFLICT);
}

#[sqlx::test]
async fn create_role_forbidden_for_regular_user(pool: PgPool) {
    let (state, app) = build_test_app(pool).await;
    let (_, user_token) = seed_user(&state, "user@example.com").await;

    let res = app_post(
        &app,
        "/api/admin/roles",
        json!({ "name": "hacker" }),
        Some(&user_token),
    )
    .await;
    assert_eq!(res.status(), StatusCode::FORBIDDEN);
}

// ─── DELETE /api/admin/roles/:id ─────────────────────────────────────────────

#[sqlx::test]
async fn delete_role_success(pool: PgPool) {
    let (state, app) = build_test_app(pool).await;
    let (_, admin_token) = seed_admin(&state, "admin@example.com").await;

    // Create a temporary role first
    let create_res = app_post(
        &app,
        "/api/admin/roles",
        json!({ "name": "temp_role" }),
        Some(&admin_token),
    )
    .await;
    let role_body = json_body(create_res).await;
    let role_id = role_body["id"].as_str().unwrap();

    let res = app_delete(
        &app,
        &format!("/api/admin/roles/{role_id}"),
        Some(&admin_token),
    )
    .await;
    assert_eq!(res.status(), StatusCode::NO_CONTENT);
}

#[sqlx::test]
async fn delete_role_not_found(pool: PgPool) {
    let (state, app) = build_test_app(pool).await;
    let (_, admin_token) = seed_admin(&state, "admin@example.com").await;
    let fake_id = uuid::Uuid::new_v4();

    let res = app_delete(
        &app,
        &format!("/api/admin/roles/{fake_id}"),
        Some(&admin_token),
    )
    .await;
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}
