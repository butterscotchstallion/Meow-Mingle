mod helpers;

use crate::helpers::get_server;
use axum::http::StatusCode;
use meow_mingle::routes;

#[tokio::test]
async fn test_session_get_by_id_returns_404_for_unknown_id() {
    let server = get_server().await;
    let unknown_id = "00000000-0000-0000-0000-000000000000";
    let url = routes::SESSION_GET_BY_ID.replace("{id}", unknown_id);

    let response = server.get(&url).await;

    response.assert_status(StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_session_get_by_id_returns_error_body_for_unknown_id() {
    let server = get_server().await;
    let unknown_id = "00000000-0000-0000-0000-000000000000";
    let url = routes::SESSION_GET_BY_ID.replace("{id}", unknown_id);

    let response = server.get(&url).await;
    let body = response.json::<serde_json::Value>();

    assert_eq!(body["status"], "ERROR");
    assert!(body["message"].is_string());
}

#[tokio::test]
async fn test_session_get_by_id_returns_400_for_invalid_uuid() {
    let server = get_server().await;
    let url = routes::SESSION_GET_BY_ID.replace("{id}", "not-a-uuid");

    let response = server.get(&url).await;

    response.assert_status(StatusCode::BAD_REQUEST);
}
