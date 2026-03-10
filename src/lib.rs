use axum::routing::get;
use axum::Router;
use dotenv::dotenv;
use sqlx::{PgPool, Pool, Postgres};
use std::env;
use std::error::Error;

pub mod config;
pub mod handlers;
pub mod hasher;
pub mod models;

use crate::cats::routes::{CATS_LIST, CAT_DETAIL};
use crate::handlers::auth::routes::{AUTH_SIGN_IN, AUTH_SIGN_UP};
pub use crate::handlers::cats;
use crate::handlers::matches::matches_list_handler;
use crate::handlers::matches::routes::MATCHES_LIST;
use crate::handlers::session::routes::*;
use handlers::auth::*;
use handlers::cats::*;
use handlers::session::*;

pub async fn create_app(pool: PgPool) -> Result<Router, Box<dyn Error>> {
    let app = Router::new()
        .route(CATS_LIST, get(cats_list_handler))
        .route(CAT_DETAIL, get(cat_detail_handler))
        .route(SESSION_GET_BY_ID, get(session_get_by_id_handler))
        .route(AUTH_SIGN_IN, axum::routing::post(sign_in_handler))
        .route(AUTH_SIGN_UP, axum::routing::post(sign_up_handler))
        .route(MATCHES_LIST, get(matches_list_handler))
        .with_state(pool);

    Ok(app)
}

pub async fn get_db_pool() -> Result<Pool<Postgres>, Box<dyn Error>> {
    dotenv().ok();
    let database_url = env::var("MM_DATABASE_URL").expect("MM_DATABASE_URL must be set");
    let pool = PgPool::connect(&database_url).await?;
    Ok(pool)
}
