use axum::http::StatusCode;
use axum_test::TestServer;
use dotenv::dotenv;
use meow_mingle::create_app;
use sqlx::PgPool;
use std::env;

async fn test_pool() -> PgPool {
    dotenv().ok();
    let database_url = env::var("MM_DATABASE_URL").expect("MM_DATABASE_URL must be set");
    PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to database")
}

#[tokio::test]
async fn test_cats_list_returns_200() {
    let pool = test_pool().await;
    let app = create_app(pool).await.expect("Failed to create app");
    let server = TestServer::new(app).expect("Failed to create test server");

    let response = server.get("/").await;

    response.assert_status(StatusCode::OK);
}

#[tokio::test]
async fn test_cats_list_response_shape() {
    let pool = test_pool().await;
    let app = create_app(pool).await.expect("Failed to create app");
    let server = TestServer::new(app).expect("Failed to create test server");

    let response = server.get("/").await;
    let body = response.json::<serde_json::Value>();

    assert_eq!(body["status"], "OK");
    assert!(body["results"].is_array());
}
