use axum::http::StatusCode;

mod common;
use common::helpers::get_server;
use meow_mingle::handlers::matches::routes::MATCHES_LIST;

#[tokio::test]
async fn test_matches_list_returns_200() {
    let server = get_server().await;
    let response = server.get(MATCHES_LIST).await;
    response.assert_status(StatusCode::OK);

    let body = response.json::<serde_json::Value>();
    assert_eq!(body["status"], "OK");
}
