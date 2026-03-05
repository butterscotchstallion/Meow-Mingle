use sqlx::types::time::OffsetDateTime;
use sqlx::types::Uuid;
use time::serde::rfc3339;

#[derive(serde::Serialize)]
pub(crate) struct Cat {
    pub id: Uuid,
    pub username: String,
    #[serde(with = "rfc3339::option")]
    pub created_at: Option<OffsetDateTime>,
    #[serde(with = "rfc3339::option")]
    pub updated_at: Option<OffsetDateTime>,
    pub activated: Option<bool>,
    pub avatar_filename: Option<String>,
}
