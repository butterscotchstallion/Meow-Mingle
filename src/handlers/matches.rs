use crate::models::status::Status;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde_json::json;
use sqlx::error::ErrorKind;
use sqlx::{Error, PgPool};
use uuid::Uuid;

pub mod routes {
    pub const MATCHES_LIST: &str = "/api/v1/matches";
    pub const MATCH_SUGGESTIONS: &str = "/api/v1/matches/suggestions";
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct MatchSuggestionsResponse {
    pub status: String,
    pub results: Vec<crate::models::cat::Cat>,
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

#[axum::debug_handler]
pub async fn match_suggestions_handler(
    State(pool): State<PgPool>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let suggestions = sqlx::query_as!(
        crate::models::cat::Cat,
        r#"
        SELECT c.id,
               c.name,
               c.password,
               c.created_at,
               c.updated_at,
               c.active,
               c.avatar_filename,
               c.biography,
               c.age,
               cat_breeds.id AS breed_id,
               cat_breeds.name AS breed_name
        FROM cats c
        JOIN cat_breeds ON c.breed_id = cat_breeds.id
        LEFT JOIN matches m
            ON (m.initiator_id = c.id OR m.target_id = c.id)
        WHERE m.id IS NULL
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
            "results": suggestions
        })),
    ))
}
