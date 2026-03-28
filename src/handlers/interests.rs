use crate::handlers::common::ApiError;
use crate::models::{interests::Interest, status::Status};
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use sqlx::PgPool;

pub mod routes {
    pub const INTERESTS_LIST: &str = "/api/v1/interests";
}

#[derive(Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct InterestListResponse {
    pub status: Status,
    pub results: Vec<Interest>,
}

#[axum::debug_handler]
#[utoipa::path(
    get,
    path = routes::INTERESTS_LIST,
    responses(
        (status = 200, description = "List of all interests", body = InterestListResponse),
        (status = 500, description = "Internal server error")
    ),
    tag = "interests"
)]
pub async fn interest_list_handler(
    State(pool): State<PgPool>,
) -> Result<(StatusCode, Json<InterestListResponse>), ApiError> {
    let interests = sqlx::query_as!(
        Interest,
        r#"
        SELECT *
        FROM interests
        ORDER BY name ASC
        "#
    )
    .fetch_all(&pool)
    .await
    .map_err(ApiError::internal)?;

    Ok((
        StatusCode::OK,
        Json(InterestListResponse {
            status: Status::Ok,
            results: interests,
        }),
    ))
}
