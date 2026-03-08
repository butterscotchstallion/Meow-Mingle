use crate::hasher;
use crate::models::cat::{get_cat_by_name, Cat};
use crate::models::session::get_or_generate_session_id;
use crate::models::status::Status;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

pub mod routes {
    pub const AUTH_SIGN_IN: &str = "/auth/sign-in";
}

#[derive(Deserialize)]
pub struct AuthPayload {
    pub name: String,
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
pub struct AuthSignInPayload {
    pub name: String,
    pub password: String,
}

pub async fn sign_in_handler(
    State(pool): State<PgPool>,
    Json(payload): Json<AuthPayload>,
) -> Result<impl IntoResponse, (StatusCode, Json<AuthResponse>)> {
    let cat_result = get_cat_by_name(&pool, payload.name).await.map_err(|e| {
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
                message: "Sign in successful".to_string(),
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
