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

pub async fn delete_existing_photos(
    pool: &PgPool,
    cat_id: Uuid,
) -> Result<Vec<String>, sqlx::Error> {
    delete_removed_photos(pool, cat_id, &[]).await
}

/// Deletes only the photos for `cat_id` whose IDs are NOT in `kept_ids`.
/// Returns the filenames of deleted photos so the caller can remove them from disk.
pub async fn delete_removed_photos(
    pool: &PgPool,
    cat_id: Uuid,
    kept_ids: &[Uuid],
) -> Result<Vec<String>, sqlx::Error> {
    // Fetch filenames of photos that will be deleted (not in kept_ids)
    let rows = sqlx::query!(
        r#"
        SELECT p.id, p.filename
        FROM cats_photos cp
        JOIN photos p ON cp.photo_id = p.id
        WHERE cp.cat_id = $1
          AND p.id != ALL($2::uuid[])
        "#,
        cat_id,
        kept_ids as &[Uuid],
    )
    .fetch_all(pool)
    .await?;

    let filenames: Vec<String> = rows.iter().map(|r| r.filename.clone()).collect();
    let removed_ids: Vec<Uuid> = rows.into_iter().map(|r| r.id).collect();

    if removed_ids.is_empty() {
        return Ok(filenames);
    }

    // Delete the join rows for removed photos
    sqlx::query!(
        r#"DELETE FROM cats_photos WHERE cat_id = $1 AND photo_id = ANY($2::uuid[])"#,
        cat_id,
        removed_ids.as_slice() as &[Uuid],
    )
    .execute(pool)
    .await?;

    // Delete the photo rows themselves
    sqlx::query!(
        r#"DELETE FROM photos WHERE id = ANY($1::uuid[])"#,
        removed_ids.as_slice() as &[Uuid],
    )
    .execute(pool)
    .await?;

    Ok(filenames)
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
