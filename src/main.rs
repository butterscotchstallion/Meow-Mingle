use meow_mingle::{create_app, get_db_pool};
use tokio::net::TcpListener;

mod hasher;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pool = get_db_pool().await?;

    let app = create_app(pool.clone()).await?;
    let listener = TcpListener::bind("127.0.0.1:3000")
        .await
        .map_err(|e| format!("Failed to bind server: {}", e))?;

    axum::serve(listener, app)
        .await
        .map_err(|e| format!("Server error: {}", e))?;

    Ok(())
}
