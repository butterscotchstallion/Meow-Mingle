use crate::models::interests::{Interest, populate_interests, with_interests};
use crate::models::photos::CatPhoto;
use crate::models::photos::{populate_photos, with_photos};
use sqlx::Error;
use sqlx::types::Uuid;
use sqlx::types::time::OffsetDateTime;
use time::serde::rfc3339;

#[derive(serde::Serialize, Debug, serde::Deserialize, PartialEq, utoipa::ToSchema, Default)]
pub struct Cat {
    pub id: Uuid,
    pub name: String,
    #[serde(skip_serializing, default)]
    pub password: String,
    #[serde(with = "rfc3339::option", rename = "createdAt")]
    pub created_at: Option<OffsetDateTime>,
    #[serde(with = "rfc3339::option", rename = "updatedAt")]
    pub updated_at: Option<OffsetDateTime>,
    #[serde(with = "rfc3339::option", rename = "lastSeen")]
    pub last_seen: Option<OffsetDateTime>,
    pub active: Option<bool>,
    #[serde(rename = "avatarFilename")]
    pub avatar_filename: Option<String>,
    #[serde(rename = "breedId")]
    pub breed_id: Option<Uuid>,
    #[serde(rename = "breedName")]
    pub breed_name: Option<String>,
    #[serde(with = "rfc3339::option", rename = "birthDate")]
    pub birth_date: Option<OffsetDateTime>,
    pub biography: Option<String>,
    pub age: Option<i32>,
    pub interests: Vec<Interest>,
    pub photos: Vec<CatPhoto>,
}

#[derive(sqlx::FromRow)]
pub struct CatRow {
    pub id: Uuid,
    pub name: String,
    pub password: String,
    pub created_at: Option<OffsetDateTime>,
    pub updated_at: Option<OffsetDateTime>,
    pub last_seen: Option<OffsetDateTime>,
    pub active: Option<bool>,
    pub avatar_filename: Option<String>,
    pub breed_id: Option<Uuid>,
    pub breed_name: Option<String>,
    pub birth_date: Option<OffsetDateTime>,
    pub biography: Option<String>,
    pub age: Option<i32>,
}

impl From<CatRow> for Cat {
    fn from(row: CatRow) -> Self {
        Cat {
            id: row.id,
            name: row.name,
            password: row.password,
            created_at: row.created_at,
            updated_at: row.updated_at,
            last_seen: row.last_seen,
            active: row.active,
            avatar_filename: row.avatar_filename,
            breed_id: row.breed_id,
            breed_name: row.breed_name,
            birth_date: row.birth_date,
            biography: row.biography,
            age: row.age,
            interests: vec![],
            photos: vec![],
        }
    }
}

// Used when registering a new cat - we don't have every field
// for the whole struct
#[derive(serde::Serialize, Debug, serde::Deserialize, PartialEq, utoipa::ToSchema)]
pub struct NewCat {
    pub name: String,
    pub password: String,
    pub breed_id: Uuid,
    pub birth_date: Option<OffsetDateTime>,
}

pub async fn get_cat_by_id(pool: &sqlx::PgPool, id: Uuid) -> Result<Option<Cat>, Error> {
    let row: Option<CatRow> = sqlx::query_as!(
        CatRow,
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
        WHERE c.id = $1
        "#,
        id
    )
    .fetch_optional(pool)
    .await?;

    let cat = with_interests(pool, row).await?;
    with_photos(pool, cat).await
}

pub async fn get_cat_by_name(pool: &sqlx::PgPool, name: String) -> Result<Option<Cat>, Error> {
    let row: Option<CatRow> = sqlx::query_as!(
        CatRow,
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
        WHERE c.name = $1
        "#,
        name
    )
    .fetch_optional(pool)
    .await?;

    let cat = with_interests(pool, row).await?;
    with_photos(pool, cat).await
}

pub async fn get_cats(pool: &sqlx::PgPool) -> Result<Vec<Cat>, Error> {
    let rows = sqlx::query_as!(
        CatRow,
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
        "#
    )
    .fetch_all(pool)
    .await?;

    let mut cats: Vec<Cat> = rows.into_iter().map(Cat::from).collect();
    populate_interests(pool, &mut cats).await?;
    populate_photos(pool, &mut cats).await?;

    Ok(cats)
}

pub async fn add_cat(pool: &sqlx::PgPool, cat: NewCat) -> Result<Cat, Error> {
    let new_cat = sqlx::query_as!(
        CatRow,
        r"
        INSERT INTO cats (name, password, birth_date, breed_id)
            VALUES ($1, $2, $3, $4)
            RETURNING
                id,
                name,
                password,
                created_at,
                updated_at,
                last_seen,
                active,
                avatar_filename,
                biography,
                birth_date,
                breed_id,
                NULL::text AS breed_name,
                DATE_PART('year', AGE(birth_date))::int AS age
        ",
        cat.name,
        cat.password,
        cat.birth_date,
        cat.breed_id
    )
    .fetch_one(pool)
    .await?;
    Ok(Cat::from(new_cat))
}

pub async fn update_last_seen(pool: &sqlx::PgPool, cat_id: Uuid) -> Result<(), Error> {
    sqlx::query!(
        r#"
        UPDATE cats
        SET last_seen = NOW()
        WHERE id = $1
        "#,
        cat_id
    )
    .execute(pool)
    .await?;
    Ok(())
}
