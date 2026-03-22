use crate::common::auth_helpers::get_session_id_and_verify;
use axum::http::StatusCode;
use common::helpers::get_server;
use cookie::Cookie;
use meow_mingle::config::{AppConfig, load_config};
use meow_mingle::handlers::rbac::{CatRoleListResponse, routes};
use meow_mingle::models::status::Status;
mod common;

#[tokio::test]
async fn test_roles_list() {
    let server = get_server().await;
    let config: AppConfig = load_config();
    let admin_session_id = get_session_id_and_verify(
        config.test_users.admin_username,
        config.test_users.admin_password,
    )
    .await;

    let response = server
        .get(routes::CAT_ROLE_LIST)
        .add_cookie(Cookie::new(
            meow_mingle::models::session::SESSION_COOKIE_NAME,
            admin_session_id,
        ))
        .await;

    response.assert_status(StatusCode::OK);

    let body = response.json::<CatRoleListResponse>();

    response.assert_status(StatusCode::OK);
    assert_eq!(body.status, Status::Ok);
    assert!(body.results.len() > 0);

    let mut admin_role_found = false;
    for role in body.results {
        if role.slug == "cat-admin" {
            admin_role_found = true;
            break;
        }
    }

    assert!(admin_role_found);
}
