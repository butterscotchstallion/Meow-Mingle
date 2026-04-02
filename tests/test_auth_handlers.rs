use axum::http::StatusCode;
use cookie::Cookie;
use meow_mingle::config;
use meow_mingle::handlers::auth::routes::*;
use meow_mingle::handlers::auth::{AuthSignInPayload, AuthSignUpResponse};
use serde_json::json;
use uuid::Uuid;

#[allow(dead_code)]
mod common;
use crate::common::auth_helpers::sign_up_and_get_session_id;
use common::auth_helpers::get_session_id_and_verify;
use common::helpers::get_server;
use meow_mingle::models::session::SESSION_COOKIE_NAME;
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

// ── Impersonate endpoint ───────────────────────────────────────────────────

#[tokio::test]
async fn test_impersonate_requires_auth() {
    // No session cookie → 401
    let server = get_server().await;
    let cfg = config::load_config();
    let target_id = cfg.test_users.unprivileged_id;

    let url = AUTH_IMPERSONATE.replace("{cat_id}", &target_id);
    let response = server.post(&url).await;

    response.assert_status(StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_impersonate_requires_admin_role() {
    // Signed in as a non-admin cat → 403
    let server = get_server().await;
    let cfg = config::load_config();
    let session_id = sign_up_and_get_session_id().await;
    let target_id = cfg.test_users.unprivileged_id;

    let url = AUTH_IMPERSONATE.replace("{cat_id}", &target_id);
    let response = server
        .post(&url)
        .add_cookie(Cookie::new(SESSION_COOKIE_NAME, session_id))
        .await;

    response.assert_status(StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_impersonate_returns_404_for_nonexistent_cat() {
    // Admin user impersonating a UUID that doesn't exist → 404
    let server = get_server().await;
    let cfg = config::load_config();
    let admin_session = get_session_id_and_verify(
        cfg.test_users.admin_username.clone(),
        cfg.test_users.admin_password.clone(),
    )
    .await;

    let nonexistent_id = Uuid::new_v4();
    let url = AUTH_IMPERSONATE.replace("{cat_id}", &nonexistent_id.to_string());
    let response = server
        .post(&url)
        .add_cookie(Cookie::new(SESSION_COOKIE_NAME, admin_session))
        .await;

    response.assert_status(StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_impersonate_succeeds_for_admin() {
    // Admin user impersonating another cat → 200 with session cookie for target
    let server = get_server().await;
    let cfg = config::load_config();
    let admin_session = get_session_id_and_verify(
        cfg.test_users.admin_username.clone(),
        cfg.test_users.admin_password.clone(),
    )
    .await;

    let target_id = cfg.test_users.unprivileged_id.clone();
    let url = AUTH_IMPERSONATE.replace("{cat_id}", &target_id);
    let response = server
        .post(&url)
        .add_cookie(Cookie::new(SESSION_COOKIE_NAME, admin_session))
        .await;

    response.assert_status(StatusCode::OK);

    let body = response.json::<serde_json::Value>();
    assert_eq!(body["status"], "OK");
    assert!(body["results"]["session_id"].as_str().is_some());

    // The returned session_id should be a valid UUID
    let returned_session_id = body["results"]["session_id"]
        .as_str()
        .expect("session_id should be a string");
    Uuid::parse_str(returned_session_id).expect("session_id should be a valid UUID");

    // The cat in the response should be the target, not the admin
    assert_eq!(
        body["results"]["cat"]["id"].as_str().unwrap(),
        target_id.as_str()
    );
    assert_eq!(
        body["results"]["cat"]["name"].as_str().unwrap(),
        cfg.test_users.unprivileged_username.as_str()
    );

    // A session cookie should be set for the target cat
    let cookie_value = response.cookie(SESSION_COOKIE_NAME).value().to_string();
    Uuid::parse_str(&cookie_value).expect("cookie should contain a valid UUID session id");
}

#[tokio::test]
async fn test_impersonate_invalid_uuid_returns_422() {
    // Passing a non-UUID path segment → 422/400 from axum's Path extractor
    let server = get_server().await;
    let cfg = config::load_config();
    let admin_session = get_session_id_and_verify(
        cfg.test_users.admin_username.clone(),
        cfg.test_users.admin_password.clone(),
    )
    .await;

    let url = "/api/v1/auth/impersonate/not-a-uuid";
    let response = server
        .post(url)
        .add_cookie(Cookie::new(SESSION_COOKIE_NAME, admin_session))
        .await;

    // Axum rejects malformed Path params before the handler runs
    response.assert_status(StatusCode::BAD_REQUEST);
}
