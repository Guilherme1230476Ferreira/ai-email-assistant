/// Integration tests for authentication endpoints.
/// Routes: POST /api/auth/register, POST /api/auth/login, GET /api/auth/me
mod common;

use axum::http::StatusCode;
use serde_json::json;
use sqlx::PgPool;

use common::*;

// ─── POST /api/auth/register ─────────────────────────────────────────────────

#[sqlx::test]
async fn register_creates_user_and_strips_password_hash(pool: PgPool) {
    let (_, app) = build_test_app(pool).await;

    let res = app_post(
        &app,
        "/api/auth/register",
        json!({
            "email": "newuser@example.com",
            "password": "Password123!"
        }),
        None,
    )
    .await;

    assert_eq!(res.status(), StatusCode::CREATED);
    let body = json_body(res).await;
    assert_eq!(body["email"], "newuser@example.com");
    assert!(
        body.get("password_hash").is_none(),
        "password_hash must NOT be exposed"
    );
}

#[sqlx::test]
async fn register_returns_409_on_duplicate_email(pool: PgPool) {
    let (_, app) = build_test_app(pool).await;

    app_post(
        &app,
        "/api/auth/register",
        json!({ "email": "dup@example.com", "password": "Password123!" }),
        None,
    )
    .await;
    let res = app_post(
        &app,
        "/api/auth/register",
        json!({ "email": "dup@example.com", "password": "Password123!" }),
        None,
    )
    .await;

    assert_eq!(res.status(), StatusCode::CONFLICT);
}

#[sqlx::test]
async fn register_returns_422_on_missing_fields(pool: PgPool) {
    let (_, app) = build_test_app(pool).await;
    let res = app_post(
        &app,
        "/api/auth/register",
        json!({ "email": "nopw@example.com" }),
        None,
    )
    .await;
    assert_eq!(res.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

// ─── POST /api/auth/login ────────────────────────────────────────────────────

#[sqlx::test]
async fn login_success_returns_jwt(pool: PgPool) {
    let (state, app) = build_test_app(pool).await;
    seed_user(&state, "login@example.com").await;

    let res = app_post(
        &app,
        "/api/auth/login",
        json!({
            "email": "login@example.com",
            "password": "Password123!"
        }),
        None,
    )
    .await;

    assert_eq!(res.status(), StatusCode::OK);
    let body = json_body(res).await;
    assert!(
        body["token"].as_str().is_some(),
        "Response must contain a token"
    );
}

#[sqlx::test]
async fn login_wrong_password_returns_401(pool: PgPool) {
    let (state, app) = build_test_app(pool).await;
    seed_user(&state, "wrongpass@example.com").await;

    let res = app_post(
        &app,
        "/api/auth/login",
        json!({
            "email": "wrongpass@example.com",
            "password": "definitely_wrong"
        }),
        None,
    )
    .await;

    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
}

#[sqlx::test]
async fn login_unknown_user_returns_401(pool: PgPool) {
    let (_, app) = build_test_app(pool).await;
    let res = app_post(
        &app,
        "/api/auth/login",
        json!({
            "email": "ghost@example.com",
            "password": "Password123!"
        }),
        None,
    )
    .await;
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
}

// ─── GET /api/auth/me ────────────────────────────────────────────────────────

#[sqlx::test]
async fn me_returns_own_profile(pool: PgPool) {
    let (state, app) = build_test_app(pool).await;
    let (_, token) = seed_user(&state, "me@example.com").await;

    let res = app_get(&app, "/api/auth/me", Some(&token)).await;

    assert_eq!(res.status(), StatusCode::OK);
    let body = json_body(res).await;
    assert_eq!(body["email"], "me@example.com");
    assert!(
        body.get("password_hash").is_none(),
        "password_hash must NOT be in /me response"
    );
}

#[sqlx::test]
async fn me_without_token_returns_401(pool: PgPool) {
    let (_, app) = build_test_app(pool).await;
    let res = app_get(&app, "/api/auth/me", None).await;
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
}

#[sqlx::test]
async fn me_with_garbage_token_returns_401(pool: PgPool) {
    let (_, app) = build_test_app(pool).await;
    let res = app_get(&app, "/api/auth/me", Some("notavalidjwt")).await;
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
}
