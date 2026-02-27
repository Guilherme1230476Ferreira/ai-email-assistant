mod api;
mod db;
mod llm;

use axum::{Router, routing::get};

#[tokio::main]
async fn main() {
    let app = Router::new().route("/health", get(|| async { "OK" }));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Backend listening on http://0.0.0.0:3000");
    axum::serve(listener, app).await.unwrap();
}
