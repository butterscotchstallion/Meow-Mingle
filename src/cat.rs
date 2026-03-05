use serde::Serialize;

#[derive(Serialize)]
pub(crate) struct Cat {
    pub username: String,
    pub password: String,
    pub created_at: String,
    pub updated_at: String,
    pub activated: bool,
}
