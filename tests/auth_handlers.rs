mod helpers;

use axum::http::StatusCode;
use meow_mingle::config;
use meow_mingle::handlers::auth::routes::AUTH_LOGIN;
use meow_mingle::handlers::auth::{AuthLoginPayload, AuthResponseWithSessionInfo};
use serde_json::json;

#[tokio::test]
async fn test_login_invalid_credentials_returns_401() {
    let server = helpers::get_server().await;
    let payload = AuthLoginPayload {
        username: "nonexistent_cat".to_string(),
        password: "wrong password".to_string(),
    };
    let response = server.post(AUTH_LOGIN).json(&payload).await;

    response.assert_status(StatusCode::UNAUTHORIZED);
    let body = response.json::<serde_json::Value>();
    assert_eq!(body["message"], "Invalid username or password");
}

#[tokio::test]
async fn test_login_missing_fields_returns_422() {
    let server = helpers::get_server().await;
    let response = server
        .post(AUTH_LOGIN)
        .json(&json!({
            "foo": "baz"
        }))
        .await;

    response.assert_status(StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn test_login_with_config_user() {
    let server = helpers::get_server().await;
    let cfg = config::load_config();
    let response = server
        .post(AUTH_LOGIN)
        .json(&AuthLoginPayload {
            username: cfg.test_users.admin_username.clone(),
            password: cfg.test_users.admin_password,
        })
        .await;
    response.assert_status(StatusCode::OK);

    let body = response.json::<AuthResponseWithSessionInfo>();

    assert_eq!(body.status, "OK");
    assert_eq!(body.message, "Login successful");
    assert_eq!(body.results.cat.username, cfg.test_users.admin_username);
    assert_eq!(body.results.session_id.len(), 36);
}
