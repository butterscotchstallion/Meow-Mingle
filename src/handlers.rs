use crate::cat::Cat;
use crate::session::Session;
use crate::status::Status;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

#[axum::debug_handler]
pub async fn cats_list_handler(
    State(pool): State<PgPool>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let cats: Vec<Cat> = sqlx::query_as!(
        Cat,
        "SELECT id, username, created_at, updated_at, active, avatar_filename FROM cats",
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| {
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
pub async fn session_get_by_id_handler(
    State(pool): State<PgPool>,
    Path(session_id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let session = sqlx::query_as!(
        Session,
        r#"
            SELECT id, cat_id, created_at, updated_at, active
            FROM sessions
            WHERE id = $1
        "#,
        session_id as Uuid
    )
    .fetch_optional(&pool)
    .await
    .map_err(|e: sqlx::Error| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "status": Status::Error,
                "message": e.to_string()
            })),
        )
    })?;

    match session {
        Some(session) => Ok((
            StatusCode::OK,
            Json(json!({
                "status": Status::Ok,
                "result": session
            })),
        )),
        None => Err((
            StatusCode::NOT_FOUND,
            Json(json!({
                "status": Status::Error,
                "message": "session not found"
            })),
        )),
    }
}
