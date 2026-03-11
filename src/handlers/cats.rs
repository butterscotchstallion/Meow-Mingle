use crate::models::cat::{get_cat_by_id, get_cats};
use crate::models::status::Status;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde_json::json;
use sqlx::PgPool;
use utoipa::ToSchema;
use uuid::Uuid;

pub mod routes {
    pub const CATS_LIST: &str = "/api/v1/cats";
    pub const CAT_DETAIL: &str = "/api/v1/cats/{id}";
}

#[derive(serde::Serialize, Debug, serde::Deserialize, ToSchema)]
pub struct CatsListResponse {
    pub status: String,
    pub results: Vec<crate::models::cat::Cat>,
}

#[derive(serde::Serialize, Debug, serde::Deserialize, utoipa::ToSchema)]
pub struct CatDetailResponse {
    pub status: Status,
    pub message: Option<String>,
    pub results: Option<crate::models::cat::Cat>,
}

#[utoipa::path(
    get,
    path = routes::CATS_LIST,
    responses(
        (status = 200, description = "List of cats", body = CatsListResponse),
        (status = 500, description = "Internal server error")
    )
)]
#[axum::debug_handler]
pub async fn cats_list_handler(
    State(pool): State<PgPool>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let cats = get_cats(&pool).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "status": Status::Error,
                "message": e.to_string()
            })),
        )
    })?;
    Ok((
        StatusCode::OK,
        Json(json!({
            "status": Status::Ok,
            "results": cats
        })),
    ))
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
        (status = 500, description = "Internal server error")
    )
)]
pub async fn cat_detail_handler(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, Json<CatDetailResponse>)> {
    tracing::info!("Getting cat detail for {}", id);
    let cat = get_cat_by_id(&pool, id).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CatDetailResponse {
                status: Status::Error,
                message: Some(e.to_string()),
                results: None,
            }),
        )
    })?;

    Ok((
        StatusCode::OK,
        Json(CatDetailResponse {
            status: Status::Ok,
            message: Option::from(String::from("Success")),
            results: cat,
        }),
    ))
}
