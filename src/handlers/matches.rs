use crate::AppState;
use crate::handlers::common::ApiError;
use crate::models::cat::{Cat, CatRow};
use crate::models::interests::populate_interests;
use crate::models::photos::populate_photos;
use crate::models::session::get_cat_from_session_id;
use crate::models::status::Status;
use axum::Json;
use axum::extract::{Query, State};
use axum::http::StatusCode;

use axum_cookie::CookieManager;
use serde_with::{StringWithSeparator, formats::CommaSeparator, serde_as};
use sqlx::{Postgres, QueryBuilder};
use utoipa::ToSchema;
use uuid::Uuid;

pub mod routes {
    pub const MATCHES_LIST: &str = "/api/v1/matches";
    pub const MATCH_SUGGESTIONS: &str = "/api/v1/matches/suggestions";
    pub const MATCH_ADD: &str = "/api/v1/matches";
}

#[derive(Debug, serde::Serialize, serde::Deserialize, sqlx::Type, utoipa::ToSchema, PartialEq)]
#[sqlx(type_name = "match_status", rename_all = "lowercase")]
pub enum MatchStatus {
    Pending,
    Accepted,
    Declined,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema, sqlx::FromRow)]
pub struct Match {
    pub id: Uuid,
    pub initiator_id: Uuid,
    pub target_id: Uuid,
    pub status: Option<MatchStatus>,
    pub seen: Option<bool>,
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

#[derive(serde::Serialize, Debug, serde::Deserialize, utoipa::ToSchema)]
pub struct MatchAddedResponse {
    pub status: Status,
    pub message: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct MatchAddRequest {
    pub initiator_id: Uuid,
    pub target_id: Uuid,
    pub status: MatchStatus,
    pub seen: Option<bool>,
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

#[serde_as]
#[derive(serde::Deserialize)]
#[serde(default)]
pub struct MatchListFilters {
    initiator_id: Option<Uuid>,
    target_id: Option<Uuid>,
    status: Option<MatchStatus>,
    seen: bool,
}

impl Default for MatchListFilters {
    fn default() -> Self {
        Self {
            initiator_id: None,
            target_id: None,
            status: None,
            seen: false,
        }
    }
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
    State(state): State<AppState>,
    cookie_manager: CookieManager,
    Query(match_list_filters): Query<MatchListFilters>,
) -> Result<(StatusCode, Json<MatchesListResponse>), ApiError> {
    let cat = match get_cat_from_session_id(&state.pool, cookie_manager).await {
        Ok(Some(cat)) => cat,
        _ => return Err(ApiError::unauthorized()),
    };

    let mut query: QueryBuilder<Postgres> = QueryBuilder::new(
        r#"SELECT id, initiator_id, target_id, status, seen FROM matches WHERE 1=1"#,
    );

    // Always scope results to the authenticated cat
    query.push(" AND (initiator_id = ");
    query.push_bind(cat.id);
    query.push(" OR target_id = ");
    query.push_bind(cat.id);
    query.push(")");

    // Optional: filter by initiator
    if let Some(initiator_id) = match_list_filters.initiator_id {
        query.push(" AND initiator_id = ");
        query.push_bind(initiator_id);
    }

    // Optional: filter by target
    if let Some(target_id) = match_list_filters.target_id {
        query.push(" AND target_id = ");
        query.push_bind(target_id);
    }

    // Optional: filter by status — default to excluding declined when no status is provided
    if let Some(status) = match_list_filters.status {
        query.push(" AND status = ");
        query.push_bind(status);
    } else {
        query.push(" AND status != 'declined'");
    }

    // Filter by seen — defaults to false so callers get unseen matches by default
    query.push(" AND seen = ");
    query.push_bind(match_list_filters.seen);

    let matches = query
        .build_query_as::<Match>()
        .fetch_all(&state.pool)
        .await
        .map_err(|e: sqlx::Error| ApiError::internal(e))?;

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
    State(state): State<AppState>,
    cookie_manager: CookieManager,
    age_filter: Query<MatchSuggestionAgeFilter>,
) -> Result<(StatusCode, Json<MatchSuggestionsResponse>), ApiError> {
    let cat = match get_cat_from_session_id(&state.pool, cookie_manager).await {
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
               c.last_seen,
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
        .fetch_all(&state.pool)
        .await
        .map_err(|e: sqlx::Error| ApiError::internal(e))?;

    let mut suggestions: Vec<Cat> = rows.into_iter().map(Cat::from).collect();
    populate_interests(&state.pool, &mut suggestions)
        .await
        .map_err(ApiError::internal)?;
    populate_photos(&state.pool, &mut suggestions)
        .await
        .map_err(ApiError::internal)?;

    Ok((
        StatusCode::OK,
        Json(MatchSuggestionsResponse {
            status: Status::Ok,
            results: suggestions,
        }),
    ))
}

// Also handles updates!
#[axum::debug_handler]
#[utoipa::path(
    post,
    path = routes::MATCH_ADD,
    responses(
        (status = 201, description = "Match created", body = MatchAddedResponse),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn match_add_update_handler(
    State(state): State<AppState>,
    cookie_manager: CookieManager,
    Json(match_request): Json<MatchAddRequest>,
) -> Result<(StatusCode, Json<MatchAddedResponse>), ApiError> {
    let cat = match get_cat_from_session_id(&state.pool, cookie_manager).await {
        Ok(Some(cat)) => cat,
        _ => return Err(ApiError::unauthorized()),
    };
    let initiator_id = cat.id;

    sqlx::query(
        r#"
        INSERT INTO matches (initiator_id, target_id, status, seen)
        VALUES ($1, $2, $3, COALESCE($4, false))
        ON CONFLICT (initiator_id, target_id)
        DO UPDATE SET status = $3, seen = COALESCE($4, matches.seen)
    "#,
    )
    .bind(initiator_id)
    .bind(match_request.target_id)
    .bind(match_request.status)
    .bind(match_request.seen)
    .execute(&state.pool)
    .await
    .map_err(ApiError::internal)?;

    Ok((
        StatusCode::CREATED,
        Json(MatchAddedResponse {
            status: Status::Ok,
            message: String::from("Match created"),
        }),
    ))
}
