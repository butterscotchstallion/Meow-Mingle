use crate::models::cat::{Cat, CatRow};
use crate::models::interests::with_interests;
use axum_cookie::CookieManager;
use sqlx::types::time::OffsetDateTime;
use time::serde::rfc3339;
use uuid::Uuid;

pub const SESSION_COOKIE_NAME: &str = "MM_SESSION_ID";

#[allow(dead_code)]
#[derive(serde::Serialize)]
pub(crate) struct Session {
    pub id: Option<Uuid>,
    #[serde(rename = "sessionId")]
    pub session_id: Uuid,
    #[serde(rename = "catId")]
    pub cat_id: Option<Uuid>,
    #[serde(with = "rfc3339::option", rename = "createdAt")]
    pub created_at: Option<OffsetDateTime>,
    #[serde(with = "rfc3339::option", rename = "updatedAt")]
    pub updated_at: Option<OffsetDateTime>,
    pub active: Option<bool>,
}

fn generate_session_id() -> Uuid {
    Uuid::new_v4()
}

pub async fn get_or_generate_session_id(
    pool: &sqlx::PgPool,
    cat_id: Uuid,
) -> Result<Uuid, sqlx::Error> {
    let new_session_id = generate_session_id();
    let row = sqlx::query!(
        r#"
        INSERT INTO sessions (cat_id, session_id, created_at, updated_at, active)
        VALUES ($1, $2, NOW(), NOW(), true)
        ON CONFLICT (cat_id)
        DO UPDATE SET updated_at = NOW()
        RETURNING session_id
        "#,
        cat_id,
        new_session_id,
    )
    .fetch_one(pool)
    .await?;
    Ok(row.session_id)
}

pub fn get_session_id_from_cookie(cookie_manager: CookieManager) -> Option<Uuid> {
    let session_id = cookie_manager
        .get(SESSION_COOKIE_NAME)
        .and_then(|c| Uuid::parse_str(c.value()).ok());

    tracing::debug!("Session id from cookie: {:?}", session_id);

    session_id
}

pub async fn get_cat_from_session_id(
    pool: &sqlx::PgPool,
    cookie_manager: CookieManager,
) -> Result<Option<Cat>, sqlx::Error> {
    let session_id = match get_session_id_from_cookie(cookie_manager) {
        Some(id) => id,
        None => return Ok(None),
    };
    tracing::debug!("Getting cat from session id: {:?}", session_id);
    let row = sqlx::query_as!(
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
        JOIN sessions s ON c.id = s.cat_id
        WHERE s.session_id = $1
        AND s.active = true
        "#,
        session_id
    )
    .fetch_optional(pool)
    .await?;

    with_interests(pool, row).await
}
