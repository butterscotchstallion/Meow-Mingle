use crate::common::helpers::get_server;
use axum::http::StatusCode;
use meow_mingle::handlers::auth::routes::AUTH_SIGN_IN;
use meow_mingle::handlers::auth::{AuthResponseWithSessionInfo, AuthSignInPayload};
use meow_mingle::models::session::get_session_id_from_cookie;

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

    // let cookie_value = response
    //     .cookie(meow_mingle::models::session::SESSION_COOKIE_NAME)
    //     .to_string();
    // let Some((_, session_id)) = cookie_value.split_once('=') else {
    //     panic!("Failed to extract session ID from cookie");
    // };
    let cookies = response.cookies();
    let session_id = get_session_id_from_cookie(cookies);

    assert_eq!(results.session_id.len(), 36);
    assert_eq!(session_id, results.session_id.as_str());
    assert_eq!(body.status, "OK");
    assert_eq!(body.message, "Sign in successful");
    assert_eq!(results.cat.name, cat_name);
    assert_eq!(results.cat.password, "");

    body.results.session_id
}
