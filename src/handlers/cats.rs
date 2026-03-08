use crate::models::cat::{get_cat_by_name, get_cats};
use crate::models::status::Status;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde_json::json;
use sqlx::PgPool;

pub mod routes {
    pub const CATS_LIST: &str = "/cats";
    pub const CAT_DETAIL: &str = "/cats/{name}";
}

#[derive(serde::Serialize, Debug, serde::Deserialize)]
pub struct CatsListResponse {
    pub status: String,
    pub results: Vec<crate::models::cat::Cat>,
}

#[derive(serde::Serialize, Debug, serde::Deserialize)]
pub struct CatDetailResponse {
    pub status: Status,
    pub message: Option<String>,
    pub results: Option<crate::models::cat::Cat>,
}

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
pub async fn cat_detail_handler(
    State(pool): State<PgPool>,
    Path(name): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, Json<CatDetailResponse>)> {
    tracing::info!("Getting cat detail for {}", name);
    let cat = get_cat_by_name(&pool, name).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CatDetailResponse {
                status: Status::Error,
                message: Option::from(String::from(e.to_string())),
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
