use sqlx::types::time::OffsetDateTime;
use sqlx::types::Uuid;
use sqlx::Error;
use time::serde::rfc3339;

#[derive(serde::Serialize, Debug, serde::Deserialize)]
pub struct Cat {
    pub id: Uuid,
    pub username: String,
    #[serde(skip_serializing, default)]
    pub password: String,
    #[serde(with = "rfc3339::option")]
    pub created_at: Option<OffsetDateTime>,
    #[serde(with = "rfc3339::option")]
    pub updated_at: Option<OffsetDateTime>,
    pub active: Option<bool>,
    pub avatar_filename: Option<String>,
    pub breed_id: Option<Uuid>,
    pub age: Option<i32>,
    pub biography: Option<String>,
}

pub async fn get_cat_by_username(
    pool: &sqlx::PgPool,
    username: String,
) -> Result<Option<Cat>, Error> {
    sqlx::query_as!(Cat, "SELECT * FROM cats WHERE username = $1", username)
        .fetch_optional(pool)
        .await
}

pub async fn get_cats(pool: &sqlx::PgPool) -> Result<Vec<Cat>, Error> {
    sqlx::query_as!(Cat, "SELECT * FROM cats")
        .fetch_all(pool)
        .await
}
