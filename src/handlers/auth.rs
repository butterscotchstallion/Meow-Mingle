use crate::hasher;
use crate::models::cat::{get_cat_by_username, Cat};
use crate::models::session::get_or_generate_session_id;
use crate::models::status::Status;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

#[derive(Deserialize)]
pub struct AuthPayload {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct AuthResponse {
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

#[derive(serde::Serialize, serde::Deserialize)]
pub struct AuthLoginPayload {
    pub username: String,
    pub password: String,
}

pub mod routes {
    pub const AUTH_LOGIN: &str = "/auth/login";
}

pub async fn login_handler(
    State(pool): State<PgPool>,
    Json(payload): Json<AuthPayload>,
) -> Result<impl IntoResponse, (StatusCode, Json<AuthResponse>)> {
    let cat_result = get_cat_by_username(&pool, payload.username)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(AuthResponse {
                    status: Status::Error,
                    message: e.to_string(),
                }),
            )
        })?;
    let cat_row = match cat_result {
        Some(r) => r,
        None => {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(AuthResponse {
                    status: Status::Error,
                    message: "Invalid username or password".to_string(),
                }),
            ));
        }
    };

    // 2. Verify password
    let is_valid = hasher::verify_password(&payload.password, &cat_row.password).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(AuthResponse {
                status: Status::Error,
                message: e.to_string(),
            }),
        )
    })?;

    if is_valid {
        let session_id = get_or_generate_session_id(&pool, cat_row.id)
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(AuthResponse {
                        status: Status::Error,
                        message: e.to_string(),
                    }),
                )
            })?;
        Ok((
            StatusCode::OK,
            Json(AuthResponseWithSessionInfo {
                status: String::from("OK"),
                message: "Login successful".to_string(),
                results: AuthSessionInfo {
                    session_id: String::from(session_id),
                    cat: cat_row,
                },
            }),
        ))
    } else {
        Err((
            StatusCode::UNAUTHORIZED,
            Json(AuthResponse {
                status: Status::Error,
                message: "Invalid username or password".to_string(),
            }),
        ))
    }
}
