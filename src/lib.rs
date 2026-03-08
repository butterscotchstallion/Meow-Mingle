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

pub use crate::handlers::cats;
use crate::handlers::session::routes::*;
use handlers::auth::*;
use handlers::cats::*;
use handlers::session::*;

pub async fn create_app(pool: PgPool) -> Result<Router, Box<dyn Error>> {
    let app = Router::new()
        .route(cats::routes::CATS_LIST, get(cats_list_handler))
        .route(cats::routes::CAT_DETAIL, get(cat_detail_handler))
        .route(SESSION_GET_BY_ID, get(session_get_by_id_handler))
        .route(
            handlers::auth::routes::AUTH_LOGIN,
            axum::routing::post(login_handler),
        )
        .with_state(pool);

    Ok(app)
}

pub async fn get_db_pool() -> Result<Pool<Postgres>, Box<dyn Error>> {
    dotenv().ok();
    let database_url = env::var("MM_DATABASE_URL").expect("MM_DATABASE_URL must be set");
    let pool = PgPool::connect(&database_url).await?;
    Ok(pool)
}
