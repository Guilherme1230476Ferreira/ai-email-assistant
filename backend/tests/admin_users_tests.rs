/// Integration tests for admin user management endpoints.
/// Routes: POST/GET /api/admin/users, DELETE /api/admin/users/:id, PUT /api/admin/users/:id/role
mod common;

use axum::http::StatusCode;
use serde_json::json;
use sqlx::PgPool;

use common::*;

// ─── GET /api/admin/users ─────────────────────────────────────────────────────

#[sqlx::test]
async fn admin_get_users_returns_list(pool: PgPool) {
    let (state, app) = build_test_app(pool).await;
    let (_, admin_token) = seed_admin(&state, "admin@example.com").await;
    seed_user(&state, "user1@example.com").await;
    seed_user(&state, "user2@example.com").await;

    let res = app_get(
        &app,
        "/api/admin/users?page=1&per_page=10",
        Some(&admin_token),
    )
    .await;

    assert_eq!(res.status(), StatusCode::OK);
    let body = json_body(res).await;
    assert!(body["items"].as_array().map(|a| a.len()).unwrap_or(0) >= 2);
    assert!(body["total"].as_i64().unwrap_or(0) >= 2);
}

#[sqlx::test]
async fn admin_get_users_blocked_for_regular_users(pool: PgPool) {
    let (state, app) = build_test_app(pool).await;
    let (_, user_token) = seed_user(&state, "regularuser@example.com").await;

    let res = app_get(&app, "/api/admin/users", Some(&user_token)).await;
    assert_eq!(res.status(), StatusCode::FORBIDDEN);
}

#[sqlx::test]
async fn admin_get_users_requires_authentication(pool: PgPool) {
    let (_, app) = build_test_app(pool).await;
    let res = app_get(&app, "/api/admin/users", None).await;
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
}

// ─── POST /api/admin/users ────────────────────────────────────────────────────

#[sqlx::test]
async fn admin_create_user_success(pool: PgPool) {
    let (state, app) = build_test_app(pool).await;
    let (_, admin_token) = seed_admin(&state, "admin@example.com").await;

    let res = app_post(
        &app,
        "/api/admin/users",
        json!({
            "email": "created@example.com",
            "password": "Password123!"
        }),
        Some(&admin_token),
    )
    .await;

    assert_eq!(res.status(), StatusCode::CREATED);
    let body = json_body(res).await;
    assert_eq!(body["email"], "created@example.com");
    assert!(body.get("password_hash").is_none());
}

#[sqlx::test]
async fn admin_create_user_forbidden_for_non_admin(pool: PgPool) {
    let (state, app) = build_test_app(pool).await;
    let (_, user_token) = seed_user(&state, "regularuser@example.com").await;

    let res = app_post(
        &app,
        "/api/admin/users",
        json!({
            "email": "victim@example.com",
            "password": "Password123!"
        }),
        Some(&user_token),
    )
    .await;

    assert_eq!(res.status(), StatusCode::FORBIDDEN);
}

#[sqlx::test]
async fn admin_create_user_conflict_on_duplicate(pool: PgPool) {
    let (state, app) = build_test_app(pool).await;
    let (_, admin_token) = seed_admin(&state, "admin@example.com").await;

    app_post(
        &app,
        "/api/admin/users",
        json!({ "email": "dup@example.com", "password": "Password123!" }),
        Some(&admin_token),
    )
    .await;
    let res = app_post(
        &app,
        "/api/admin/users",
        json!({ "email": "dup@example.com", "password": "Password123!" }),
        Some(&admin_token),
    )
    .await;

    assert_eq!(res.status(), StatusCode::CONFLICT);
}

// ─── PUT /api/admin/users/:id/role ────────────────────────────────────────────

#[sqlx::test]
async fn admin_update_user_role_success(pool: PgPool) {
    let (state, app) = build_test_app(pool).await;
    let (_, admin_token) = seed_admin(&state, "admin@example.com").await;
    let (user_id, _) = seed_user(&state, "target@example.com").await;

    let admin_role = state
        .role_repo
        .get_role_by_name("admin")
        .await
        .unwrap()
        .unwrap();

    let res = app_put(
        &app,
        &format!("/api/admin/users/{user_id}/role"),
        json!({
            "role_id": admin_role.id
        }),
        Some(&admin_token),
    )
    .await;

    assert_eq!(res.status(), StatusCode::OK);
}

#[sqlx::test]
async fn admin_update_user_role_nonexistent_user(pool: PgPool) {
    let (state, app) = build_test_app(pool).await;
    let (_, admin_token) = seed_admin(&state, "admin@example.com").await;
    let fake_id = uuid::Uuid::new_v4();
    let admin_role = state
        .role_repo
        .get_role_by_name("admin")
        .await
        .unwrap()
        .unwrap();

    let res = app_put(
        &app,
        &format!("/api/admin/users/{fake_id}/role"),
        json!({
            "role_id": admin_role.id
        }),
        Some(&admin_token),
    )
    .await;

    // Not found or internal error
    assert!(
        res.status() == StatusCode::NOT_FOUND || res.status() == StatusCode::INTERNAL_SERVER_ERROR
    );
}

// ─── DELETE /api/admin/users/:id ─────────────────────────────────────────────

#[sqlx::test]
async fn admin_delete_user_success(pool: PgPool) {
    let (state, app) = build_test_app(pool).await;
    let (_, admin_token) = seed_admin(&state, "admin@example.com").await;
    let (user_id, _) = seed_user(&state, "todelete@example.com").await;

    let res = app_delete(
        &app,
        &format!("/api/admin/users/{user_id}"),
        Some(&admin_token),
    )
    .await;
    assert_eq!(res.status(), StatusCode::NO_CONTENT);
}

#[sqlx::test]
async fn admin_delete_user_returns_404_for_nonexistent(pool: PgPool) {
    let (state, app) = build_test_app(pool).await;
    let (_, admin_token) = seed_admin(&state, "admin@example.com").await;
    let fake_id = uuid::Uuid::new_v4();

    let res = app_delete(
        &app,
        &format!("/api/admin/users/{fake_id}"),
        Some(&admin_token),
    )
    .await;
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[sqlx::test]
async fn admin_delete_user_forbidden_for_regular_user(pool: PgPool) {
    let (state, app) = build_test_app(pool).await;
    let (_, user_token) = seed_user(&state, "regularuser@example.com").await;
    let (target_id, _) = seed_user(&state, "target@example.com").await;

    let res = app_delete(
        &app,
        &format!("/api/admin/users/{target_id}"),
        Some(&user_token),
    )
    .await;
    assert_eq!(res.status(), StatusCode::FORBIDDEN);
}
