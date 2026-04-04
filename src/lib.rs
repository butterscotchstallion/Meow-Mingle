use crate::config::AppConfig;
use axum::Router;
use axum::extract::DefaultBodyLimit;
use axum::http::{HeaderName, HeaderValue, Method};
use axum::routing::{get, put};
use dotenv::dotenv;
use sqlx::{PgPool, Pool, Postgres};
use std::env;
use std::error::Error;
use tower_http::cors::{AllowOrigin, CorsLayer};

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub config: AppConfig,
}
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Meow Mingle API",
        description = "A dating app for cats to find your purrfect match",
        license(name = "MIT", identifier = "MIT"),
        version = "0.1.0"
    ),
    paths(
        crate::handlers::cats::cat_autocomplete_handler,
        crate::handlers::cats::cat_detail_handler,
        crate::handlers::cats::cat_session_profile_handler,
        crate::handlers::cats::cat_update_profile_handler,
        crate::handlers::auth::sign_in_handler,
        crate::handlers::auth::sign_up_handler,
        crate::handlers::matches::matches_list_handler,
        crate::handlers::matches::match_suggestions_handler,
        crate::handlers::matches::match_add_update_handler,
        crate::handlers::matches::match_mark_seen_handler,
        crate::handlers::session::session_get_from_cookie_handler,
        crate::handlers::breeds::breeds_list_handler,
        crate::handlers::rbac::cat_roles_list_handler,
        crate::handlers::interests::interest_list_handler
    ),
    components(
        schemas(
            crate::models::cat::Cat,
            crate::handlers::cats::CatAutocompleteResponse,
            crate::handlers::cats::CatDetailResponse,
            crate::handlers::common::GenericResponse,
            crate::handlers::matches::Match,
            crate::handlers::matches::MatchStatus,
            crate::handlers::matches::MatchesListResponse,
            crate::handlers::matches::MatchAddedResponse,
            crate::handlers::auth::AuthSignUpPayload,
            crate::handlers::auth::AuthSignUpResponse,
            crate::handlers::breeds::Breed,
            crate::handlers::breeds::BreedsListResponse,
            crate::handlers::rbac::CatRoleListResponse,
            crate::handlers::interests::InterestListResponse
        )
    ),
    tags(
        (name = "cats", description = "Cat endpoints"),
        (name = "matches", description = "Match endpoints"),
        (name = "auth", description = "Auth endpoints"),
        (name = "rbac", description = "Role Based Access Control endpoints"),
        (name = "interests", description = "Interests listed on profiles"),
    )
)]
pub struct ApiDoc;
pub mod config;
pub mod handlers;
pub mod hasher;
pub mod models;
use axum_cookie::prelude::*;

use crate::cats::routes::CAT_DETAIL;
use crate::handlers::auth::routes::{AUTH_IMPERSONATE, AUTH_SIGN_IN, AUTH_SIGN_UP};
use crate::handlers::breeds::breeds_list_handler;
use crate::handlers::breeds::routes::BREEDS_LIST;
pub use crate::handlers::cats;
use crate::handlers::cats::routes::{CAT_AUTOCOMPLETE, CAT_SESSION_PROFILE};
use crate::handlers::interests::interest_list_handler;
use crate::handlers::interests::routes::INTERESTS_LIST;
use crate::handlers::matches::routes::{
    MATCH_ADD, MATCH_MARK_SEEN, MATCH_SUGGESTIONS, MATCHES_LIST,
};
use crate::handlers::matches::{
    match_add_update_handler, match_mark_seen_handler, match_suggestions_handler,
    matches_list_handler,
};
use crate::handlers::rbac::cat_roles_list_handler;
use crate::handlers::rbac::routes::CAT_ROLE_LIST;
use crate::handlers::session::routes::*;
use handlers::auth::*;
use handlers::cats::*;
use handlers::session::*;

pub async fn create_app(pool: PgPool, config: AppConfig) -> Result<Router, Box<dyn Error>> {
    let state = AppState { pool, config };
    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::exact(HeaderValue::from_static(
            "http://localhost:5173",
        )))
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::OPTIONS])
        .allow_headers([
            HeaderName::from_static("content-type"),
            HeaderName::from_static("authorization"),
            HeaderName::from_static("cookie"),
        ])
        .allow_credentials(true);

    let api_router = Router::new()
        .route(CAT_DETAIL, get(cat_detail_handler))
        .route(CAT_SESSION_PROFILE, get(cat_session_profile_handler))
        .route(CAT_AUTOCOMPLETE, get(cat_autocomplete_handler))
        .route(CAT_SESSION_PROFILE, put(cat_update_profile_handler))
        .route(
            SESSION_GET_FROM_COOKIE,
            get(session_get_from_cookie_handler),
        )
        .route(AUTH_SIGN_IN, axum::routing::post(sign_in_handler))
        .route(AUTH_SIGN_UP, axum::routing::post(sign_up_handler))
        .route(AUTH_IMPERSONATE, axum::routing::post(impersonate_handler))
        .route(MATCHES_LIST, get(matches_list_handler))
        .route(MATCH_SUGGESTIONS, get(match_suggestions_handler))
        .route(MATCH_ADD, axum::routing::post(match_add_update_handler))
        .route(MATCH_MARK_SEEN, put(match_mark_seen_handler))
        .route(BREEDS_LIST, get(breeds_list_handler))
        .route(CAT_ROLE_LIST, get(cat_roles_list_handler))
        .route(INTERESTS_LIST, get(interest_list_handler))
        .with_state(state)
        .layer(DefaultBodyLimit::max(25 * 1024 * 1024))
        .layer(CookieLayer::default())
        .layer(cors);

    let swagger_router: Router = SwaggerUi::new("/swagger-ui")
        .url("/api-docs/openapi.json", ApiDoc::openapi())
        .into();

    let app = Router::new().merge(api_router).merge(swagger_router);

    Ok(app)
}

pub async fn get_db_pool() -> Result<Pool<Postgres>, Box<dyn Error>> {
    dotenv().ok();
    let database_url = env::var("MM_DATABASE_URL").expect("MM_DATABASE_URL must be set");

    // Keep the per-process pool small so that all test binaries running in
    // parallel don't saturate Postgres's connection limit.
    // Production deployments can override this via MM_DB_MAX_CONNECTIONS.
    let max_connections: u32 = env::var("MM_DB_MAX_CONNECTIONS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(5);

    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(max_connections)
        .connect(&database_url)
        .await?;

    Ok(pool)
}
