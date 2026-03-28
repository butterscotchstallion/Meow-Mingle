use crate::AppState;
use crate::cats::CatDetailResponse;
use crate::handlers::common::ApiError;
use crate::models::cat::{Cat, CatRow};
use crate::models::interests::populate_interests;
use crate::models::session::get_session_id_from_cookie;
use crate::models::status::Status;
use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum_cookie::CookieManager;
use sqlx::Error;

pub mod routes {
    pub const SESSION_GET_FROM_COOKIE: &str = "/api/v1/session";
}

#[axum::debug_handler]
#[utoipa::path(
    get,
    path = routes::SESSION_GET_FROM_COOKIE,
    responses(
        (status = 200, description = "Session for current user", body = CatDetailResponse),
        (status = 404, description = "Session not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn session_get_from_cookie_handler(
    State(state): State<AppState>,
    cookie_manager: CookieManager,
) -> Result<(StatusCode, Json<CatDetailResponse>), ApiError> {
    let session_id = get_session_id_from_cookie(cookie_manager);
    let row = sqlx::query_as!(
        CatRow,
        r#"
            SELECT c.id,
                   c.name,
                   c.password,
                   c.created_at,
                   c.updated_at,
                   c.last_seen,
                   c.active,
                   c.avatar_filename,
                   c.biography,
                   c.birth_date,
                   c.breed_id,
                   b.name AS breed_name,
                   DATE_PART('year', AGE(c.birth_date))::int AS age
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
        session_id
    )
    .fetch_optional(&state.pool)
    .await
    .map_err(|e: Error| ApiError::internal(e))?;

    let mut cat = row.map(Cat::from);
    if let Some(c) = cat.as_mut() {
        let mut v = vec![std::mem::take(c)];
        populate_interests(&state.pool, &mut v)
            .await
            .map_err(ApiError::internal)?;
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
        None => Err(ApiError::not_found()),
    }
}
