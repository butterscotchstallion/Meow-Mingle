use crate::handlers::*;
use axum::routing::get;
use axum::Router;
use sqlx::PgPool;

mod cat;
mod handlers;
pub mod hasher;
mod status;

pub async fn create_app(pool: PgPool) -> Result<Router, Box<dyn std::error::Error>> {
    let app = Router::new()
        .route("/", get(cats_list_handler))
        .with_state(pool);

    Ok(app)
}
