mod config_loader;

use axum::{routing::get, Json, Router};
use serde_json::{json, Value};
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

async fn create_app() -> Result<Router, E> {
    let app = Router::new().route("/", get(welcome_handler));

    Ok(app)
}

fn welcome_handler() -> Json<Value> {
    Json(json!({"message": "User created successfully"}))
}
