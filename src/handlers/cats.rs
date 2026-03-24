use crate::handlers::common::{ApiError, GenericResponse};
use crate::models::cat::get_cat_by_id;
use crate::models::interests::Interest;
use crate::models::photos::CatPhoto;
use crate::models::session::get_cat_from_session_id;
use crate::models::status::Status;
use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum_cookie::CookieManager;
use sqlx::PgPool;
use sqlx::types::time::OffsetDateTime;
use time::serde::rfc3339;
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

#[derive(serde::Serialize, Debug, serde::Deserialize, PartialEq, utoipa::ToSchema)]
pub struct CatProfileUpdatePayload {
    pub biography: String,
    pub interests: Vec<Interest>,
    pub photos: Vec<CatPhoto>,
    #[serde(rename = "avatarFilename")]
    pub avatar_filename: String,
    #[serde(with = "rfc3339", rename = "birthDate")]
    pub birth_date: OffsetDateTime,
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
    Json(cat_profile): Json<CatProfileUpdatePayload>,
) -> Result<(StatusCode, Json<GenericResponse>), ApiError> {
    let cat = match get_cat_from_session_id(&pool, cookie_manager).await {
        Ok(Some(cat)) => cat,
        _ => return Err(ApiError::unauthorized()),
    };

    sqlx::query(
        r#"
        UPDATE cats
        SET
        biography = $1,
        avatar_filename = $2,
        birth_date = $3
        WHERE id = $4
    "#,
    )
    .bind(cat_profile.biography)
    .bind(cat_profile.avatar_filename)
    .bind(cat_profile.birth_date)
    .bind(cat.id)
    .execute(&pool)
    .await
    .map_err(|e: sqlx::Error| ApiError::internal(e))?;

    Ok((
        StatusCode::OK,
        Json(GenericResponse {
            status: Status::Ok,
            message: Some(String::from("Profile updated")),
        }),
    ))
}
