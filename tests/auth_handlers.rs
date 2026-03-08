use axum::http::StatusCode;
use meow_mingle::config;
use meow_mingle::handlers::auth::routes::AUTH_SIGN_IN;
use meow_mingle::handlers::auth::AuthSignInPayload;
use serde_json::json;

#[allow(dead_code)]
mod common;
use common::auth_helpers::get_session_id_and_verify;
use common::helpers::get_server;

#[tokio::test]
async fn test_sign_in_invalid_credentials_returns_401() {
    let server = get_server().await;
    let payload = AuthSignInPayload {
        name: "nonexistent_cat".to_string(),
        password: "wrong password".to_string(),
    };
    let response = server.post(AUTH_SIGN_IN).json(&payload).await;

    response.assert_status(StatusCode::UNAUTHORIZED);
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
