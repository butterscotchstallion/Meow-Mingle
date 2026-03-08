use crate::common::helpers::get_server;
use axum::http::StatusCode;
use meow_mingle::handlers::auth::routes::AUTH_SIGN_IN;
use meow_mingle::handlers::auth::{AuthLoginPayload, AuthResponseWithSessionInfo};

#[allow(dead_code)]
pub async fn get_session_id_and_verify(name: String, password: String) -> String {
    let server = get_server().await;
    let cat_name = name.clone();
    let response = server
        .post(AUTH_SIGN_IN)
        .json(&AuthLoginPayload {
            name: name.clone(),
            password,
        })
        .await;
    response.assert_status(StatusCode::OK);

    let body = response.json::<AuthResponseWithSessionInfo>();

    assert_eq!(body.status, "OK");
    assert_eq!(body.message, "Sign in successful");
    assert_eq!(body.results.cat.name, cat_name);
    assert_eq!(body.results.cat.password, "");
    assert_eq!(body.results.session_id.len(), 36);

    body.results.session_id
}
