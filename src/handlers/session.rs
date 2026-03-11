use crate::cats::CatDetailResponse;
use crate::models::cat::{Cat, CatRow};
use crate::models::interests::populate_interests;
use crate::models::status::Status;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use sqlx::PgPool;
use uuid::Uuid;

pub mod routes {
    pub const SESSION_GET_BY_ID: &str = "/api/v1/session/{id}";
}

#[axum::debug_handler]
#[utoipa::path(
    get,
    path = routes::SESSION_GET_BY_ID,
    params(
        ("id" = String, Path, description = "Session id")
    ),
    responses(
        (status = 200, description = "Details for a specific cat", body = CatDetailResponse),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn session_get_by_id_handler(
    State(pool): State<PgPool>,
    Path(session_id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, Json<CatDetailResponse>)> {
    let row = sqlx::query_as!(
        CatRow,
        r#"
            SELECT c.*, b.name AS breed_name
            FROM cats c
            JOIN sessions s ON c.id = s.cat_id
            JOIN cat_breeds b ON c.breed_id = b.id
            WHERE s.active = true
            AND (
                s.created_at >= DATE_TRUNC('month', current_date - interval '1' month)
                OR
                s.updated_at >= DATE_TRUNC('month', current_date - interval '1' month)
            )
            AND s.session_id = $1
        "#,
        session_id as Uuid
    )
    .fetch_optional(&pool)
    .await
    .map_err(|e: sqlx::Error| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CatDetailResponse {
                status: Status::Error,
                message: Some(e.to_string()),
                results: None,
            }),
        )
    })?;

    let mut cat = row.map(Cat::from);
    if let Some(c) = cat.as_mut() {
        let mut v = vec![std::mem::take(c)];
        populate_interests(&pool, &mut v).await.map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(CatDetailResponse {
                    status: Status::Error,
                    message: Some(e.to_string()),
                    results: None,
                }),
            )
        })?;
        *c = v.remove(0);
    }

    match cat {
        Some(cat) => Ok((
            StatusCode::OK,
            Json(CatDetailResponse {
                status: Status::Ok,
                message: None,
                results: Some(cat),
            }),
        )),
        None => Err((
            StatusCode::NOT_FOUND,
            Json(CatDetailResponse {
                status: Status::Error,
                message: Some("Session not found".to_string()),
                results: None,
            }),
        )),
    }
}
