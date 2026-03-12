use axum::routing::get;
use axum::Router;
use dotenv::dotenv;
use sqlx::{PgPool, Pool, Postgres};
use std::env;
use std::error::Error;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Meow Mingle API",
        description = "A cat dating app to find your purrfect match",
        license(name = "MIT", identifier = "MIT"),
        version = "0.1.0"
    ),
    paths(
        crate::handlers::cats::cats_list_handler,
        crate::handlers::cats::cat_detail_handler,
        crate::handlers::auth::sign_in_handler,
        crate::handlers::auth::sign_up_handler,
        crate::handlers::matches::matches_list_handler,
        crate::handlers::matches::match_suggestions_handler,
        crate::handlers::session::session_get_by_id_handler,
    ),
    components(
        schemas(
            crate::models::cat::Cat,
            crate::handlers::cats::CatsListResponse,
            crate::handlers::cats::CatDetailResponse,
            crate::handlers::matches::Match,
            crate::handlers::matches::MatchStatus,
            crate::handlers::matches::MatchesListResponse,
            crate::handlers::auth::AuthSignUpPayload,
            crate::handlers::auth::AuthSignUpResponse,
        )
    ),
    tags(
        (name = "cats", description = "Cat endpoints"),
        (name = "matches", description = "Match endpoints"),
        (name = "auth", description = "Auth endpoints"),
    )
)]
pub struct ApiDoc;
pub mod config;
pub mod handlers;
pub mod hasher;
pub mod models;
use axum_cookie::prelude::*;

use crate::cats::routes::{CATS_LIST, CAT_DETAIL};
use crate::handlers::auth::routes::{AUTH_SIGN_IN, AUTH_SIGN_UP};
pub use crate::handlers::cats;
use crate::handlers::matches::routes::{MATCHES_LIST, MATCH_SUGGESTIONS};
use crate::handlers::matches::{match_suggestions_handler, matches_list_handler};
use crate::handlers::session::routes::*;
use handlers::auth::*;
use handlers::cats::*;
use handlers::session::*;

pub async fn create_app(pool: PgPool) -> Result<Router, Box<dyn Error>> {
    let api_router = Router::new()
        .route(CATS_LIST, get(cats_list_handler))
        .route(CAT_DETAIL, get(cat_detail_handler))
        .route(SESSION_GET_BY_ID, get(session_get_by_id_handler))
        .route(AUTH_SIGN_IN, axum::routing::post(sign_in_handler))
        .route(AUTH_SIGN_UP, axum::routing::post(sign_up_handler))
        .route(MATCHES_LIST, get(matches_list_handler))
        .route(MATCH_SUGGESTIONS, get(match_suggestions_handler))
        .with_state(pool)
        .layer(CookieLayer::default());

    let swagger_router: Router = SwaggerUi::new("/swagger-ui")
        .url("/api-docs/openapi.json", ApiDoc::openapi())
        .into();

    let app = Router::new().merge(api_router).merge(swagger_router);

    Ok(app)
}

pub async fn get_db_pool() -> Result<Pool<Postgres>, Box<dyn Error>> {
    dotenv().ok();
    let database_url = env::var("MM_DATABASE_URL").expect("MM_DATABASE_URL must be set");
    let pool = PgPool::connect(&database_url).await?;
    Ok(pool)
}
