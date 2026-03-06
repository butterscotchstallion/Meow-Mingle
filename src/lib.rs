use crate::handlers::*;
use axum::routing::get;
use axum::Router;
use dotenv::dotenv;
use sqlx::{PgPool, Pool, Postgres};
use std::env;
use std::error::Error;

mod cat;
mod handlers;
pub mod hasher;
mod session;
mod status;

pub mod routes {
    pub const CATS_LIST: &str = "/";
    pub const SESSION_GET_BY_ID: &str = "/sessions/{id}";
}

pub async fn create_app(pool: PgPool) -> Result<Router, Box<dyn std::error::Error>> {
    let app = Router::new()
        .route(routes::CATS_LIST, get(cats_list_handler))
        .route(routes::SESSION_GET_BY_ID, get(session_get_by_id_handler))
        .with_state(pool);

    Ok(app)
}

pub async fn get_db_pool() -> Result<Pool<Postgres>, Box<dyn Error>> {
    dotenv().ok();
    let database_url = env::var("MM_DATABASE_URL").expect("MM_DATABASE_URL must be set");
    let pool = PgPool::connect(&database_url).await?;
    Ok(pool)
}
