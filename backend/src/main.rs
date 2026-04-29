use ai_email_assistant::infrastructure::config::Config;
use ai_email_assistant::router::create_router;
use ai_email_assistant::state::AppState;

#[tokio::main]
async fn main() {
    // Initialize tracing (structured logging)
    tracing_subscriber::fmt::init();

    let config = Config::from_env().expect("failed to load configuration");
    let port = config.port;

    let app_state = AppState::new(config)
        .await
        .expect("failed to create app state");
    let app = create_router(app_state.clone()).await;

    let addr = format!("0.0.0.0:{}", port);
    tracing::info!("Server running at http://localhost:{}", port);
    tracing::info!(
        "Swagger UI at http://localhost:{}/swagger-ui",
        &app_state.config.port
    );

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .await
    .unwrap();
}
