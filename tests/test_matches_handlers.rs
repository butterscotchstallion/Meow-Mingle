use axum::http::StatusCode;
use cookie::Cookie;

mod common;
use crate::common::auth_helpers::{sign_up_and_get_session_and_cat_id, sign_up_and_get_session_id};
use common::helpers::get_server;
use meow_mingle::get_db_pool;
use meow_mingle::handlers::matches::routes::{
    MATCH_ADD, MATCH_MARK_SEEN, MATCH_SUGGESTIONS, MATCHES_LIST,
};
use meow_mingle::handlers::matches::{
    MatchAddRequest, MatchAddedResponse, MatchStatus, MatchSuggestionsResponse, MatchesListResponse,
};
use meow_mingle::models::status::Status;
use serde_json::json;
use tracing::debug;
use uuid::Uuid;

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

    response.assert_status(StatusCode::OK);
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

#[tokio::test]
async fn test_match_suggestions_interest_filter() {
    let server = get_server().await;
    let pool = get_db_pool().await.expect("Failed to get DB pool");

    // Sign up a new cat and grab both its session ID and cat ID
    let (session_id, cat_id) = sign_up_and_get_session_and_cat_id().await;

    // Pick a well-known interest from the seed data: 'Catnip'
    let interest_id: Uuid =
        sqlx::query_scalar!("SELECT id FROM interests WHERE name = 'Catnip' LIMIT 1")
            .fetch_one(&pool)
            .await
            .expect("Catnip interest should exist in seed data");

    // Assign that interest to the signed-in cat so the filter has something to match against
    sqlx::query!(
        "INSERT INTO cats_interests (cat_id, interest_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
        cat_id,
        interest_id
    )
    .execute(&pool)
    .await
    .expect("Failed to insert cat interest");

    // Request suggestions filtered to only cats that share the Catnip interest
    let url = format!("{}?interest_ids={}", MATCH_SUGGESTIONS, interest_id);
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
        assert!(
            cat.interests.iter().any(|i| i.id == interest_id),
            "cat '{}' does not have the filtered interest ({})",
            cat.name,
            interest_id
        );
    }
}

/**
 * Two-way matching
 * 1. Create cat 1
 * 2. Create cat 2
 * 3. Add match with cat-1 as initiator, cat-2 as target
 * 4. Get matches using cat-2 session and confirm match is pending with cat-1 as initiator
 * 5. Update match status
 * 6. Get matches for cat-1 and should have cat-2 match
 */
