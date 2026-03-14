use axum::http::StatusCode;
use cookie::Cookie;

mod common;
use crate::common::auth_helpers::sign_up_and_get_session_id;
use common::helpers::get_server;
use meow_mingle::handlers::matches::routes::{MATCH_SUGGESTIONS, MATCHES_LIST};
use meow_mingle::handlers::matches::{MatchSuggestionsResponse, MatchesListResponse};
use meow_mingle::models::status::Status;

#[tokio::test]
async fn test_matches_list_returns_200() {
    let server = get_server().await;
    let session_id: String = sign_up_and_get_session_id().await;
    let response = server
        .get(MATCHES_LIST)
        .add_cookie(Cookie::new(
            meow_mingle::models::session::SESSION_COOKIE_NAME,
            session_id,
        ))
        .await;

    response.assert_status(StatusCode::OK);

    let body = response.json::<MatchesListResponse>();

    assert_eq!(body.status, Status::Ok);
}

#[tokio::test]
async fn test_match_suggestions_returns_200() {
    let server = get_server().await;
    let session_id: String = sign_up_and_get_session_id().await;
    let response = server
        .get(MATCH_SUGGESTIONS)
        .add_cookie(Cookie::new(
            meow_mingle::models::session::SESSION_COOKIE_NAME,
            session_id,
        ))
        .await;
    response.assert_status(StatusCode::OK);

    let body = response.json::<MatchSuggestionsResponse>();
    assert_eq!(body.status, Status::Ok);
    assert!(body.results.len() > 0, "Match suggestions list empty?")
}

#[tokio::test]
async fn test_match_suggestions_age_filter() {
    let server = get_server().await;
    let session_id: String = sign_up_and_get_session_id().await;
    let age_lt = 8;
    let age_gt = 15;
    let url = format!("{}?lt={}&gt={}", MATCH_SUGGESTIONS, age_lt, age_gt);
    let response = server
        .get(&url)
        .add_cookie(Cookie::new(
            meow_mingle::models::session::SESSION_COOKIE_NAME,
            session_id,
        ))
        .await;
    response.assert_status(StatusCode::OK);
    let body = response.json::<MatchSuggestionsResponse>();
    assert_eq!(body.status, Status::Ok);

    for cat in body.results {
        let age = cat.age.expect("cat should have a calculated age");
        assert!(
            age >= age_gt && age <= age_lt,
            "cat age {} is not within the range [{}, {}]",
            age,
            age_gt,
            age_lt
        );
    }
}
