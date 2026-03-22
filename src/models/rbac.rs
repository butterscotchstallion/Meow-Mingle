use sqlx::types::Uuid;

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, utoipa::ToSchema)]
pub struct Role {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, utoipa::ToSchema)]
pub struct CatsRoles {
    #[serde(rename = "catId")]
    pub cat_id: Uuid,
    #[serde(rename = "roleId")]
    pub role_id: Uuid,
}
