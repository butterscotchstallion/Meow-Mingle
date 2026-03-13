mod common;
use common::helpers::get_server;

use crate::common::auth_helpers::get_session_id_and_verify;
use axum::http::StatusCode;
use meow_mingle::cats::CatDetailResponse;
use meow_mingle::handlers::session::routes::*;
use meow_mingle::models::status::Status;
use tracing::log::debug;

#[tokio::test]
async fn test_session_get_by_id_returns_404_for_unknown_id() {
    let server = get_server().await;
    let unknown_id = "00000000-0000-0000-0000-000000000000";
    let url = SESSION_GET_BY_ID.replace("{id}", unknown_id);

    let response = server.get(&url).await;

    response.assert_status(StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_session_get_by_id_returns_error_body_for_unknown_id() {
    let server = get_server().await;
    let unknown_id = "00000000-0000-0000-0000-000000000000";
    let url = SESSION_GET_BY_ID.replace("{id}", unknown_id);

    let response = server.get(&url).await;
    let body = response.json::<serde_json::Value>();

    assert_eq!(body["status"], "ERROR");
    assert!(body["message"].is_string());
}

#[tokio::test]
async fn test_session_get_by_id_returns_400_for_invalid_uuid() {
    let server = get_server().await;
    let url = SESSION_GET_BY_ID.replace("{id}", "not-a-uuid");

    let response = server.get(&url).await;

    response.assert_status(StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_get_cat_by_session_id() {
    let server = get_server().await;
    let cfg = meow_mingle::config::load_config();
    let name = cfg.test_users.admin_username.clone();
    let session_id =
        get_session_id_and_verify(name.clone(), cfg.test_users.admin_password.clone()).await;

    assert!(SESSION_GET_BY_ID.contains("{id}"));
    let url = SESSION_GET_BY_ID.replace("{id}", &session_id);
    let response = server.get(&url).await;
    let body = response.json::<CatDetailResponse>();

    debug!("Response body: {:?}", body);

    let cat = body
        .results
        .as_ref()
        .expect("Expected a cat in the response results, but got None");

    response.assert_status(StatusCode::OK);
    assert_eq!(body.status, Status::Ok);
    assert_eq!(cat.name, name);
}
