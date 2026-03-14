use crate::handlers::common::ApiError;
use crate::models::cat::{Cat, CatRow};
use crate::models::interests::populate_interests;
use crate::models::session::get_cat_from_session_id;
use crate::models::status::Status;
use axum::Json;
use axum::extract::{Query, State};
use axum::http::StatusCode;

use axum_cookie::CookieManager;
use serde_with::{StringWithSeparator, formats::CommaSeparator, serde_as};
use sqlx::{Error, PgPool, Postgres, QueryBuilder};
use utoipa::ToSchema;
use uuid::Uuid;

pub mod routes {
    pub const MATCHES_LIST: &str = "/api/v1/matches";
    pub const MATCH_SUGGESTIONS: &str = "/api/v1/matches/suggestions";
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

#[derive(Debug, serde::Serialize, serde::Deserialize, ToSchema)]
pub struct MatchSuggestionsResponse {
    pub status: Status,
    pub results: Vec<Cat>,
}

#[serde_as]
#[derive(serde::Deserialize, Default)]
#[serde(default)]
pub struct MatchSuggestionAgeFilter {
    pub lt: i32,
    pub gt: i32,
    #[serde_as(as = "StringWithSeparator::<CommaSeparator, Uuid>")]
    pub interest_ids: Vec<Uuid>,
}

#[axum::debug_handler]
#[utoipa::path(
    get,
    path = routes::MATCHES_LIST,
    responses(
        (status = 200, description = "List of all matches for a specific cat", body = MatchesListResponse),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn matches_list_handler(
    State(pool): State<PgPool>,
    cookie_manager: CookieManager,
) -> Result<(StatusCode, Json<MatchesListResponse>), ApiError> {
    let cat = match get_cat_from_session_id(&pool, cookie_manager).await {
        Ok(Some(cat)) => cat,
        _ => return Err(ApiError::unauthorized()),
    };

    let matches = sqlx::query_as!(
        Match,
        r#"
        SELECT id, initiator_id, target_id, status AS "status: MatchStatus"
        FROM matches
        WHERE 1=1
        AND (initiator_id = $1 OR target_id = $1)
        AND matches.status != 'declined'
        "#,
        cat.id
    )
    .fetch_all(&pool)
    .await
    .map_err(|e: Error| ApiError::internal(e))?;

    Ok((
        StatusCode::OK,
        Json(MatchesListResponse {
            status: Status::Ok,
            results: matches,
        }),
    ))
}

#[axum::debug_handler]
#[utoipa::path(
    get,
    path = routes::MATCH_SUGGESTIONS,
    responses(
        (status = 200, description = "List of match suggestions for a specific cat", body = MatchSuggestionsResponse),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn match_suggestions_handler(
    State(pool): State<PgPool>,
    cookie_manager: CookieManager,
    age_filter: Query<MatchSuggestionAgeFilter>,
) -> Result<(StatusCode, Json<MatchSuggestionsResponse>), ApiError> {
    let cat = match get_cat_from_session_id(&pool, cookie_manager).await {
        Ok(Some(cat)) => cat,
        _ => return Err(ApiError::unauthorized()),
    };

    tracing::info!("Getting suggestions for cat: {:?}", cat);

    let mut query: QueryBuilder<Postgres> = QueryBuilder::new(
        r#"
        SELECT c.id,
               c.name,
               c.password,
               c.created_at,
               c.updated_at,
               c.active,
               c.avatar_filename,
               c.biography,
               c.birth_date,
               cat_breeds.id AS breed_id,
               cat_breeds.name AS breed_name,
               DATE_PART('year', AGE(c.birth_date))::int AS age
        FROM cats c
        JOIN cat_breeds ON c.breed_id = cat_breeds.id
        LEFT JOIN matches m
            ON (m.initiator_id = c.id OR m.target_id = c.id)
        WHERE m.id IS NULL
        AND c.id != "#,
    );
    query.push_bind(cat.id);

    // age_filter.lt = "younger than N years" => birth_date > NOW() - INTERVAL 'N years'
    if age_filter.lt > 0 {
        query.push(" AND c.birth_date > NOW() - ");
        query.push_bind(format!("{} years", age_filter.lt));
        query.push("::interval");
    }

    // age_filter.gt = "older than N years" => birth_date < NOW() - INTERVAL 'N years'
    if age_filter.gt > 0 {
        query.push(" AND c.birth_date < NOW() - ");
        query.push_bind(format!("{} years", age_filter.gt));
        query.push("::interval");
    }

    // interest_ids = only return cats that have at least one matching interest
    if !age_filter.interest_ids.is_empty() {
        query.push(" AND EXISTS (SELECT 1 FROM cats_interests ci WHERE ci.cat_id = c.id AND ci.interest_id = ANY(");
        query.push_bind(age_filter.interest_ids.clone());
        query.push("))");
    }

    let rows = query
        .build_query_as::<CatRow>()
        .fetch_all(&pool)
        .await
        .map_err(|e: Error| ApiError::internal(e))?;

    let mut suggestions: Vec<Cat> = rows.into_iter().map(Cat::from).collect();
    populate_interests(&pool, &mut suggestions)
        .await
        .map_err(|e| ApiError::internal(e))?;

    Ok((
        StatusCode::OK,
        Json(MatchSuggestionsResponse {
            status: Status::Ok,
            results: suggestions,
        }),
    ))
}
