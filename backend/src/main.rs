mod handlers;
mod infrastructure;
mod models;
mod repositories;
mod router;
mod services;
mod state;

#[tokio::main]
async fn main() {
    // Initialize tracing (structured logging)
    tracing_subscriber::fmt::init();

    let app = router::build_router();

    let port = 3000;
    let addr = format!("0.0.0.0:{}", port);
    tracing::info!("Server running at http://localhost:{}", port);
    tracing::info!("Swagger UI at http://localhost:{}/swagger", port);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
