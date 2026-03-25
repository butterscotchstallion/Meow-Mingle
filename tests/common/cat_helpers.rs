use crate::common::helpers::get_server;

use axum::http::StatusCode;
use cookie::Cookie;
use meow_mingle::cats::{CatDetailResponse, routes};

use meow_mingle::models::cat::Cat;
use meow_mingle::models::status::Status;

// Used to get cat profile by ID or from session
#[allow(dead_code)]
async fn get_cat_profile_and_verify(session_id: String, url: &str) -> Cat {
    let server = get_server().await;
    let response = server
        .get(&url)
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

#[allow(dead_code)]
pub async fn get_cat_session_profile_and_verify(session_id: String) -> Cat {
    get_cat_profile_and_verify(session_id, routes::CAT_SESSION_PROFILE).await
}

#[allow(dead_code)]
pub async fn get_cat_profile_by_id_and_verify(session_id: String, cat_id: &str) -> Cat {
    let url = routes::CAT_DETAIL.replace("{id}", &cat_id);
    get_cat_profile_and_verify(session_id, &url).await
}
