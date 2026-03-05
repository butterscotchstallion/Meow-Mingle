use crate::cat::Cat;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{routing::get, Json, Router};
use dotenv::dotenv;
use serde_json::json;
use sqlx::PgPool;
use status::Status;
use std::env;
use tokio::net::TcpListener;

mod cat;
mod status;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables from .env file
    dotenv().ok();

    let database_url = env::var("MM_DATABASE_URL").expect("MM_DATABASE_URL must be set");
    let pool = PgPool::connect(&database_url).await?;

    let app = create_app(pool.clone()).await?;
    let listener = TcpListener::bind("127.0.0.1:3000")
        .await
        .map_err(|e| format!("Failed to bind server: {}", e))?;

    axum::serve(listener, app)
        .await
        .map_err(|e| format!("Server error: {}", e))?;

    Ok(())
}

async fn create_app(pool: PgPool) -> Result<Router, Box<dyn std::error::Error>> {
    let app = Router::new()
        .route("/", get(cats_list_handler))
        .with_state(pool);

    Ok(app)
}

#[axum::debug_handler]
async fn cats_list_handler(
    State(pool): State<PgPool>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let cats: Vec<Cat> = sqlx::query_as!(Cat, "SELECT * FROM cats")
        .fetch_all(&pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "status": Status::Error,
                    "message": e.to_string()
                })),
            )
        })?;
    Ok((
        StatusCode::OK,
        Json(json!({
            "status": Status::Ok,
            "results": cats
        })),
    ))
}