#[tokio::test]
pub async fn test_two_way_matching() {
    let server = get_server().await;
    let (salt_session_id, salt_cat_id) = sign_up_and_get_session_and_cat_id().await;
    let (pepper_session_id, pepper_cat_id) = sign_up_and_get_session_and_cat_id().await;

    // Add match with Salt as initiator
    let response = server
        .post(MATCH_ADD)
        .add_cookie(Cookie::new(
            meow_mingle::models::session::SESSION_COOKIE_NAME,
            &salt_session_id,
        ))
        .json(&MatchAddRequest {
            initiator_id: salt_cat_id,
            target_id: pepper_cat_id,
            status: meow_mingle::handlers::matches::MatchStatus::Pending,
            seen: Some(false),
        })
        .await;

    response.assert_status(StatusCode::CREATED);

    let body = response.json::<MatchAddedResponse>();
    assert_eq!(body.status, Status::Ok);

    // Get matches for Pepper: should contain salt
    let response = server
        .get(MATCHES_LIST)
        .add_cookie(Cookie::new(
            meow_mingle::models::session::SESSION_COOKIE_NAME,
            pepper_session_id.clone(),
        ))
        .await;
    let body = response.json::<MatchesListResponse>();
    assert_eq!(body.status, Status::Ok);
    assert_eq!(body.results.len(), 1);

    debug!("Salt: {}", salt_cat_id);
    debug!("Pepper: {}", pepper_cat_id);
    debug!("{:?}", body.results);

    let mut salt_found_in_pepper_matches = false;
    for cat_match in body.results {
        // Salt would be the initiator, and the target is pepper
        if cat_match.initiator_id == salt_cat_id && cat_match.target_id == pepper_cat_id {
            salt_found_in_pepper_matches = true;
            break;
        }
    }

    assert!(
        salt_found_in_pepper_matches,
        "Salt not found in Pepper matches"
    );

    // Update match status: set status to accepted for match with Pepper as target and
    // Salt as initiator
    let response = server
        .post(MATCHES_LIST)
        .json(&json!(MatchAddRequest {
            initiator_id: salt_cat_id,
            target_id: pepper_cat_id,
            status: meow_mingle::handlers::matches::MatchStatus::Accepted,
            seen: None,
        }))
        .add_cookie(Cookie::new(
            meow_mingle::models::session::SESSION_COOKIE_NAME,
            pepper_session_id,
        ))
        .await;
    response.assert_status(StatusCode::CREATED);
    let body = response.json::<MatchAddedResponse>();
    assert_eq!(body.status, Status::Ok);

    // Get matches for Salt which should list Pepper as a successful match
    let response = server
        .get(MATCHES_LIST)
        .add_cookie(Cookie::new(
            meow_mingle::models::session::SESSION_COOKIE_NAME,
            &salt_session_id,
        ))
        .await;
    response.assert_status(StatusCode::OK);
    let body = response.json::<MatchesListResponse>();
    assert_eq!(body.status, Status::Ok);
    assert_eq!(body.results.len(), 1);

    let mut pepper_found_in_pepper_matches = false;
    for cat_match in body.results {
        if cat_match.initiator_id == salt_cat_id && cat_match.target_id == pepper_cat_id {
            pepper_found_in_pepper_matches = true;
            break;
        }
    }

    assert!(
        pepper_found_in_pepper_matches,
        "Pepper not found in Salt's matches"
    );
}

// ── mark-seen tests ──────────────────────────────────────────────────────────

