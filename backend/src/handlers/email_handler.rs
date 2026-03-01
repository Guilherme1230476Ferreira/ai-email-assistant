/// Email handlers — HTTP endpoints for email operations.
/// Real implementations added in Sprint 3/4.
use axum::Json;

use crate::models::dto::{GenerateReplyRequest, GenerateReplyResponse, HistoryQuery};

#[utoipa::path(
    post,
    path = "/api/emails/generate-reply",
    request_body = GenerateReplyRequest,
    responses(
        (status = 200, description = "AI-generated reply", body = GenerateReplyResponse),
        (status = 400, description = "Invalid request"),
        (status = 500, description = "LLM service error")
    ),
    tag = "Emails"
)]
pub async fn generate_reply(Json(_body): Json<GenerateReplyRequest>) -> &'static str {
    // TODO: Sprint 4 — call LLM service and return generated reply
    "not implemented"
}

#[utoipa::path(
    get,
    path = "/api/emails/history",
    params(HistoryQuery),
    responses(
        (status = 200, description = "Email history list")
    ),
    tag = "Emails"
)]
pub async fn get_history(
    axum::extract::Query(_query): axum::extract::Query<HistoryQuery>,
) -> &'static str {
    // TODO: Sprint 3 — query email history from database
    "not implemented"
}
