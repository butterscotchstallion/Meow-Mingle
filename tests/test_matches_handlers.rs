use axum::http::StatusCode;

mod common;
use crate::common::auth_helpers::get_session_id_and_verify;
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
    let cfg = meow_mingle::config::load_config();
    let session_id: String = get_session_id_and_verify(
        cfg.test_users.admin_username.clone(),
        cfg.test_users.admin_password,
    )
    .await;
    assert_eq!(session_id.len(), 36);

    let response = server.get(MATCH_SUGGESTIONS).await;
    response.assert_status(StatusCode::OK);

    let body = response.json::<serde_json::Value>();
    assert_eq!(body["status"], "OK");
}
