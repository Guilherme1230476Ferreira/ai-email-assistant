pub mod app_error;
pub mod infrastructure;
pub mod handlers;
pub mod middleware;
pub mod models;
pub mod repositories;
pub mod router;
pub mod services;
pub mod state;

use infrastructure::config::Config;
use router::create_router;
use state::AppState;

pub use app_error::AppError;

#[tokio::main]
async fn main() {
    // Initialize tracing (structured logging)
    tracing_subscriber::fmt::init();

    let config = Config::from_env().expect("failed to load configuration");
    let port = config.port;

        let app_state = AppState::new(config).await.expect("failed to create app state");
    let app = create_router(app_state.clone()).await;

    let addr = format!("0.0.0.0:{}", port);
    tracing::info!("Server running at http://localhost:{}", port);
        tracing::info!(
        "Swagger UI at http://localhost:{}/swagger-ui",
        &app_state.config.port
    );

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
