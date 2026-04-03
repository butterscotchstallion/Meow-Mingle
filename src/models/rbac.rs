use crate::handlers::common::ApiError;
use sqlx::PgPool;
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

/// Returns `true` if the cat with `cat_id` holds the role identified by `role_slug`.
pub async fn cat_has_role(pool: &PgPool, cat_id: Uuid, role_slug: &str) -> Result<bool, ApiError> {
    sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) > 0 AS "has_role!"
        FROM cats_roles cr
        JOIN roles r ON cr.role_id = r.id
        WHERE cr.cat_id = $1
          AND r.slug = $2
        "#,
        cat_id,
        role_slug,
    )
    .fetch_one(pool)
    .await
    .map_err(ApiError::internal)
}
