use crate::hasher;
use crate::models::cat::Cat;
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
    status: Status,
    message: String,
}

#[derive(Serialize)]
pub struct AuthSessionInfo {
    pub session_id: String,
    pub cat: Cat,
}

#[derive(Serialize)]
pub struct AuthResponseWithSessionInfo {
    status: Status,
    message: String,
    results: AuthSessionInfo,
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
    // 1. Fetch user by username
    let row = sqlx::query!(
        "SELECT password FROM cats WHERE username = $1",
        payload.username
    )
    .fetch_optional(&pool)
    .await
    .map_err(|e: sqlx::Error| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(AuthResponse {
                status: Status::Error,
                message: e.to_string(),
            }),
        )
    })?;

    let user_row = match row {
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
    let is_valid = hasher::verify_password(&payload.password, &user_row.password).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(AuthResponse {
                status: Status::Error,
                message: e.to_string(),
            }),
        )
    })?;

    if is_valid {
        Ok((
            StatusCode::OK,
            Json(AuthResponse {
                status: Status::Ok,
                message: "Login successful".to_string(),
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
