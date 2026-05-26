use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use chrono::{DateTime, Utc};

use crate::{app_error::AppError, middleware::auth::AuthUser, state::AppState};

/// POST /api/extension/ping
/// Called by the Chrome extension on startup and every 60 s.
/// Records the current timestamp for the authenticated user.
#[axum::debug_handler(state = AppState)]
pub async fn extension_ping_handler(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> Result<impl IntoResponse, AppError> {
    let mut map = state.extension_pings.write().await;
    map.insert(auth_user.0.id, Utc::now());
    Ok((StatusCode::OK, Json(serde_json::json!({ "ok": true }))))
}

/// GET /api/extension/status
/// Returns whether the extension has pinged within the last 90 seconds.
#[axum::debug_handler(state = AppState)]
pub async fn extension_status_handler(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> Result<impl IntoResponse, AppError> {
    let map = state.extension_pings.read().await;
    let last_seen: Option<DateTime<Utc>> = map.get(&auth_user.0.id).copied();

    let connected = last_seen
        .map(|t: DateTime<Utc>| (Utc::now() - t).num_seconds() < 90)
        .unwrap_or(false);

    let last_seen_iso: Option<String> = last_seen.map(|t: DateTime<Utc>| t.to_rfc3339());

    Ok((
        StatusCode::OK,
        Json(serde_json::json!({
            "connected": connected,
            "last_seen": last_seen_iso,
        })),
    ))
}
