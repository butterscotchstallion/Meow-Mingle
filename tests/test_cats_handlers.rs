use axum::http::StatusCode;
use axum_test::multipart::{MultipartForm, Part};
use cookie::Cookie;
use meow_mingle::cats::routes;
use meow_mingle::models::status::Status;
mod common;
use crate::common::auth_helpers::{sign_up_and_get_session_and_cat_id, sign_up_and_get_session_id};
use crate::common::cat_helpers::{
    get_cat_profile_by_id_and_verify, get_cat_session_profile_and_verify,
};
use common::helpers::get_server;
use meow_mingle::handlers::common::GenericResponse;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

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
    let avatar_filename = String::from("avatar.jpg");
    let birth_date = OffsetDateTime::now_utc();
    let birth_date_str = birth_date
        .format(&Rfc3339)
        .expect("Failed to format birth_date");

    // Build a small 1x1 white PNG for the photo upload
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
        .add_text("avatar_filename", avatar_filename.clone())
        .add_text("birth_date", birth_date_str)
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
    assert_eq!(
        cat.avatar_filename.as_deref(),
        Some(avatar_filename.as_str())
    );
    assert!(
        !cat.photos.is_empty(),
        "Expected at least one uploaded photo"
    );
}
