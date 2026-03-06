use sqlx::types::time::OffsetDateTime;
use sqlx::types::Uuid;
use time::serde::rfc3339;

#[derive(serde::Serialize)]
pub(crate) struct Session {
    pub id: Option<Uuid>,
    pub cat_id: Option<Uuid>,
    #[serde(with = "rfc3339::option")]
    pub created_at: Option<OffsetDateTime>,
    #[serde(with = "rfc3339::option")]
    pub updated_at: Option<OffsetDateTime>,
    pub active: Option<bool>,
}
