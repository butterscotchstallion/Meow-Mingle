use crate::AppState;
use crate::handlers::common::ApiError;
use crate::models::status::Status;
use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use uuid::Uuid;

pub mod routes {
    pub const BREEDS_LIST: &str = "/api/v1/breeds";
}

#[derive(Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct Breed {
    pub id: Uuid,
    pub name: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct BreedsListResponse {
    pub status: Status,
    pub results: Vec<Breed>,
}

#[axum::debug_handler]
#[utoipa::path(
    get,
    path = routes::BREEDS_LIST,
    responses(
        (status = 200, description = "List of all cat breeds", body = BreedsListResponse),
        (status = 500, description = "Internal server error")
    ),
    tag = "breeds"
)]
pub async fn breeds_list_handler(
    State(state): State<AppState>,
) -> Result<(StatusCode, Json<BreedsListResponse>), ApiError> {
    let breeds = sqlx::query_as!(
        Breed,
        r#"
        SELECT id AS "id!", name AS "name!"
        FROM cat_breeds
        ORDER BY name ASC
        "#
    )
    .fetch_all(&state.pool)
    .await
    .map_err(ApiError::internal)?;

    Ok((
        StatusCode::OK,
        Json(BreedsListResponse {
            status: Status::Ok,
            results: breeds,
        }),
    ))
}
