use crate::hasher;
use crate::models::cat::{Cat, NewCat, get_cat_by_name, update_last_seen};
use crate::models::session::get_or_generate_session_id;
use crate::models::status::Status;
use axum::Json;
use axum::extract::State;
use axum::http::header::SET_COOKIE;
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use serde::Serialize;
use sqlx::PgPool;
use time::Duration;
use time::OffsetDateTime;

pub mod routes {
    pub const AUTH_SIGN_IN: &str = "/api/v1/auth/sign-in";
    pub const AUTH_SIGN_UP: &str = "/api/v1/auth/sign-up";
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct AuthSignInResponse {
    pub(crate) status: Status,
    pub(crate) message: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct AuthSessionInfo {
    pub session_id: String,
    pub cat: Cat,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct AuthResponseWithSessionInfo {
    pub status: String,
    pub message: String,
    pub results: AuthSessionInfo,
}

#[derive(serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct AuthSignInPayload {
    pub name: String,
    pub password: String,
}

#[derive(serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct AuthSignUpPayload {
    pub cat: NewCat,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct AuthSignUpResponseResults {
    pub cat: Cat,
    pub session_id: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct AuthSignUpResponse {
    pub status: Status,
    pub message: String,
    pub results: Option<AuthSignUpResponseResults>,
}

#[axum::debug_handler]
#[utoipa::path(
    post,
    path = routes::AUTH_SIGN_IN,
    request_body = AuthSignInPayload,
    responses(
        (status = 200, description = "Auth sign in", body = AuthSignInResponse),
        (status = 500, description = "Internal server error", body = AuthSignInResponse)
    )
)]
pub async fn sign_in_handler(
    State(pool): State<PgPool>,
    Json(payload): Json<AuthSignInPayload>,
) -> Result<impl IntoResponse, (StatusCode, Json<AuthSignInResponse>)> {
    let invalid_credentials = || {
        Ok((
            StatusCode::OK,
            HeaderMap::new(),
            Json(
                serde_json::to_value(AuthSignInResponse {
                    status: Status::Error,
                    message: "Invalid username or password".to_string(),
                })
                .unwrap(),
            ),
        ))
    };

    let cat_result = get_cat_by_name(&pool, payload.name).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(AuthSignInResponse {
                status: Status::Error,
                message: e.to_string(),
            }),
        )
    })?;

    let cat_row = match cat_result {
        Some(r) => r,
        None => return invalid_credentials(),
    };

    // Verify password
    let is_valid = hasher::verify_password(&payload.password, &cat_row.password).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(AuthSignInResponse {
                status: Status::Error,
                message: e.to_string(),
            }),
        )
    })?;

    if !is_valid {
        return invalid_credentials();
    }

    update_last_seen(&pool, cat_row.id).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(AuthSignInResponse {
                status: Status::Error,
                message: e.to_string(),
            }),
        )
    })?;

    let session_id = get_or_generate_session_id(&pool, cat_row.id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(AuthSignInResponse {
                    status: Status::Error,
                    message: e.to_string(),
                }),
            )
        })?;

    let mut headers = HeaderMap::new();
    let expires = OffsetDateTime::now_utc() + Duration::days(30);
    let expires_str = expires
        .format(&time::format_description::well_known::Rfc2822)
        .unwrap_or_default();

    headers.insert(
        SET_COOKIE,
        format!(
            "{}={}; Path=/; SameSite=None; Secure; Expires={}",
            crate::models::session::SESSION_COOKIE_NAME,
            session_id,
            expires_str
        )
        .parse()
        .unwrap(),
    );

    Ok((
        StatusCode::OK,
        headers,
        Json(
            serde_json::to_value(AuthResponseWithSessionInfo {
                status: String::from("OK"),
                message: "Sign in successful".to_string(),
                results: AuthSessionInfo {
                    session_id: String::from(session_id),
                    cat: cat_row,
                },
            })
            .unwrap(),
        ),
    ))
}

#[axum::debug_handler]
#[utoipa::path(
    post,
    path = routes::AUTH_SIGN_UP,
    request_body = AuthSignUpPayload,
    responses(
        (status = 200, description = "Auth sign up", body = AuthSignUpResponse),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn sign_up_handler(
    State(pool): State<PgPool>,
    Json(payload): Json<AuthSignUpPayload>,
) -> Result<impl IntoResponse, (StatusCode, Json<AuthSignUpResponse>)> {
    let hashed_password = hasher::hash_password(&payload.cat.password).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(AuthSignUpResponse {
                status: Status::Error,
                message: format!("Failed to hash password: {}", e),
                results: None,
            }),
        )
    })?;

    let cat_from_payload = NewCat {
        name: payload.cat.name,
        password: hashed_password,
        birth_date: payload.cat.birth_date,
        breed_id: payload.cat.breed_id,
    };

    let cat = crate::models::cat::add_cat(&pool, cat_from_payload)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(AuthSignUpResponse {
                    status: Status::Error,
                    message: e.to_string(),
                    results: None,
                }),
            )
        })?;

    let session_id = get_or_generate_session_id(&pool, cat.id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(AuthSignUpResponse {
                    status: Status::Error,
                    message: format!("Failed to create session: {}", e),
                    results: None,
                }),
            )
        })?;

    Ok((
        StatusCode::CREATED,
        Json(AuthSignUpResponse {
            status: Status::Ok,
            message: "Cat registered successfully".to_string(),
            results: Option::from(AuthSignUpResponseResults {
                cat,
                session_id: String::from(session_id),
            }),
        }),
    ))
}
