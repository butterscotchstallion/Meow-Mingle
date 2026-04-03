use axum::http::StatusCode;
use axum_test::multipart::{MultipartForm, Part};
use cookie::Cookie;
use meow_mingle::cats::routes;
use meow_mingle::config;
use meow_mingle::handlers::cats::CatAutocompleteResponse;
use meow_mingle::models::session::SESSION_COOKIE_NAME;
use meow_mingle::models::status::Status;
mod common;
use crate::common::auth_helpers::{
    get_session_id_and_verify, sign_up_and_get_session_and_cat_id, sign_up_and_get_session_id,
};
use crate::common::cat_helpers::{
    get_cat_profile_by_id_and_verify, get_cat_session_profile_and_verify,
};
use common::helpers::get_server;
use meow_mingle::handlers::common::GenericResponse;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

// ── Autocomplete endpoint ─────────────────────────────────────────────────

#[tokio::test]
async fn test_autocomplete_requires_auth() {
    // No session cookie → 401
    let server = get_server().await;
    let response = server
        .get(routes::CAT_AUTOCOMPLETE)
        .add_query_param("q", "frey")
        .await;

    response.assert_status(StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_autocomplete_requires_admin_role() {
    // Signed in as a freshly-registered (non-admin) cat → 403
    let server = get_server().await;
    let session_id = sign_up_and_get_session_id().await;

    let response = server
        .get(routes::CAT_AUTOCOMPLETE)
        .add_query_param("q", "frey")
        .add_cookie(Cookie::new(SESSION_COOKIE_NAME, session_id))
        .await;

    response.assert_status(StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_autocomplete_empty_query_returns_empty_results() {
    // Admin with q="" → 200 with empty results (short-circuit path)
    let server = get_server().await;
    let cfg = config::load_config();
    let admin_session = get_session_id_and_verify(
        cfg.test_users.admin_username.clone(),
        cfg.test_users.admin_password.clone(),
    )
    .await;

    let response = server
        .get(routes::CAT_AUTOCOMPLETE)
        .add_query_param("q", "")
        .add_cookie(Cookie::new(SESSION_COOKIE_NAME, admin_session))
        .await;

    response.assert_status(StatusCode::OK);

    let body = response.json::<CatAutocompleteResponse>();
    assert_eq!(body.status, Status::Ok);
    assert!(
        body.results.is_empty(),
        "Expected empty results for an empty query, got {} result(s)",
        body.results.len()
    );
}

#[tokio::test]
async fn test_autocomplete_missing_query_param_returns_empty_results() {
    // Admin with no ?q parameter at all → defaults to "" → same empty-results path
    let server = get_server().await;
    let cfg = config::load_config();
    let admin_session = get_session_id_and_verify(
        cfg.test_users.admin_username.clone(),
        cfg.test_users.admin_password.clone(),
    )
    .await;

    let response = server
        .get(routes::CAT_AUTOCOMPLETE)
        .add_cookie(Cookie::new(SESSION_COOKIE_NAME, admin_session))
        .await;

    response.assert_status(StatusCode::OK);

    let body = response.json::<CatAutocompleteResponse>();
    assert_eq!(body.status, Status::Ok);
    assert!(
        body.results.is_empty(),
        "Expected empty results when q param is absent"
    );
}

#[tokio::test]
async fn test_autocomplete_returns_matching_cats_for_admin() {
    // Admin searches for their own name — must find at least themselves
    let server = get_server().await;
    let cfg = config::load_config();
    let admin_session = get_session_id_and_verify(
        cfg.test_users.admin_username.clone(),
        cfg.test_users.admin_password.clone(),
    )
    .await;

    // Use the first few characters of the admin username as the search term
    let query = &cfg.test_users.admin_username[..3];

    let response = server
        .get(routes::CAT_AUTOCOMPLETE)
        .add_query_param("q", query)
        .add_cookie(Cookie::new(SESSION_COOKIE_NAME, admin_session))
        .await;

    response.assert_status(StatusCode::OK);

    let body = response.json::<CatAutocompleteResponse>();
    assert_eq!(body.status, Status::Ok);
    assert!(
        !body.results.is_empty(),
        "Expected at least one result matching '{}', got none",
        query
    );

    // The admin cat must appear in the results
    let admin_id = cfg
        .test_users
        .admin_id
        .parse::<uuid::Uuid>()
        .expect("admin_id in config should be a valid UUID");
    let found = body.results.iter().any(|cat| cat.id == admin_id);
    assert!(
        found,
        "Expected admin cat '{}' (id={}) to appear in autocomplete results for query '{}', but it was not found. Results: {:?}",
        cfg.test_users.admin_username,
        admin_id,
        query,
        body.results.iter().map(|c| &c.name).collect::<Vec<_>>()
    );
}

#[tokio::test]
async fn test_autocomplete_result_cats_have_no_password() {
    // Passwords must never be returned by the API
    let server = get_server().await;
    let cfg = config::load_config();
    let admin_session = get_session_id_and_verify(
        cfg.test_users.admin_username.clone(),
        cfg.test_users.admin_password.clone(),
    )
    .await;

    let query = &cfg.test_users.admin_username[..3];

    let response = server
        .get(routes::CAT_AUTOCOMPLETE)
        .add_query_param("q", query)
        .add_cookie(Cookie::new(SESSION_COOKIE_NAME, admin_session))
        .await;

    response.assert_status(StatusCode::OK);

    // Check the raw JSON so we catch both `""` and a missing field
    let raw = response.json::<serde_json::Value>();
    for cat in raw["results"]
        .as_array()
        .expect("results should be an array")
    {
        let password = cat["password"].as_str().unwrap_or("");
        assert!(
            password.is_empty(),
            "Expected password to be empty/absent in autocomplete result, got: '{}'",
            password
        );
    }
}

#[tokio::test]
async fn test_autocomplete_no_results_for_nonexistent_name() {
    // A query guaranteed not to match any cat name
    let server = get_server().await;
    let cfg = config::load_config();
    let admin_session = get_session_id_and_verify(
        cfg.test_users.admin_username.clone(),
        cfg.test_users.admin_password.clone(),
    )
    .await;

    let response = server
        .get(routes::CAT_AUTOCOMPLETE)
        .add_query_param("q", "zzz_no_such_cat_zzz")
        .add_cookie(Cookie::new(SESSION_COOKIE_NAME, admin_session))
        .await;

    response.assert_status(StatusCode::OK);

    let body = response.json::<CatAutocompleteResponse>();
    assert_eq!(body.status, Status::Ok);
    assert!(
        body.results.is_empty(),
        "Expected no results for a nonsense query, got {} result(s)",
        body.results.len()
    );
}

// ── Existing cat profile tests ────────────────────────────────────────────

#[tokio::test]
async fn test_get_cat_detail_by_cat_id() {
    let (session_id, cat_id) = sign_up_and_get_session_and_cat_id().await;
    let cat_id = String::from(cat_id);
    get_cat_profile_by_id_and_verify(session_id, &cat_id).await;
}

#[tokio::test]
async fn test_get_cat_profile_from_session() {
    let session_id: String = sign_up_and_get_session_id().await;
    get_cat_session_profile_and_verify(session_id).await;
}

#[tokio::test]
pub async fn test_update_cat_profile() {
    let server = get_server().await;
    let session_id = sign_up_and_get_session_id().await;
    let bio = String::from("Sphinx of The Black Quartz, judge my vow");
    let birth_date = OffsetDateTime::now_utc();
    let birth_date_str = birth_date
        .format(&Rfc3339)
        .expect("Failed to format birth_date");

    // Build a small 1x1 white PNG — reused for both the avatar and photo uploads
    let png_bytes: Vec<u8> = vec![
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // PNG signature
        0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52, // IHDR chunk length + type
        0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, // 1x1
        0x08, 0x02, 0x00, 0x00, 0x00, 0x90, 0x77, 0x53, // 8-bit RGB
        0xDE, 0x00, 0x00, 0x00, 0x0C, 0x49, 0x44, 0x41, // IDAT chunk
        0x54, 0x08, 0xD7, 0x63, 0xF8, 0xFF, 0xFF, 0x3F, 0x00, 0x05, 0xFE, 0x02, 0xFE, 0xDC, 0xCC,
        0x59, 0xE7, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, // IEND chunk
        0x44, 0xAE, 0x42, 0x60, 0x82,
    ];

    let form = MultipartForm::new()
        .add_text("biography", bio.clone())
        .add_text("birth_date", birth_date_str)
        .add_part(
            "avatar",
            Part::bytes(png_bytes.clone())
                .file_name("avatar.png")
                .mime_type("image/png"),
        )
        .add_part(
            "photo",
            Part::bytes(png_bytes)
                .file_name("test.png")
                .mime_type("image/png"),
        );

    let response = server
        .put(&routes::CAT_SESSION_PROFILE)
        .multipart(form)
        .add_cookie(Cookie::new(
            meow_mingle::models::session::SESSION_COOKIE_NAME,
            session_id.clone(),
        ))
        .await;

    response.assert_status(StatusCode::OK);

    let body = response.json::<GenericResponse>();
    assert_eq!(body.status, Status::Ok);

    // Get profile and verify the updated content
    let cat = get_cat_session_profile_and_verify(session_id.clone()).await;

    assert_eq!(cat.biography.as_deref(), Some(bio.as_str()));

    let avatar = cat
        .avatar_filename
        .as_deref()
        .expect("Expected an avatar_filename, but got None");
    assert!(
        !avatar.is_empty(),
        "Expected a non-empty avatar filename after upload"
    );
    assert!(
        avatar.ends_with(".png"),
        "Expected avatar filename to end with .png, got: {}",
        avatar
    );

    assert!(
        !cat.photos.is_empty(),
        "Expected at least one uploaded photo"
    );
}
