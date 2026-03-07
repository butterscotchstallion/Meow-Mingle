use crate::helpers::get_server;
use axum::http::StatusCode;
use meow_mingle::routes;
mod helpers;

#[tokio::test]
async fn test_cats_list_returns_200() {
    let server = get_server().await;
    let response = server.get(routes::CATS_LIST).await;
    response.assert_status(StatusCode::OK);

    let body = response.json::<serde_json::Value>();
    assert_eq!(body["status"], "OK");
}

#[tokio::test]
async fn test_cats_list_response_shape() {
    let server = get_server().await;
    let response = server.get(routes::CATS_LIST).await;
    let body = response.json::<serde_json::Value>();

    assert_eq!(body["status"], "OK");
    assert!(body["results"].is_array());
}
