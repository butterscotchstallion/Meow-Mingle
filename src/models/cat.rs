use sqlx::types::time::OffsetDateTime;
use sqlx::types::Uuid;
use sqlx::Error;
use time::serde::rfc3339;

#[derive(serde::Serialize, Debug, serde::Deserialize, PartialEq)]
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

// Used when registering a new cat - we don't have every field
// for the whole struct
#[derive(serde::Serialize, Debug, serde::Deserialize, PartialEq)]
pub struct NewCat {
    pub name: String,
    pub password: String,
    pub breed_id: Uuid,
    pub age: Option<i32>,
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

pub async fn add_cat(pool: &sqlx::PgPool, cat: NewCat) -> Result<Cat, Error> {
    let new_cat = sqlx::query_as!(
        Cat,
        r"
        INSERT INTO cats (name, password, age, breed_id)
            VALUES ($1, $2, $3, $4)
            RETURNING
                id,
                name,
                password,
                created_at,
                updated_at,
                active,
                avatar_filename,
                biography,
                age,
                breed_id,
                NULL::text AS breed_name
",
        cat.name,
        cat.password,
        cat.age,
        cat.breed_id
    )
    .fetch_one(pool)
    .await?;
    Ok(new_cat)
}
