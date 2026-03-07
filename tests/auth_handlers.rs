mod helpers;

use axum::http::StatusCode;
use meow_mingle::config;
use meow_mingle::handlers::auth::routes::AUTH_LOGIN;
use serde_json::json;

#[tokio::test]
async fn test_login_invalid_credentials_returns_401() {
    let server = helpers::get_server().await;

    let response = server
        .post(AUTH_LOGIN)
        .json(&json!({
            "username": "nonexistent_cat",
            "password": "wrong password"
        }))
        .await;

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
            "username": "only_username"
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
        .json(&json!({
            "username": cfg.test_users.admin_username,
            "password": cfg.test_users.admin_password
        }))
        .await;

    response.assert_status(StatusCode::OK);
}
