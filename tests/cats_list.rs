use axum::http::StatusCode;
use axum_test::TestServer;
use dotenv::dotenv;
use meow_mingle::create_app;
use once_cell::sync::OnceCell;
use sqlx::PgPool;
use std::env;

static SERVER: OnceCell<TestServer> = OnceCell::new();

async fn get_server() -> &'static TestServer {
    if let Some(server) = SERVER.get() {
        return server;
    }

    dotenv().ok();
    let database_url = env::var("MM_DATABASE_URL").expect("MM_DATABASE_URL must be set");
    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to database");
    let app = create_app(pool).await.expect("Failed to create app");
    let server = TestServer::new(app);

    SERVER.get_or_init(|| server)
}

#[tokio::test]
async fn test_cats_list_returns_200() {
    let server = get_server().await;
    let response = server.get("/").await;
    response.assert_status(StatusCode::OK);
}

#[tokio::test]
async fn test_cats_list_response_shape() {
    let server = get_server().await;
    let response = server.get("/").await;
    let body = response.json::<serde_json::Value>();

    assert_eq!(body["status"], "OK");
    assert!(body["results"].is_array());
}
