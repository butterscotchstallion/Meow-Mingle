use axum::http::StatusCode;
use meow_mingle::cats::{routes, CatDetailResponse, CatsListResponse};
use meow_mingle::models::status::Status;
mod common;
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
    let body = response.json::<CatsListResponse>();

    assert_eq!(body.status, "OK");
    assert!(body.results.len() > 0);
}

#[tokio::test]
async fn test_cats_get_cat_by_name() {
    let server = get_server().await;
    let cfg = meow_mingle::config::load_config();
    let url = routes::CAT_DETAIL.replace("{name}", &*cfg.test_users.admin_username.to_string());
    let response = server.get(&url).await;
    let body = response.json::<CatDetailResponse>();

    let cat = body
        .results
        .as_ref()
        .expect("Expected a cat in the response results, but got None");
    assert_eq!(body.status, Status::Ok);
    assert_eq!(cat.name, cfg.test_users.admin_username);
}
