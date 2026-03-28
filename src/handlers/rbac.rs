use crate::handlers::common::ApiError;
use crate::models::rbac::Role;
use crate::models::session::get_cat_from_session_id;
use crate::models::status::Status;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use axum_cookie::CookieManager;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

pub mod routes {
    pub const CAT_ROLE_LIST: &str = "/api/v1/roles";
}

#[derive(Serialize, Deserialize, utoipa::ToSchema)]
pub struct CatRoleListResponse {
    pub status: Status,
    pub results: Vec<Role>,
}

#[utoipa::path(
    get,
    path = routes::CAT_ROLE_LIST,
    responses(
        (status = 200, description = "List of roles for current cat", body = CatRoleListResponse),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    tag = "roles"
)]
pub async fn cat_roles_list_handler(
    State(pool): State<PgPool>,
    cookie_manager: CookieManager,
) -> Result<(StatusCode, Json<CatRoleListResponse>), ApiError> {
    let cat = match get_cat_from_session_id(&pool, cookie_manager).await {
        Ok(Some(cat)) => cat,
        _ => return Err(ApiError::unauthorized()),
    };
    let roles: Vec<Role> = sqlx::query_as!(
        Role,
        r#"
        SELECT r.id AS "id!", r.name AS "name!", r.slug
        FROM roles r
        JOIN cats_roles cr ON cr.role_id = r.id
        WHERE cr.cat_id = $1
        ORDER BY r.name ASC
        "#,
        cat.id
    )
    .fetch_all(&pool)
    .await
    .map_err(ApiError::internal)?;

    Ok((
        StatusCode::OK,
        Json(CatRoleListResponse {
            status: Status::Ok,
            results: roles,
        }),
    ))
}
