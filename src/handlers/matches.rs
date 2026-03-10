use crate::models::status::Status;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde_json::json;
use sqlx::{Error, PgPool};
use uuid::Uuid;

pub mod routes {
    pub const MATCHES_LIST: &str = "/api/v1/matches";
}

#[derive(Debug, serde::Serialize, serde::Deserialize, sqlx::Type)]
#[sqlx(type_name = "match_status", rename_all = "lowercase")]
pub enum MatchStatus {
    Pending,
    Accepted,
    Declined,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Match {
    pub id: Uuid,
    pub initiator_id: Uuid,
    pub target_id: Uuid,
    pub status: MatchStatus,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct MatchesListResponse {
    pub status: String,
    pub results: Vec<Match>,
}

#[axum::debug_handler]
pub async fn matches_list_handler(
    State(pool): State<PgPool>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let matches = sqlx::query_as!(
        Match,
        r#"
        SELECT id, initiator_id, target_id, status AS "status: MatchStatus"
        FROM matches
        "#
    )
    .fetch_all(&pool)
    .await
    .map_err(|e: Error| {
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
            "results": matches
        })),
    ))
}
