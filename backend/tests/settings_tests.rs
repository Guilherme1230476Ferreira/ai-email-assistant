/// Integration tests for admin settings endpoints.
/// Routes: GET/PUT /api/admin/settings, POST /api/admin/settings/verify
mod common;

use axum::http::StatusCode;
use serde_json::json;
use sqlx::PgPool;

use common::*;

// ─── GET /api/admin/settings ──────────────────────────────────────────────────

#[sqlx::test]
async fn get_settings_returns_singleton(pool: PgPool) {
    let (state, app) = build_test_app(pool).await;
    let (_, admin_token) = seed_admin(&state, "admin@example.com").await;

    let res = app_get(&app, "/api/admin/settings", Some(&admin_token)).await;

    assert_eq!(res.status(), StatusCode::OK);
    let body = json_body(res).await;
    // Must have the base URL and model fields
    assert!(body["llm_base_url"].as_str().is_some());
    assert!(body["llm_model"].as_str().is_some());
    // API key must be masked or absent
    let masked = body["llm_api_key"].as_str().unwrap_or("").to_string();
    assert!(
        !masked.contains("real_secret"),
        "Real key must not be exposed"
    );
}

#[sqlx::test]
async fn get_settings_forbidden_for_regular_user(pool: PgPool) {
    let (state, app) = build_test_app(pool).await;
    let (_, user_token) = seed_user(&state, "user@example.com").await;
    let res = app_get(&app, "/api/admin/settings", Some(&user_token)).await;
    assert_eq!(res.status(), StatusCode::FORBIDDEN);
}

#[sqlx::test]
async fn get_settings_requires_authentication(pool: PgPool) {
    let (_, app) = build_test_app(pool).await;
    let res = app_get(&app, "/api/admin/settings", None).await;
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
}

// ─── PUT /api/admin/settings ──────────────────────────────────────────────────

#[sqlx::test]
async fn update_settings_persists_changes(pool: PgPool) {
    let (state, app) = build_test_app(pool).await;
    let (_, admin_token) = seed_admin(&state, "admin@example.com").await;

    let res = app_put(
        &app,
        "/api/admin/settings",
        json!({
            "llm_base_url": "https://custom.llm.api",
            "llm_model": "custom-model-v1",
            "llm_api_key": null
        }),
        Some(&admin_token),
    )
    .await;

    assert_eq!(res.status(), StatusCode::OK);
    let body = json_body(res).await;
    assert_eq!(body["llm_base_url"], "https://custom.llm.api");
    assert_eq!(body["llm_model"], "custom-model-v1");
}

#[sqlx::test]
async fn update_settings_encrypts_api_key(pool: PgPool) {
    let (state, app) = build_test_app(pool).await;
    let (_, admin_token) = seed_admin(&state, "admin@example.com").await;

    app_put(
        &app,
        "/api/admin/settings",
        json!({
            "llm_base_url": "https://api.openai.com/v1",
            "llm_model": "gpt-4o",
            "llm_api_key": "super_secret_key_123"
        }),
        Some(&admin_token),
    )
    .await;

    // Verify it was encrypted in DB, not stored in plaintext
    let db_settings = state.settings_repo.get_settings().await.unwrap();
    let encrypted_key = db_settings.llm_api_key_encrypted.unwrap();
    assert_ne!(
        encrypted_key, "super_secret_key_123",
        "Key must be stored encrypted"
    );

    // Verify we can still decrypt it
    let decrypted = state
        .settings_repo
        .get_decrypted_api_key()
        .await
        .unwrap()
        .unwrap();
    assert_eq!(decrypted, "super_secret_key_123");
}

#[sqlx::test]
async fn update_settings_ignores_masked_placeholder(pool: PgPool) {
    let (state, app) = build_test_app(pool).await;
    let (_, admin_token) = seed_admin(&state, "admin@example.com").await;

    // First set a real key
    app_put(
        &app,
        "/api/admin/settings",
        json!({
            "llm_base_url": "https://api.openai.com/v1",
            "llm_model": "gpt-4o",
            "llm_api_key": "original_secret_key"
        }),
        Some(&admin_token),
    )
    .await;

    // Then submit with a masked placeholder — should not overwrite
    app_put(
        &app,
        "/api/admin/settings",
        json!({
            "llm_base_url": "https://api.openai.com/v1",
            "llm_model": "gpt-4o",
            "llm_api_key": "sk-...****"
        }),
        Some(&admin_token),
    )
    .await;

    let decrypted = state
        .settings_repo
        .get_decrypted_api_key()
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        decrypted, "original_secret_key",
        "Placeholder should not overwrite existing key"
    );
}

#[sqlx::test]
async fn update_settings_forbidden_for_regular_user(pool: PgPool) {
    let (state, app) = build_test_app(pool).await;
    let (_, user_token) = seed_user(&state, "user@example.com").await;

    let res = app_put(
        &app,
        "/api/admin/settings",
        json!({
            "llm_base_url": "http://evil.server",
            "llm_model": "bad-model",
            "llm_api_key": null
        }),
        Some(&user_token),
    )
    .await;

    assert_eq!(res.status(), StatusCode::FORBIDDEN);
}
