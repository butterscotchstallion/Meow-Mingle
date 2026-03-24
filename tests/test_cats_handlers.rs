use axum::http::StatusCode;
use cookie::Cookie;
use meow_mingle::cats::{CatDetailResponse, routes};
use meow_mingle::models::photos::CatPhoto;
use meow_mingle::models::status::Status;
mod common;
use crate::common::auth_helpers::sign_up_and_get_session_id;
use crate::common::cat_helpers::get_cat_session_profile_and_verify;
use common::helpers::get_server;
use meow_mingle::handlers::cats::CatProfileUpdatePayload;
use meow_mingle::handlers::common::GenericResponse;
use meow_mingle::models::interests::Interest;
use time::OffsetDateTime;

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

#[tokio::test]
pub async fn test_update_cat_profile() {
    let server = get_server().await;
    let session_id = sign_up_and_get_session_id().await;
    let bio = String::from("Sphinx of The Black Quartz, judge my vow");
    let interests: Vec<Interest> = Vec::new();
    let photos: Vec<CatPhoto> = Vec::new();
    let avatar_filename = String::from("/path/to/some/avatar.jpg");
    let birth_date = OffsetDateTime::now_utc();
    let response = server
        .put(&routes::CAT_SESSION_PROFILE)
        .json(&CatProfileUpdatePayload {
            biography: bio.clone(),
            interests: interests.clone(),
            photos: photos.clone(),
            avatar_filename: avatar_filename.clone(),
            birth_date,
        })
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
    assert_eq!(cat.interests, interests);
    assert_eq!(cat.photos, photos);
    assert_eq!(
        cat.avatar_filename.as_deref(),
        Some(avatar_filename.as_str())
    );
}
