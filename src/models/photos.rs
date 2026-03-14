use sqlx::{Error, PgPool};
use sqlx::types::Uuid;
use sqlx::types::time::OffsetDateTime;
use std::collections::HashMap;
use time::serde::rfc3339;

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, utoipa::ToSchema)]
pub struct CatPhoto {
    pub id: Uuid,
    pub order: Option<i32>,
    #[serde(with = "rfc3339::option", rename = "createdAt")]
    pub created_at: Option<OffsetDateTime>,
}

#[derive(Debug, sqlx::FromRow)]
pub struct CatPhotoRow {
    pub cat_id: Uuid,
    pub photo_id: Uuid,
    pub order: Option<i32>,
    pub created_at: Option<OffsetDateTime>,
}

pub async fn get_cat_photos_map(
    pool: &PgPool,
) -> Result<HashMap<Uuid, Vec<CatPhoto>>, Error> {
    let rows = sqlx::query_as!(
        CatPhotoRow,
        r#"
        SELECT cp.cat_id,
               p.id AS photo_id,
               p."order",
               p.created_at
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
