use axum::http::StatusCode;
use meow_mingle::config;
use meow_mingle::handlers::auth::routes::*;
use meow_mingle::handlers::auth::{AuthSignInPayload, AuthSignUpResponse};
use serde_json::json;

#[allow(dead_code)]
mod common;
use crate::common::auth_helpers::sign_up_and_get_session_id;
use common::auth_helpers::get_session_id_and_verify;
use common::helpers::get_server;
use meow_mingle::models::status::Status;

#[tokio::test]
async fn test_sign_in_invalid_credentials_returns_200() {
    let server = get_server().await;
    let payload = AuthSignInPayload {
        name: "nonexistent_cat".to_string(),
        password: "wrong password".to_string(),
    };
    let response = server.post(AUTH_SIGN_IN).json(&payload).await;

    response.assert_status(StatusCode::OK);
    let body = response.json::<serde_json::Value>();
    assert_eq!(body["message"], "Invalid username or password");
}

#[tokio::test]
async fn test_sign_in_missing_fields_returns_422() {
    let server = get_server().await;
    let response = server
        .post(AUTH_SIGN_IN)
        .json(&json!({
            "foo": "baz"
        }))
        .await;

    response.assert_status(StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn test_sign_in_with_config_user() {
    let cfg = config::load_config();
    get_session_id_and_verify(
        cfg.test_users.admin_username.clone(),
        cfg.test_users.admin_password,
    )
    .await;
}

#[tokio::test]
async fn test_sign_up() {
    sign_up_and_get_session_id().await;
}

#[tokio::test]
async fn test_sign_in_wrong_password_returns_200() {
    let cfg = config::load_config();
    let server = get_server().await;
    let payload = AuthSignInPayload {
        name: cfg.test_users.admin_username.clone(),
        password: "definitively_wrong_password".to_string(),
    };
    let response = server.post(AUTH_SIGN_IN).json(&payload).await;

    response.assert_status(StatusCode::OK);
    let body = response.json::<AuthSignUpResponse>();
    assert_eq!(body.message, "Invalid username or password");
}

#[tokio::test]
async fn test_sign_in_empty_name_returns_200() {
    let server = get_server().await;
    let payload = AuthSignInPayload {
        name: "".to_string(),
        password: "somepassword".to_string(),
    };
    let response = server.post(AUTH_SIGN_IN).json(&payload).await;

    response.assert_status(StatusCode::OK);
    let body = response.json::<AuthSignUpResponse>();
    assert_eq!(body.message, "Invalid username or password");
}

#[tokio::test]
async fn test_sign_in_empty_password_returns_200() {
    let cfg = config::load_config();
    let server = get_server().await;
    let payload = AuthSignInPayload {
        name: cfg.test_users.admin_username.clone(),
        password: "".to_string(),
    };
    let response = server.post(AUTH_SIGN_IN).json(&payload).await;

    response.assert_status(StatusCode::OK);
    let body = response.json::<AuthSignUpResponse>();
    assert_eq!(body.message, "Invalid username or password");
}

#[tokio::test]
async fn test_sign_in_response_has_error_status_on_failure() {
    let server = get_server().await;
    let payload = AuthSignInPayload {
        name: "nonexistent_cat".to_string(),
        password: "wrong password".to_string(),
    };
    let response = server.post(AUTH_SIGN_IN).json(&payload).await;

    let body = response.json::<AuthSignUpResponse>();
    assert_eq!(body.status, Status::Error);
}
