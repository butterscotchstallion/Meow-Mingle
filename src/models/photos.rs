use sqlx::types::Uuid;
use sqlx::types::time::OffsetDateTime;
use sqlx::{Error, PgPool};
use std::collections::HashMap;
use time::serde::rfc3339;

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, utoipa::ToSchema, Clone)]
pub struct CatPhoto {
    pub id: Uuid,
    pub order: Option<i32>,
    #[serde(with = "rfc3339::option", rename = "createdAt")]
    pub created_at: Option<OffsetDateTime>,
    pub filename: String,
    pub width: Option<i32>,
    pub height: Option<i32>,
    #[serde(rename = "altText")]
    pub alt_text: Option<String>,
}

#[derive(Debug, sqlx::FromRow)]
pub struct CatPhotoRow {
    pub cat_id: Uuid,
    pub photo_id: Uuid,
    pub order: Option<i32>,
    pub created_at: Option<OffsetDateTime>,
    pub filename: String,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub alt_text: Option<String>,
}

pub async fn delete_existing_photos(pool: &PgPool, cat_id: Uuid) -> Result<u64, sqlx::Error> {
    let rows_affected = sqlx::query(r#"DELETE FROM cats_photos WHERE cat_id = $1"#)
        .bind(cat_id)
        .execute(pool)
        .await?
        .rows_affected();
    Ok(rows_affected)
}

pub async fn get_cat_photos_map(pool: &PgPool) -> Result<HashMap<Uuid, Vec<CatPhoto>>, Error> {
    let rows = sqlx::query_as!(
        CatPhotoRow,
        r#"
        SELECT cp.cat_id,
               p.id AS photo_id,
               p."order",
               p.created_at,
               p.filename,
               p.width,
               p.height,
               p.alt_text
        FROM cats_photos cp
        JOIN photos p ON cp.photo_id = p.id
        ORDER BY cp.cat_id, p."order" ASC NULLS LAST
        "#
    )
    .fetch_all(pool)
    .await?;

    let mut map: HashMap<Uuid, Vec<CatPhoto>> = HashMap::new();

    for row in rows {
        map.entry(row.cat_id).or_default().push(CatPhoto {
            id: row.photo_id,
            order: row.order,
            created_at: row.created_at,
            filename: row.filename,
            width: row.width,
            height: row.height,
            alt_text: row.alt_text,
        });
    }

    Ok(map)
}

pub async fn populate_photos(
    pool: &PgPool,
    cats: &mut [crate::models::cat::Cat],
) -> Result<(), Error> {
    let mut map = get_cat_photos_map(pool).await?;
    for cat in cats.iter_mut() {
        cat.photos = map.remove(&cat.id).unwrap_or_default();
    }
    Ok(())
}

pub async fn with_photos(
    pool: &PgPool,
    cat: Option<crate::models::cat::Cat>,
) -> Result<Option<crate::models::cat::Cat>, Error> {
    let mut cat = cat;
    if let Some(c) = cat.as_mut() {
        let mut v = vec![std::mem::take(c)];
        populate_photos(pool, &mut v).await?;
        *c = v.remove(0);
    }
    Ok(cat)
}
