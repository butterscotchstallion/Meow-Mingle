use crate::common::helpers::get_server;
use axum::http::StatusCode;
use meow_mingle::handlers::auth::routes::AUTH_SIGN_IN;
use meow_mingle::handlers::auth::{AuthResponseWithSessionInfo, AuthSignInPayload};

#[allow(dead_code)]
pub async fn get_session_id_and_verify(name: String, password: String) -> String {
    let server = get_server().await;
    let cat_name = name.clone();
    let response = server
        .post(AUTH_SIGN_IN)
        .json(&AuthSignInPayload {
            name: name.clone(),
            password,
        })
        .await;
    response.assert_status(StatusCode::OK);

    let body = response.json::<AuthResponseWithSessionInfo>();
    let results = &body.results;

    let session_id = response
        .cookie(meow_mingle::models::session::SESSION_COOKIE_NAME)
        .value()
        .parse::<uuid::Uuid>()
        .expect("session cookie should be a valid UUID");

    assert_eq!(results.session_id.len(), 36);
    assert_eq!(
        session_id,
        results.session_id.parse::<uuid::Uuid>().unwrap()
    );
    assert_eq!(body.status, "OK");
    assert_eq!(body.message, "Sign in successful");
    assert_eq!(results.cat.name, cat_name);
    assert_eq!(results.cat.password, "");

    body.results.session_id
}
