use meow_mingle::config::load_config;
use meow_mingle::{create_app, get_db_pool};
use tokio::net::TcpListener;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, fmt};

pub fn setup_tracing() {
    tracing_subscriber::registry()
        .with(fmt::layer().pretty())
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_tracing();

    let pool = get_db_pool().await?;
    let config = load_config();

    let app = create_app(pool.clone(), config).await?;
    let listener = TcpListener::bind("127.0.0.1:3000")
        .await
        .map_err(|e| format!("Failed to bind server: {}", e))?;

    axum::serve(listener, app)
        .await
        .map_err(|e| format!("Server error: {}", e))?;

    Ok(())
}
