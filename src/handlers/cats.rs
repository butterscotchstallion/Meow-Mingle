use crate::handlers::common::ApiError;
use crate::models::cat::get_cat_by_id;
use crate::models::session::get_cat_from_session_id;
use crate::models::status::Status;
use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum_cookie::CookieManager;
use sqlx::PgPool;
use uuid::Uuid;

pub mod routes {
    pub const CAT_DETAIL: &str = "/api/v1/cats/{id}";
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
