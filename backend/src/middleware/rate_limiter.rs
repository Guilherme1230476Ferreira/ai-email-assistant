use std::sync::Arc;
use std::time::{Duration, Instant};

use axum::{
    extract::{ConnectInfo, Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use std::net::SocketAddr;

use crate::{app_error::AppError, state::AppState};

pub async fn rate_limit_middleware(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let ip = addr.ip();
    let now = Instant::now();
    let window = Duration::from_secs(60);
    let max_requests = 5;

    let mut rate_limiters = state.rate_limiters.write().await;

    let requests = rate_limiters.entry(ip).or_insert_with(Vec::new);

    // Remove old requests
    requests.retain(|&time| now.duration_since(time) < window);

    if requests.len() >= max_requests {
        return Err(AppError::new(
            StatusCode::TOO_MANY_REQUESTS,
            "Too many login attempts. Please try again later.",
        ));
    }

    requests.push(now);

    Ok(next.run(request).await)
}
