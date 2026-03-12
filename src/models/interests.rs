use crate::models::cat::{Cat, CatRow};
use sqlx::{Error, PgPool};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, sqlx::FromRow)]
pub struct CatInterest {
    pub cat_id: Uuid,
    pub interest_id: Uuid,
    pub interest_name: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, utoipa::ToSchema)]
pub struct Interest {
    pub id: Uuid,
    pub name: String,
}

pub async fn get_cat_interests_map(
    pool: &PgPool,
) -> Result<HashMap<Uuid, Vec<Interest>>, sqlx::Error> {
    let rows = sqlx::query_as!(
        CatInterest,
        r#"
        SELECT ci.cat_id, i.id AS interest_id, i.name AS interest_name
        FROM cats_interests ci
        JOIN interests i ON ci.interest_id = i.id
        "#
    )
    .fetch_all(pool)
    .await?;

    let mut map: HashMap<Uuid, Vec<Interest>> = HashMap::new();

    for row in rows {
        map.entry(row.cat_id).or_default().push(Interest {
            id: row.interest_id,
            name: row.interest_name,
        });
    }

    Ok(map)
}

pub async fn populate_interests(
    pool: &PgPool,
    cats: &mut [crate::models::cat::Cat],
) -> Result<(), sqlx::Error> {
    let mut map = get_cat_interests_map(pool).await?;
    for cat in cats.iter_mut() {
        cat.interests = map.remove(&cat.id).unwrap_or_default();
    }
    Ok(())
}

pub async fn with_interests(
    pool: &sqlx::PgPool,
    row: Option<CatRow>,
) -> Result<Option<Cat>, Error> {
    let mut cat = row.map(Cat::from);
    if let Some(c) = cat.as_mut() {
        let mut v = vec![std::mem::take(c)];
        populate_interests(pool, &mut v).await?;
        *c = v.remove(0);
    }
    Ok(cat)
}
