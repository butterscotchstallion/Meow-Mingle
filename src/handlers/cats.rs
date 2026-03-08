use crate::models::cat::get_cats;
use crate::models::status::Status;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde_json::json;
use sqlx::PgPool;

pub mod routes {
    pub const CATS_LIST: &str = "/cats";
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
