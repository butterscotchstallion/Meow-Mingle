use axum::http::StatusCode;
use meow_mingle::handlers::breeds::{BreedsListResponse, routes};
use meow_mingle::models::status::Status;

mod common;
use common::helpers::get_server;

#[tokio::test]
async fn test_breeds_list_returns_200() {
    let server = get_server().await;
    let response = server.get(routes::BREEDS_LIST).await;

    response.assert_status(StatusCode::OK);

    let body = response.json::<BreedsListResponse>();
    assert_eq!(body.status, Status::Ok);
}

#[tokio::test]
async fn test_breeds_list_returns_results() {
    let server = get_server().await;
    let response = server.get(routes::BREEDS_LIST).await;

    let body = response.json::<BreedsListResponse>();

    assert!(
        !body.results.is_empty(),
        "Expected at least one breed, but got an empty list"
    );
}

#[tokio::test]
async fn test_breeds_list_contains_maine_coon() {
    let server = get_server().await;
    let response = server.get(routes::BREEDS_LIST).await;

    let body = response.json::<BreedsListResponse>();

    assert!(
        body.results.iter().any(|b| b.name == "Maine Coon"),
        "Expected 'Maine Coon' in breeds list, but it was not found"
    );
}

#[tokio::test]
async fn test_breeds_list_does_not_require_auth() {
    // Hit the endpoint with no cookie at all and confirm it still returns 200
    let server = get_server().await;
    let response = server.get(routes::BREEDS_LIST).await;

    response.assert_status(StatusCode::OK);
}

#[tokio::test]
async fn test_breeds_have_valid_ids() {
    let server = get_server().await;
    let response = server.get(routes::BREEDS_LIST).await;

    let body = response.json::<BreedsListResponse>();

    for breed in &body.results {
        assert_eq!(
            breed.id.to_string().len(),
            36,
            "Breed '{}' has an invalid UUID: {}",
            breed.name,
            breed.id
        );
    }
}
