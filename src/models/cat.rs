use sqlx::types::time::OffsetDateTime;
use sqlx::types::Uuid;
use sqlx::Error;
use time::serde::rfc3339;

#[derive(serde::Serialize, Debug, serde::Deserialize)]
pub struct Cat {
    pub id: Uuid,
    pub name: String,
    #[serde(skip_serializing, default)]
    pub password: String,
    #[serde(with = "rfc3339::option")]
    pub created_at: Option<OffsetDateTime>,
    #[serde(with = "rfc3339::option")]
    pub updated_at: Option<OffsetDateTime>,
    pub active: Option<bool>,
    pub avatar_filename: Option<String>,
    pub breed_id: Option<Uuid>,
    pub breed_name: Option<String>,
    pub age: Option<i32>,
    pub biography: Option<String>,
}

pub async fn get_cat_by_name(pool: &sqlx::PgPool, name: String) -> Result<Option<Cat>, Error> {
    sqlx::query_as!(
        Cat,
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
        WHERE c.name = $1
        "#,
        name
    )
    .fetch_optional(pool)
    .await
}

pub async fn get_cats(pool: &sqlx::PgPool) -> Result<Vec<Cat>, Error> {
    sqlx::query_as!(
        Cat,
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
        "#
    )
    .fetch_all(pool)
    .await
}
