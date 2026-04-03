use crate::AppState;
use crate::handlers::common::{ApiError, GenericResponse};
use crate::models::cat::get_cat_by_id;
use serde_json;

use crate::models::photos::delete_removed_photos;
use crate::models::session::get_cat_from_session_id;
use crate::models::status::Status;
use axum::Json;
use axum::extract::{Multipart, Path, Query, State};
use axum::http::StatusCode;
use axum_cookie::CookieManager;
use sqlx::types::time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tracing::debug;
use uuid::Uuid;

pub mod routes {
    pub const CAT_DETAIL: &str = "/api/v1/cats/{id}";
    pub const CAT_SESSION_PROFILE: &str = "/api/v1/profile";
    pub const CAT_AUTOCOMPLETE: &str = "/api/v2/cats/autocomplete";
}

#[derive(serde::Deserialize, Default)]
#[serde(default)]
pub struct CatAutocompleteQuery {
    pub q: String,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, utoipa::ToSchema)]
pub struct CatAutocompleteResponse {
    pub status: crate::models::status::Status,
    pub results: Vec<crate::models::cat::Cat>,
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
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    cookie_manager: CookieManager,
) -> Result<(StatusCode, Json<CatDetailResponse>), ApiError> {
    match get_cat_from_session_id(&state.pool, cookie_manager).await {
        Ok(Some(_)) => {}
        _ => return Err(ApiError::unauthorized()),
    };

    tracing::info!("Getting cat detail for {}", id);

    let cat = get_cat_by_id(&state.pool, id)
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
pub async fn cat_autocomplete_handler(
    State(state): State<AppState>,
    cookie_manager: CookieManager,
    Query(params): Query<CatAutocompleteQuery>,
) -> Result<(StatusCode, Json<CatAutocompleteResponse>), ApiError> {
    use crate::models::cat::Cat;
    use crate::models::cat::CatRow;

    let caller = match get_cat_from_session_id(&state.pool, cookie_manager).await {
        Ok(Some(cat)) => cat,
        _ => return Err(ApiError::unauthorized()),
    };

    let is_admin = crate::models::rbac::cat_has_role(&state.pool, caller.id, "cat-admin").await?;

    if !is_admin {
        return Err(ApiError::forbidden());
    }

    if params.q.is_empty() {
        return Ok((
            StatusCode::OK,
            Json(CatAutocompleteResponse {
                status: crate::models::status::Status::Ok,
                results: vec![],
            }),
        ));
    }

    let rows = sqlx::query_as!(
        CatRow,
        r#"
        SELECT c.id, c.name, c.password, c.created_at, c.updated_at,
               c.last_seen, c.active, c.avatar_filename, c.biography,
               c.birth_date, cat_breeds.id AS breed_id,
               cat_breeds.name AS breed_name,
               DATE_PART('year', AGE(c.birth_date))::int AS age
        FROM cats c
        JOIN cat_breeds ON c.breed_id = cat_breeds.id
        WHERE c.name ILIKE $1
        ORDER BY c.name ASC
        LIMIT 20
        "#,
        format!("%{}%", params.q)
    )
    .fetch_all(&state.pool)
    .await
    .map_err(ApiError::internal)?;

    let cats: Vec<Cat> = rows.into_iter().map(Cat::from).collect();

    debug!("querying autocomplete for {}", params.q);
    debug!("results: {:?}", cats);

    Ok((
        StatusCode::OK,
        Json(CatAutocompleteResponse {
            status: crate::models::status::Status::Ok,
            results: cats,
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
    State(state): State<AppState>,
    cookie_manager: CookieManager,
) -> Result<(StatusCode, Json<CatDetailResponse>), ApiError> {
    let cat = match get_cat_from_session_id(&state.pool, cookie_manager).await {
        Ok(Some(cat)) => cat,
        _ => return Err(ApiError::unauthorized()),
    };

    let cat = get_cat_by_id(&state.pool, cat.id)
        .await
        .map_err(ApiError::internal)?;

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
    put,
    path = routes::CAT_SESSION_PROFILE,
    responses(
        (status = 200, description = "Update cat profile", body = GenericResponse),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn cat_update_profile_handler(
    State(state): State<AppState>,
    cookie_manager: CookieManager,
    mut multipart: Multipart,
) -> Result<(StatusCode, Json<GenericResponse>), ApiError> {
    let cat = match get_cat_from_session_id(&state.pool, cookie_manager).await {
        Ok(Some(cat)) => cat,
        _ => return Err(ApiError::unauthorized()),
    };
    let photo_upload_dir = &state.config.photo_upload_dir;

    let mut biography: Option<String> = None;
    let mut new_avatar_filename: Option<String> = None;
    let mut birth_date: Option<OffsetDateTime> = None;
    let mut uploaded_photo_ids: Vec<Uuid> = Vec::new();
    // JSON array of {id: Uuid, order: i32} sent by the client after a drag-reorder
    let mut photo_order: Vec<(Uuid, i32)> = Vec::new();
    // IDs of existing photos the user chose to keep — anything not in this list gets deleted
    let mut kept_photo_ids: Vec<Uuid> = Vec::new();

    fs::create_dir_all(photo_upload_dir)
        .await
        .map_err(ApiError::internal)?;

    while let Some(field) = multipart.next_field().await.map_err(ApiError::internal)? {
        let name = field.name().unwrap_or("").to_string();

        match name.as_str() {
            "biography" => {
                biography = Some(field.text().await.map_err(ApiError::internal)?);
            }
            "avatar" => {
                let original_filename = field.file_name().unwrap_or("upload").to_string();
                let ext = std::path::Path::new(&original_filename)
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("bin");

                let bytes = field.bytes().await.map_err(ApiError::internal)?;

                // Delete the existing avatar file from disk if one exists
                if let Some(ref existing) = cat.avatar_filename {
                    let old_path = format!("{}/{}", photo_upload_dir, existing);
                    if fs::try_exists(&old_path).await.unwrap_or(false) {
                        fs::remove_file(&old_path)
                            .await
                            .map_err(ApiError::internal)?;
                        debug!("Deleted old avatar file: {}", old_path);
                    }
                }

                // Store as <cat_uuid>.<ext> so it's stable and easy to find
                let stored_filename = format!("{}.{}", cat.id, ext);
                let file_path = format!("{}/{}", photo_upload_dir, stored_filename);

                let mut file = fs::File::create(&file_path)
                    .await
                    .map_err(ApiError::internal)?;
                file.write_all(&bytes).await.map_err(ApiError::internal)?;

                debug!("Saved avatar for cat {} to {}", cat.id, file_path);
                new_avatar_filename = Some(stored_filename);
            }
            "birth_date" => {
                let raw = field.text().await.map_err(ApiError::internal)?;
                let parsed = OffsetDateTime::parse(&raw, &Rfc3339).map_err(ApiError::internal)?;
                birth_date = Some(parsed);
            }
            "photo" => {
                let original_filename = field.file_name().unwrap_or("upload").to_string();
                let ext = std::path::Path::new(&original_filename)
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("bin");

                let bytes = field.bytes().await.map_err(ApiError::internal)?;

                // Insert first with a temporary placeholder filename so the DB
                // generates the UUID for us via gen_random_uuid(). We then rename
                // the stored file to match that UUID.
                let placeholder = format!("pending_{}", Uuid::new_v4());
                let row = sqlx::query!(
                    r#"INSERT INTO photos (filename) VALUES ($1) RETURNING id"#,
                    placeholder
                )
                .fetch_one(&state.pool)
                .await
                .map_err(ApiError::internal)?;

                let photo_id = row.id;
                let stored_filename = format!("{}.{}", photo_id, ext);
                let file_path = format!("{}/{}", photo_upload_dir, stored_filename);

                let mut file = fs::File::create(&file_path)
                    .await
                    .map_err(ApiError::internal)?;
                file.write_all(&bytes).await.map_err(ApiError::internal)?;

                // Update the row with the real filename now that we have it
                sqlx::query!(
                    r#"UPDATE photos SET filename = $1 WHERE id = $2"#,
                    stored_filename,
                    photo_id
                )
                .execute(&state.pool)
                .await
                .map_err(ApiError::internal)?;

                uploaded_photo_ids.push(photo_id);
            }
            "kept_photo_ids" => {
                let raw = field.text().await.map_err(ApiError::internal)?;
                debug!("kept_photo_ids raw: {}", raw);
                let parsed: Vec<serde_json::Value> =
                    serde_json::from_str(&raw).map_err(ApiError::internal)?;
                for entry in &parsed {
                    if let Some(id) = entry.as_str().and_then(|s| Uuid::parse_str(s).ok()) {
                        kept_photo_ids.push(id);
                    }
                }
                debug!("kept_photo_ids parsed {} entries", kept_photo_ids.len());
            }
            "photo_order" => {
                let raw = field.text().await.map_err(ApiError::internal)?;
                debug!("photo_order raw: {}", raw);
                // Expect a JSON array: [{"id": "<uuid>", "order": 0}, ...]
                let parsed: Vec<serde_json::Value> =
                    serde_json::from_str(&raw).map_err(ApiError::internal)?;
                for entry in &parsed {
                    let id = entry["id"].as_str().and_then(|s| Uuid::parse_str(s).ok());
                    let order = entry["order"].as_i64().map(|n| n as i32);
                    debug!("photo_order entry — id: {:?}, order: {:?}", id, order);
                    if let (Some(id), Some(order)) = (id, order) {
                        photo_order.push((id, order));
                    }
                }
                debug!("photo_order parsed {} entries", photo_order.len());
            }
            _ => {}
        }
    }

    // Delete photos that are not in kept_photo_ids (includes newly uploaded ones
    // which aren't in kept_photo_ids yet — they get linked below)
    let deleted_filenames = delete_removed_photos(&state.pool, cat.id, &kept_photo_ids)
        .await
        .map_err(ApiError::internal)?;

    for filename in &deleted_filenames {
        let path = format!("{}/{}", photo_upload_dir, filename);
        if fs::try_exists(&path).await.unwrap_or(false) {
            if let Err(e) = fs::remove_file(&path).await {
                debug!("Failed to delete photo file {}: {}", path, e);
            } else {
                debug!("Deleted photo file: {}", path);
            }
        }
    }
    debug!(
        "Deleted {} removed photos for cat {}",
        deleted_filenames.len(),
        cat.id
    );

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
    .bind(new_avatar_filename)
    .bind(birth_date)
    .bind(cat.id)
    .execute(&state.pool)
    .await
    .map_err(|e: sqlx::Error| ApiError::internal(e))?;

    // Link any uploaded photos to this cat, assigning order after existing photos
    let existing_count = sqlx::query_scalar!(
        r#"SELECT COUNT(*) FROM cats_photos WHERE cat_id = $1"#,
        cat.id
    )
    .fetch_one(&state.pool)
    .await
    .map_err(ApiError::internal)?
    .unwrap_or(0) as i32;

    for (i, photo_id) in uploaded_photo_ids.iter().enumerate() {
        let order = existing_count + i as i32;
        sqlx::query(r#"INSERT INTO cats_photos (cat_id, photo_id) VALUES ($1, $2)"#)
            .bind(cat.id)
            .bind(photo_id)
            .execute(&state.pool)
            .await
            .map_err(ApiError::internal)?;

        sqlx::query!(
            r#"UPDATE photos SET "order" = $1 WHERE id = $2"#,
            order,
            photo_id
        )
        .execute(&state.pool)
        .await
        .map_err(ApiError::internal)?;
    }

    // Apply photo ordering last
    debug!("Applying order for {} photos", photo_order.len());
    for (photo_id, order) in &photo_order {
        debug!("Setting order={} for photo_id={}", order, photo_id);
        let rows = sqlx::query!(
            r#"UPDATE photos SET "order" = $1 WHERE id = $2"#,
            order,
            photo_id
        )
        .execute(&state.pool)
        .await
        .map_err(ApiError::internal)?
        .rows_affected();
        debug!("  → rows affected: {}", rows);
    }

    Ok((
        StatusCode::OK,
        Json(GenericResponse {
            status: Status::Ok,
            message: Some(String::from("Profile updated")),
        }),
    ))
}
