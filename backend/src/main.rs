mod handlers;
mod infrastructure;
mod models;
mod repositories;
mod router;
mod services;
mod state;

use std::sync::Arc;

use infrastructure::config::Config;
use state::AppState;

#[tokio::main]
async fn main() {
    // Initialize tracing (structured logging)
    tracing_subscriber::fmt::init();

    let config = Config::from_env().expect("failed to load configuration");
    let port = config.port;

    let state = Arc::new(AppState::new(config).await);
    let app = router::build_router(state);

    let addr = format!("0.0.0.0:{}", port);
    tracing::info!("Server running at http://localhost:{}", port);
    tracing::info!("Swagger UI at http://localhost:{}/swagger", port);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
