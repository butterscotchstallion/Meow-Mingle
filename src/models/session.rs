use sqlx::types::time::OffsetDateTime;
use time::serde::rfc3339;
use uuid::Uuid;

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
    let session_id = generate_session_id();
    sqlx::query(
        r#"
        INSERT INTO sessions (cat_id, session_id, created_at, updated_at)
		VALUES ($1, $2, NOW(), NOW())
		ON CONFLICT(cat_id)
	DO UPDATE SET session_id = $2, updated_at = NOW()
    "#,
    )
    .bind(cat_id)
    .bind(session_id)
    .execute(pool)
    .await?
    .rows_affected();
    Ok(session_id)
}
