use crate::models::interests::{populate_interests, Interest};
use sqlx::types::time::OffsetDateTime;
use sqlx::types::Uuid;
use sqlx::Error;
use time::serde::rfc3339;

#[derive(serde::Serialize, Debug, serde::Deserialize, PartialEq, utoipa::ToSchema, Default)]
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
    pub interests: Vec<Interest>,
}

pub struct CatRow {
    pub id: Uuid,
    pub name: String,
    pub password: String,
    pub created_at: Option<OffsetDateTime>,
    pub updated_at: Option<OffsetDateTime>,
    pub active: Option<bool>,
    pub avatar_filename: Option<String>,
    pub breed_id: Option<Uuid>,
    pub breed_name: Option<String>,
    pub age: Option<i32>,
    pub biography: Option<String>,
}

impl From<CatRow> for Cat {
    fn from(row: CatRow) -> Self {
        Cat {
            id: row.id,
            name: row.name,
            password: row.password,
            created_at: row.created_at,
            updated_at: row.updated_at,
            active: row.active,
            avatar_filename: row.avatar_filename,
            breed_id: row.breed_id,
            breed_name: row.breed_name,
            age: row.age,
            biography: row.biography,
            interests: vec![],
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
    pub age: Option<i32>,
}

pub async fn get_cat_by_name(pool: &sqlx::PgPool, name: String) -> Result<Option<Cat>, Error> {
    let row = sqlx::query_as!(
        CatRow,
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
    .await?;

    let mut cat = row.map(Cat::from);

    if let Some(c) = cat.as_mut() {
        let mut v = vec![std::mem::take(c)];
        populate_interests(pool, &mut v).await?;
        *c = v.remove(0);
    }

    Ok(cat)
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
    .await?;

    let mut cats: Vec<Cat> = rows.into_iter().map(Cat::from).collect();
    populate_interests(pool, &mut cats).await?;

    Ok(cats)
}

pub async fn add_cat(pool: &sqlx::PgPool, cat: NewCat) -> Result<Cat, Error> {
    let new_cat = sqlx::query_as!(
        CatRow,
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
    Ok(Cat::from(new_cat))
}
