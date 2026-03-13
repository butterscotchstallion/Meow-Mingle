use axum::http::StatusCode;
use cookie::Cookie;
use meow_mingle::cats::{CatDetailResponse, CatsListResponse, routes};
use meow_mingle::models::status::Status;
mod common;
use crate::common::auth_helpers::sign_up_and_get_session_id;
use common::helpers::get_server;

#[tokio::test]
async fn test_cats_list_returns_200() {
    let server = get_server().await;
    let response = server.get(routes::CATS_LIST).await;

    response.assert_status(StatusCode::OK);

    let body = response.json::<serde_json::Value>();
    assert_eq!(body["status"], "OK");
}

#[tokio::test]
async fn test_cats_list_response_shape() {
    let server = get_server().await;
    let response = server.get(routes::CATS_LIST).await;

    response.assert_status(StatusCode::OK);

    let body = response.json::<CatsListResponse>();

    assert_eq!(body.status, Status::Ok);
    assert!(!body.results.is_empty());
}

#[tokio::test]
async fn test_cats_get_cat_detail() {
    let server = get_server().await;
    let cfg = meow_mingle::config::load_config();
    let cat_id = cfg.test_users.unprivileged_id.to_string();
    assert_eq!(cat_id.len(), 36, "Expected a UUID, but got {}", cat_id);

    let session_id: String = sign_up_and_get_session_id().await;

    assert!(
        routes::CAT_DETAIL.contains("{id}"),
        "cat detail URL doesn't have id!"
    );

    let url = routes::CAT_DETAIL.replace("{id}", &cat_id);
    let response = server
        .get(&url)
        .add_cookie(Cookie::new(
            meow_mingle::models::session::SESSION_COOKIE_NAME,
            session_id.to_string(),
        ))
        .await;

    response.assert_status(StatusCode::OK);

    let body = response.json::<CatDetailResponse>();

    let cat = body
        .results
        .as_ref()
        .expect("Expected a cat in the response results, but got None");
    assert_eq!(body.status, Status::Ok);
    assert_eq!(cat.name, cfg.test_users.unprivileged_username);
    assert_eq!(String::from(cat.id), cat_id);
    assert!(!cat.interests.is_empty());
}
