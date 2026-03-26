use crate::handlers::common::{ApiError, GenericResponse};
use crate::models::cat::get_cat_by_id;

use crate::models::photos::delete_existing_photos;
use crate::models::session::get_cat_from_session_id;
use crate::models::status::Status;
use axum::Json;
use axum::extract::{Multipart, Path, State};
use axum::http::StatusCode;
use axum_cookie::CookieManager;
use sqlx::PgPool;
use sqlx::types::time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tracing::debug;
use uuid::Uuid;

pub mod routes {
    pub const CAT_DETAIL: &str = "/api/v1/cats/{id}";
    pub const CAT_SESSION_PROFILE: &str = "/api/v1/profile";
}

#[derive(serde::Serialize, Debug, serde::Deserialize, utoipa::ToSchema)]
pub struct CatDetailResponse {
    pub status: Status,
    pub message: Option<String>,
    pub results: Option<crate::models::cat::Cat>,
}

#[axum::debug_handler]
#[utoipa::path(
    get,
    path = routes::CAT_DETAIL,
    params(
        ("id" = String, Path, description = "Cat ID")
    ),
    responses(
        (status = 200, description = "Details for a specific cat", body = CatDetailResponse),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn cat_detail_handler(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
    cookie_manager: CookieManager,
) -> Result<(StatusCode, Json<CatDetailResponse>), ApiError> {
    match get_cat_from_session_id(&pool, cookie_manager).await {
        Ok(Some(_)) => {}
        _ => return Err(ApiError::unauthorized()),
    };

    tracing::info!("Getting cat detail for {}", id);

    let cat = get_cat_by_id(&pool, id)
        .await
        .map_err(|e| ApiError::internal(e))?;

    Ok((
        StatusCode::OK,
        Json(CatDetailResponse {
            status: Status::Ok,
            message: Some("Success".to_string()),
            results: cat,
        }),
    ))
}

#[axum::debug_handler]
#[utoipa::path(
    get,
    path = routes::CAT_SESSION_PROFILE,
    responses(
        (status = 200, description = "Details for the signed in cat", body = CatDetailResponse),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn cat_session_profile_handler(
    State(pool): State<PgPool>,
    cookie_manager: CookieManager,
) -> Result<(StatusCode, Json<CatDetailResponse>), ApiError> {
    let cat = match get_cat_from_session_id(&pool, cookie_manager).await {
        Ok(Some(cat)) => cat,
        _ => return Err(ApiError::unauthorized()),
    };

    let cat = get_cat_by_id(&pool, cat.id)
        .await
        .map_err(|e| ApiError::internal(e))?;

    Ok((
        StatusCode::OK,
        Json(CatDetailResponse {
            status: Status::Ok,
            message: Some("Success".to_string()),
            results: cat,
        }),
    ))
}

const PHOTO_UPLOAD_DIR: &str = "../ui/src/public/images/cats";

#[axum::debug_handler]
#[utoipa::path(
    put,
    path = routes::CAT_SESSION_PROFILE,
    responses(
        (status = 200, description = "Update cat profile", body = GenericResponse),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn cat_update_profile_handler(
    State(pool): State<PgPool>,
    cookie_manager: CookieManager,
    mut multipart: Multipart,
) -> Result<(StatusCode, Json<GenericResponse>), ApiError> {
    let cat = match get_cat_from_session_id(&pool, cookie_manager).await {
        Ok(Some(cat)) => cat,
        _ => return Err(ApiError::unauthorized()),
    };

    let mut biography: Option<String> = None;
    let mut avatar_filename: Option<String> = None;
    let mut birth_date: Option<OffsetDateTime> = None;
    let mut uploaded_photo_ids: Vec<Uuid> = Vec::new();

    fs::create_dir_all(PHOTO_UPLOAD_DIR)
        .await
        .map_err(|e| ApiError::internal(e))?;

    // Delete existing photos regardless of how many new ones they upload
    let photos_deleted = delete_existing_photos(&pool, cat.id)
        .await
        .map_err(|e| ApiError::internal(e))?;
    debug!(
        "Deleted {} existing photos for cat {}",
        photos_deleted, cat.id
    );

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| ApiError::internal(e))?
    {
        let name = field.name().unwrap_or("").to_string();

        match name.as_str() {
            "biography" => {
                biography = Some(field.text().await.map_err(|e| ApiError::internal(e))?);
            }
            "avatar_filename" => {
                avatar_filename = Some(field.text().await.map_err(|e| ApiError::internal(e))?);
            }
            "birth_date" => {
                let raw = field.text().await.map_err(|e| ApiError::internal(e))?;
                let parsed =
                    OffsetDateTime::parse(&raw, &Rfc3339).map_err(|e| ApiError::internal(e))?;
                birth_date = Some(parsed);
            }
            "photo" => {
                let original_filename = field.file_name().unwrap_or("upload").to_string();
                let ext = std::path::Path::new(&original_filename)
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("bin");

                let bytes = field.bytes().await.map_err(|e| ApiError::internal(e))?;

                // Insert first with a temporary placeholder filename so the DB
                // generates the UUID for us via gen_random_uuid(). We then rename
                // the stored file to match that UUID.
                let placeholder = format!("pending_{}", Uuid::new_v4());
                let row = sqlx::query!(
                    r#"INSERT INTO photos (filename) VALUES ($1) RETURNING id"#,
                    placeholder
                )
                .fetch_one(&pool)
                .await
                .map_err(|e| ApiError::internal(e))?;

                let photo_id = row.id;
                let stored_filename = format!("{}.{}", photo_id, ext);
                let file_path = format!("{}/{}", PHOTO_UPLOAD_DIR, stored_filename);

                let mut file = fs::File::create(&file_path)
                    .await
                    .map_err(|e| ApiError::internal(e))?;
                file.write_all(&bytes)
                    .await
                    .map_err(|e| ApiError::internal(e))?;

                // Update the row with the real filename now that we have it
                sqlx::query!(
                    r#"UPDATE photos SET filename = $1 WHERE id = $2"#,
                    stored_filename,
                    photo_id
                )
                .execute(&pool)
                .await
                .map_err(|e| ApiError::internal(e))?;

                uploaded_photo_ids.push(photo_id);
            }
            _ => {}
        }
    }

    // Update the cat's core profile fields
    sqlx::query(
        r#"
        UPDATE cats
        SET
            biography = COALESCE($1, biography),
            avatar_filename = COALESCE($2, avatar_filename),
            birth_date = COALESCE($3, birth_date)
        WHERE id = $4
        "#,
    )
    .bind(biography)
    .bind(avatar_filename)
    .bind(birth_date)
    .bind(cat.id)
    .execute(&pool)
    .await
    .map_err(|e: sqlx::Error| ApiError::internal(e))?;

    // Link any uploaded photos to this cat
    for photo_id in uploaded_photo_ids {
        sqlx::query(r#"INSERT INTO cats_photos (cat_id, photo_id) VALUES ($1, $2)"#)
            .bind(cat.id)
            .bind(photo_id)
            .execute(&pool)
            .await
            .map_err(|e| ApiError::internal(e))?;
    }

    Ok((
        StatusCode::OK,
        Json(GenericResponse {
            status: Status::Ok,
            message: Some(String::from("Profile updated")),
        }),
    ))
}
