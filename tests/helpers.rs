use axum_test::TestServer;
use meow_mingle::{create_app, get_db_pool};
use once_cell::sync::OnceCell;

static SERVER: OnceCell<TestServer> = OnceCell::new();

pub async fn get_server() -> &'static TestServer {
    if let Some(server) = SERVER.get() {
        return server;
    }

    let pool = get_db_pool().await.expect("Failed to get DB pool");
    let app = create_app(pool).await.expect("Failed to create app");
    let server = TestServer::new(app);

    SERVER.get_or_init(|| server)
}
