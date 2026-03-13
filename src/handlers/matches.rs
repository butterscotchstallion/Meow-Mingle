use crate::handlers::common::get_unauthorized_response;
use crate::models::cat::{Cat, CatRow};
use crate::models::interests::populate_interests;
use crate::models::session::get_cat_from_session_id;
use crate::models::status::Status;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use axum_cookie::CookieManager;
use serde_json::json;
use sqlx::{Error, PgPool};
use utoipa::ToSchema;
use uuid::Uuid;

pub mod routes {
    pub const MATCHES_LIST: &str = "/api/v1/matches";
    pub const MATCH_SUGGESTIONS: &str = "/api/v1/matches/suggestions";
}

#[derive(Debug, serde::Serialize, serde::Deserialize, ToSchema)]
pub struct MatchSuggestionsResponse {
    pub status: Status,
    pub results: Vec<Cat>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, sqlx::Type, utoipa::ToSchema)]
#[sqlx(type_name = "match_status", rename_all = "lowercase")]
pub enum MatchStatus {
    Pending,
    Accepted,
    Declined,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct Match {
    pub id: Uuid,
    pub initiator_id: Uuid,
    pub target_id: Uuid,
    pub status: MatchStatus,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, ToSchema)]
pub struct MatchesListResponse {
    pub status: Status,
    pub results: Vec<Match>,
}

#[axum::debug_handler]
#[utoipa::path(
    get,
    path = routes::MATCHES_LIST,
    responses(
        (status = 200, description = "List of all matches for a specific cat", body = MatchesListResponse),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn matches_list_handler(
    State(pool): State<PgPool>,
    cookie_manager: CookieManager,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let unauthorized_response = get_unauthorized_response();
    let cat = match get_cat_from_session_id(&pool, cookie_manager).await {
        Ok(Some(cat)) => cat,
        Err(_) => return Ok(unauthorized_response),
        _ => return Ok(unauthorized_response),
    };
    let matches = sqlx::query_as!(
        Match,
        r#"
        SELECT id, initiator_id, target_id, status AS "status: MatchStatus"
        FROM matches
        WHERE initiator_id = $1
        AND matches.status != 'declined'
        "#,
        cat.id
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
#[utoipa::path(
    get,
    path = routes::MATCH_SUGGESTIONS,
    responses(
        (status = 200, description = "List of match suggestions for a specific cat", body = MatchSuggestionsResponse),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn match_suggestions_handler(
    State(pool): State<PgPool>,
    cookie_manager: CookieManager,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let unauthorized_response = get_unauthorized_response();
    let cat = match get_cat_from_session_id(&pool, cookie_manager).await {
        Ok(Some(cat)) => cat,
        Err(_) => return Ok(unauthorized_response),
        _ => return Ok(unauthorized_response),
    };
    tracing::info!("Getting suggestions for cat: {:?}", cat);
    let rows = sqlx::query_as!(
        CatRow,
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
        AND c.id = $1
        "#,
        cat.id
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

    let mut suggestions: Vec<Cat> = rows.into_iter().map(Cat::from).collect();
    populate_interests(&pool, &mut suggestions)
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
            "results": suggestions
        })),
    ))
}
