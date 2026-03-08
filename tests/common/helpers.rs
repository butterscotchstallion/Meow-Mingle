use axum_test::TestServer;
use meow_mingle::{create_app, get_db_pool};
use once_cell::sync::OnceCell;
use std::sync::Once;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

static SERVER: OnceCell<TestServer> = OnceCell::new();
static TRACING_INIT: Once = Once::new();

pub fn setup_tracing() {
    TRACING_INIT.call_once(|| {
        tracing_subscriber::registry()
            .with(fmt::layer().pretty())
            .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
            .init();
    });
}

pub async fn get_server() -> &'static TestServer {
    setup_tracing();

    if let Some(server) = SERVER.get() {
        return server;
    }

    let pool = get_db_pool().await.expect("Failed to get DB pool");
    let app = create_app(pool).await.expect("Failed to create app");
    let server = TestServer::new(app);

    SERVER.get_or_init(|| server)
}
