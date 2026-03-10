use axum::http::StatusCode;

mod common;
use common::helpers::get_server;
use meow_mingle::handlers::matches::routes::{MATCHES_LIST, MATCH_SUGGESTIONS};

#[tokio::test]
async fn test_matches_list_returns_200() {
    let server = get_server().await;
    let response = server.get(MATCHES_LIST).await;
    response.assert_status(StatusCode::OK);

    let body = response.json::<serde_json::Value>();
    assert_eq!(body["status"], "OK");
}

#[tokio::test]
async fn test_match_suggestions_returns_200() {
    let server = get_server().await;
    let response = server.get(MATCH_SUGGESTIONS).await;
    response.assert_status(StatusCode::OK);

    let body = response.json::<serde_json::Value>();
    assert_eq!(body["status"], "OK");
}
