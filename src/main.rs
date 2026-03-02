use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{routing::get, Json, Router};
use serde_json::json;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // API server
    let app = create_app().await?;

    // Start the server
    let listener = TcpListener::bind("127.0.0.1:3000")
        .await
        .map_err(|e| format!("Failed to bind server: {}", e))?;

    axum::serve(listener, app)
        .await
        .map_err(|e| format!("Server error: {}", e))?;

    Ok(())
}

async fn create_app() -> Result<Router, Box<dyn std::error::Error>> {
    let app = Router::new().route("/", get(welcome_handler));

    Ok(app)
}

#[axum::debug_handler]
async fn welcome_handler() -> impl IntoResponse {
    (
        StatusCode::CREATED,
        Json(json!({"message": "Hello world!"})),
    )
}