#[tokio::test]
async fn test_mark_seen_requires_auth() {
    let server = get_server().await;

    // Create a real match so the match ID exists in the DB
    let (session_id, cat_id) = sign_up_and_get_session_and_cat_id().await;
    let (_, target_id) = sign_up_and_get_session_and_cat_id().await;

    let create_resp = server
        .post(MATCH_ADD)
        .add_cookie(Cookie::new(
            meow_mingle::models::session::SESSION_COOKIE_NAME,
            &session_id,
        ))
        .json(&MatchAddRequest {
            initiator_id: cat_id,
            target_id,
            status: MatchStatus::Pending,
            seen: Some(false),
        })
        .await;
    create_resp.assert_status(StatusCode::CREATED);

    // Fetch the match to get its real ID
    let list_resp = server
        .get(MATCHES_LIST)
        .add_cookie(Cookie::new(
            meow_mingle::models::session::SESSION_COOKIE_NAME,
            &session_id,
        ))
        .await;
    list_resp.assert_status(StatusCode::OK);
    let list_body = list_resp.json::<MatchesListResponse>();
    let match_id = list_body.results[0].id;

    let url = MATCH_MARK_SEEN.replace("{match_id}", &match_id.to_string());

    // No cookie → should get 401
    let response = server.put(&url).await;
    response.assert_status(StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_mark_seen_returns_404_for_nonexistent_match() {
    let server = get_server().await;
    let (session_id, _cat_id) = sign_up_and_get_session_and_cat_id().await;

    let nonexistent_id = Uuid::new_v4();
    let url = MATCH_MARK_SEEN.replace("{match_id}", &nonexistent_id.to_string());

    let response = server
        .put(&url)
        .add_cookie(Cookie::new(
            meow_mingle::models::session::SESSION_COOKIE_NAME,
            &session_id,
        ))
        .await;

    response.assert_status(StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_mark_seen_returns_404_when_cat_is_not_initiator() {
    let server = get_server().await;

    // Cat A creates the match (A = initiator, B = target)
    let (session_a, cat_a_id) = sign_up_and_get_session_and_cat_id().await;
    let (session_b, cat_b_id) = sign_up_and_get_session_and_cat_id().await;

    let create_resp = server
        .post(MATCH_ADD)
        .add_cookie(Cookie::new(
            meow_mingle::models::session::SESSION_COOKIE_NAME,
            &session_a,
        ))
        .json(&MatchAddRequest {
            initiator_id: cat_a_id,
            target_id: cat_b_id,
            status: MatchStatus::Pending,
            seen: Some(false),
        })
        .await;
    create_resp.assert_status(StatusCode::CREATED);

    // A fetches their match list to retrieve the match ID
    let list_resp = server
        .get(MATCHES_LIST)
        .add_cookie(Cookie::new(
            meow_mingle::models::session::SESSION_COOKIE_NAME,
            &session_a,
        ))
        .await;
    list_resp.assert_status(StatusCode::OK);
    let list_body = list_resp.json::<MatchesListResponse>();
    let match_id = list_body.results[0].id;

    let url = MATCH_MARK_SEEN.replace("{match_id}", &match_id.to_string());

    // B (the target, not initiator) tries to mark it as seen → 404
    let response = server
        .put(&url)
        .add_cookie(Cookie::new(
            meow_mingle::models::session::SESSION_COOKIE_NAME,
            &session_b,
        ))
        .await;

    response.assert_status(StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_mark_seen_sets_seen_to_true() {
    let server = get_server().await;

    // Cat A creates a match with seen = false
    let (session_a, cat_a_id) = sign_up_and_get_session_and_cat_id().await;
    let (_, cat_b_id) = sign_up_and_get_session_and_cat_id().await;

    let create_resp = server
        .post(MATCH_ADD)
        .add_cookie(Cookie::new(
            meow_mingle::models::session::SESSION_COOKIE_NAME,
            &session_a,
        ))
        .json(&MatchAddRequest {
            initiator_id: cat_a_id,
            target_id: cat_b_id,
            status: MatchStatus::Pending,
            seen: Some(false),
        })
        .await;
    create_resp.assert_status(StatusCode::CREATED);

    // Fetch the match while it's still unseen (default filter is seen=false)
    let list_resp = server
        .get(MATCHES_LIST)
        .add_cookie(Cookie::new(
            meow_mingle::models::session::SESSION_COOKIE_NAME,
            &session_a,
        ))
        .await;
    list_resp.assert_status(StatusCode::OK);
    let list_body = list_resp.json::<MatchesListResponse>();
    assert_eq!(
        list_body.results.len(),
        1,
        "should have one unseen match before mark-seen"
    );
    let match_id = list_body.results[0].id;

    let url = MATCH_MARK_SEEN.replace("{match_id}", &match_id.to_string());

    // A marks the match as seen
    let mark_resp = server
        .put(&url)
        .add_cookie(Cookie::new(
            meow_mingle::models::session::SESSION_COOKIE_NAME,
            &session_a,
        ))
        .await;
    mark_resp.assert_status(StatusCode::OK);
    let mark_body = mark_resp.json::<MatchAddedResponse>();
    assert_eq!(mark_body.status, meow_mingle::models::status::Status::Ok);
    assert_eq!(mark_body.message, "Match marked as seen");

    // GET with seen=true → match should now appear
    let seen_url = format!("{}?seen=true", MATCHES_LIST);
    let seen_resp = server
        .get(&seen_url)
        .add_cookie(Cookie::new(
            meow_mingle::models::session::SESSION_COOKIE_NAME,
            &session_a,
        ))
        .await;
    seen_resp.assert_status(StatusCode::OK);
    let seen_body = seen_resp.json::<MatchesListResponse>();
    assert!(
        seen_body.results.iter().any(|m| m.id == match_id),
        "match should appear in seen=true list after mark-seen"
    );

    // GET with seen=false (default) → match should no longer appear
    let unseen_resp = server
        .get(MATCHES_LIST)
        .add_cookie(Cookie::new(
            meow_mingle::models::session::SESSION_COOKIE_NAME,
            &session_a,
        ))
        .await;
    unseen_resp.assert_status(StatusCode::OK);
    let unseen_body = unseen_resp.json::<MatchesListResponse>();
    assert!(
        !unseen_body.results.iter().any(|m| m.id == match_id),
        "match should NOT appear in seen=false list after mark-seen"
    );
}
