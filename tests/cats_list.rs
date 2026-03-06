use axum::http::StatusCode;
use axum_test::TestServer;
use meow_mingle::{create_app, get_db_pool};
use once_cell::sync::OnceCell;

static SERVER: OnceCell<TestServer> = OnceCell::new();

async fn get_server() -> &'static TestServer {
    if let Some(server) = SERVER.get() {
        return server;
    }

    let pool = get_db_pool().await.expect("Failed to get DB pool");
    let app = create_app(pool).await.expect("Failed to create app");
    let server = TestServer::new(app);

    SERVER.get_or_init(|| server)
}

#[tokio::test]
async fn test_cats_list_returns_200() {
    let server = get_server().await;
    let response = server.get("/cats").await;
    response.assert_status(StatusCode::OK);
}

#[tokio::test]
async fn test_cats_list_response_shape() {
    let server = get_server().await;
    let response = server.get("/cats").await;
    let body = response.json::<serde_json::Value>();

    assert_eq!(body["status"], "OK");
    assert!(body["results"].is_array());
}
