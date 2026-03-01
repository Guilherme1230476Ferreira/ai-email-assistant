/// Admin handlers — HTTP endpoints for backoffice operations.
/// Real implementations added in Sprint 3.
use axum::Json;

use crate::models::dto::{LogsQuery, SettingsResponse, UpdateSettingsRequest};

#[utoipa::path(
    get,
    path = "/api/admin/settings",
    responses(
        (status = 200, description = "Current LLM settings", body = SettingsResponse)
    ),
    tag = "Admin"
)]
pub async fn get_settings() -> &'static str {
    // TODO: Sprint 3 — read settings from database
    "not implemented"
}

#[utoipa::path(
    post,
    path = "/api/admin/settings",
    request_body = UpdateSettingsRequest,
    responses(
        (status = 200, description = "Settings updated")
    ),
    tag = "Admin"
)]
pub async fn update_settings(Json(_body): Json<UpdateSettingsRequest>) -> &'static str {
    // TODO: Sprint 3 — update settings in database
    "not implemented"
}

#[utoipa::path(
    get,
    path = "/api/admin/logs",
    params(LogsQuery),
    responses(
        (status = 200, description = "Audit log entries")
    ),
    tag = "Admin"
)]
pub async fn get_logs(
    axum::extract::Query(_query): axum::extract::Query<LogsQuery>,
) -> &'static str {
    // TODO: Sprint 3 — query audit logs from database
    "not implemented"
}
