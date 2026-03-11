use axum::http::StatusCode;

mod common;
use common::helpers::get_server;

#[tokio::test]
async fn test_openapi_json_returns_200() {
    let server = get_server().await;
    let response = server.get("/api-docs/openapi.json").await;
    response.assert_status(StatusCode::OK);
}

#[tokio::test]
async fn test_openapi_json_is_valid() {
    let server = get_server().await;
    let response = server.get("/api-docs/openapi.json").await;

    response.assert_status(StatusCode::OK);

    let body = response.json::<serde_json::Value>();
    assert_eq!(body["openapi"], "3.1.0");
    assert!(body["paths"].is_object(), "Expected paths to be an object");
    assert!(body["info"].is_object(), "Expected info to be an object");
}

#[tokio::test]
async fn test_swagger_ui_returns_200() {
    let server = get_server().await;
    let response = server.get("/swagger-ui/").await;
    response.assert_status(StatusCode::OK);
}
