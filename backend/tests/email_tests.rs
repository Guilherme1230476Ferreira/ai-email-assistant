/// Integration tests for email history and telemetry endpoints.
/// LLM generation itself is NOT tested here (requires an external API).
/// Routes: GET /api/emails, GET /api/telemetry
mod common;

use axum::http::StatusCode;
use sqlx::PgPool;

use common::*;

// ─── GET /api/emails ──────────────────────────────────────────────────────────

#[sqlx::test]
async fn get_emails_returns_empty_for_new_user(pool: PgPool) {
    let (state, app) = build_test_app(pool).await;
    let (_, user_token) = seed_user(&state, "emptyuser@example.com").await;

    let res = app_get(&app, "/api/emails?page=1&per_page=10", Some(&user_token)).await;

    assert_eq!(res.status(), StatusCode::OK);
    let body = json_body(res).await;
    assert_eq!(body["items"].as_array().map(|a| a.len()).unwrap_or(0), 0);
    assert_eq!(body["total"].as_i64().unwrap_or(-1), 0);
}

#[sqlx::test]
async fn get_emails_requires_authentication(pool: PgPool) {
    let (_, app) = build_test_app(pool).await;
    let res = app_get(&app, "/api/emails", None).await;
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
}

#[sqlx::test]
async fn get_emails_isolates_by_user(pool: PgPool) {
    let (state, app) = build_test_app(pool).await;
    let (user1_id, user1_token) = seed_user(&state, "user1@example.com").await;
    let (user2_id, user2_token) = seed_user(&state, "user2@example.com").await;

    // Seed emails for both users directly
    state
        .email_repo
        .create_email(user1_id, "User1 email", "Reply1", None, None)
        .await
        .unwrap();
    state
        .email_repo
        .create_email(user2_id, "User2 email", "Reply2", None, None)
        .await
        .unwrap();

    let res1 = app_get(&app, "/api/emails?page=1&per_page=10", Some(&user1_token)).await;
    let body1 = json_body(res1).await;
    assert_eq!(
        body1["total"].as_i64().unwrap_or(0),
        1,
        "User1 should only see their own emails"
    );

    let res2 = app_get(&app, "/api/emails?page=1&per_page=10", Some(&user2_token)).await;
    let body2 = json_body(res2).await;
    assert_eq!(
        body2["total"].as_i64().unwrap_or(0),
        1,
        "User2 should only see their own emails"
    );
}

#[sqlx::test]
async fn get_emails_pagination_works(pool: PgPool) {
    let (state, app) = build_test_app(pool).await;
    let (user_id, user_token) = seed_user(&state, "pageduser@example.com").await;

    for i in 0..5 {
        state
            .email_repo
            .create_email(user_id, &format!("Email #{}", i), "Reply", None, None)
            .await
            .unwrap();
    }

    // Page 1 with 2 per page
    let res = app_get(&app, "/api/emails?page=1&per_page=2", Some(&user_token)).await;
    assert_eq!(res.status(), StatusCode::OK);
    let body = json_body(res).await;
    assert_eq!(body["items"].as_array().map(|a| a.len()).unwrap_or(0), 2);
    assert_eq!(body["total"].as_i64().unwrap_or(0), 5);
}

// ─── GET /api/telemetry ───────────────────────────────────────────────────────

#[sqlx::test]
async fn get_telemetry_returns_data_for_authenticated_user(pool: PgPool) {
    let (state, app) = build_test_app(pool).await;
    let (user_id, user_token) = seed_user(&state, "teluser@example.com").await;
    state
        .email_repo
        .create_email(user_id, "Test email", "Test reply", None, None)
        .await
        .unwrap();

    let res = app_get(&app, "/api/telemetry", Some(&user_token)).await;

    assert_eq!(res.status(), StatusCode::OK);
    let body = json_body(res).await;
    // Telemetry fields must be present
    assert!(body.get("tokens_saved").is_some());
    assert!(body.get("context_retrieval_rate").is_some());
    assert!(body.get("avg_similarity_score").is_some());
}

#[sqlx::test]
async fn get_telemetry_requires_authentication(pool: PgPool) {
    let (_, app) = build_test_app(pool).await;
    let res = app_get(&app, "/api/telemetry", None).await;
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
}
