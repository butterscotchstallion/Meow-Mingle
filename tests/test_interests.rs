use axum::http::StatusCode;
use meow_mingle::handlers::interests::{InterestListResponse, routes};
use meow_mingle::models::status::Status;

mod common;
use common::helpers::get_server;

#[tokio::test]
async fn test_interests_list_returns_ok_status() {
    let server = get_server().await;
    let response = server.get(routes::INTERESTS_LIST).await;

    response.assert_status(StatusCode::OK);

    let body = response.json::<InterestListResponse>();
    assert_eq!(body.status, Status::Ok);
}

#[tokio::test]
async fn test_interests_list_returns_results() {
    let server = get_server().await;
    let response = server.get(routes::INTERESTS_LIST).await;

    let body = response.json::<InterestListResponse>();
    assert!(
        !body.results.is_empty(),
        "Expected at least one interest, but got an empty list"
    );
}

#[tokio::test]
async fn test_interests_list_contains_catnip() {
    let server = get_server().await;
    let response = server.get(routes::INTERESTS_LIST).await;

    let body = response.json::<InterestListResponse>();
    assert!(
        body.results.iter().any(|i| i.name == "Catnip"),
        "Expected 'Catnip' in interests list, but it was not found"
    );
}

#[tokio::test]
async fn test_interests_list_contains_all_seeded_interests() {
    let server = get_server().await;
    let response = server.get(routes::INTERESTS_LIST).await;

    let body = response.json::<InterestListResponse>();
    let seeded = [
        "Catnip",
        "Bird Watching",
        "Laser Pointer Chasing",
        "Yarn & String Play",
        "Napping in Sunbeams",
        "Cardboard Box Exploring",
        "Feather Toy Hunting",
        "Fish & Aquariums",
        "Scratching Posts",
        "Window Perching",
        "Chasing Mice",
        "Knocking Things Off Tables",
        "Midnight Zoomies",
        "Grooming & Self-Care",
        "Cozy Blanket Kneading",
        "Bug Hunting",
        "Treat Tasting",
        "Paper Bag Crinkling",
        "Tail Chasing",
        "Cuddling & Lap Sitting",
    ];

    for name in &seeded {
        assert!(
            body.results.iter().any(|i| i.name == *name),
            "Expected '{}' in interests list, but it was not found",
            name
        );
    }
}
