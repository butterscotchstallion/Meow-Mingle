use crate::common::helpers::get_server;

use axum::http::StatusCode;
use cookie::Cookie;
use meow_mingle::cats::{CatDetailResponse, routes};

use meow_mingle::models::cat::Cat;
use meow_mingle::models::status::Status;

#[allow(dead_code)]
pub async fn get_cat_session_profile_and_verify(session_id: String) -> Cat {
    let server = get_server().await;
    let response = server
        .get(routes::CAT_SESSION_PROFILE)
        .add_cookie(Cookie::new(
            meow_mingle::models::session::SESSION_COOKIE_NAME,
            session_id,
        ))
        .await;

    response.assert_status(StatusCode::OK);

    let body = response.json::<CatDetailResponse>();
    assert_eq!(body.status, Status::Ok);
    let cat = body
        .results
        .expect("Expected a cat in the response results, but got None");
    assert!(cat.name.len() > 0);
    assert_eq!(cat.password.len(), 0);

    cat
}
