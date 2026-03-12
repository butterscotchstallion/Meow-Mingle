use axum::http::StatusCode;
use meow_mingle::config;
use meow_mingle::handlers::auth::routes::*;
use meow_mingle::handlers::auth::{AuthSignInPayload, AuthSignUpPayload, AuthSignUpResponse};
use serde_json::json;
use uuid::Uuid;

#[allow(dead_code)]
mod common;
use common::auth_helpers::get_session_id_and_verify;
use common::helpers::get_server;
use meow_mingle::models::cat::NewCat;
use meow_mingle::models::status::Status;

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

#[tokio::test]
async fn test_sign_up() {
    let server = get_server().await;
    let name = Uuid::new_v4().to_string();
    let password = name.clone();
    let maine_coon_breed_id = "910ee31d-1fb6-428c-8b84-418cb8e55f20";
    let age = Option::from(3);
    let cat = NewCat {
        name: name.clone(),
        password: password.clone(),
        age,
        breed_id: maine_coon_breed_id.parse().unwrap(),
    };
    let response = server
        .post(AUTH_SIGN_UP)
        .json(&AuthSignUpPayload { cat })
        .await;

    response.assert_status(StatusCode::CREATED);

    let body = response.json::<AuthSignUpResponse>();
    let results = body.results.as_ref().expect("results should be present");

    assert_eq!(body.status, Status::Ok);
    assert_eq!(results.cat.name, name);
    assert_eq!(results.cat.age, age);
    assert_eq!(
        results.cat.breed_id.unwrap().to_string(),
        maine_coon_breed_id
    );
    assert_eq!(results.cat.password, "");
}

#[tokio::test]
async fn test_session_id_cookie_check_with_no_cookie() {}
