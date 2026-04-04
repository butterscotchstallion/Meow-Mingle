use crate::models::status::Status;
use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct GenericResponse {
    pub status: Status,
    pub message: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ApiErrorBody {
    pub status: Status,
    pub message: String,
}

#[derive(Debug)]
pub struct ApiError {
    pub status_code: StatusCode,
    pub body: ApiErrorBody,
}

impl ApiError {
    pub fn new(status_code: StatusCode, message: impl Into<String>) -> Self {
        Self {
            status_code,
            body: ApiErrorBody {
                status: Status::Error,
                message: message.into(),
            },
        }
    }

    pub fn unauthorized() -> Self {
        Self::new(
            StatusCode::UNAUTHORIZED,
            "You are not logged in or your account is inactive",
        )
    }

    pub fn forbidden() -> Self {
        Self::new(
            StatusCode::FORBIDDEN,
            "You do not have permission to perform this action",
        )
    }

    pub fn internal(e: impl std::fmt::Display) -> Self {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
    }

    pub fn not_found() -> Self {
        Self::new(StatusCode::NOT_FOUND, "Not found")
    }

    pub fn bad_request() -> Self {
        Self::new(StatusCode::BAD_REQUEST, "Bad request")
    }

    pub fn unprocessable_request(e: impl std::fmt::Display) -> Self {
        Self::new(StatusCode::UNPROCESSABLE_ENTITY, e.to_string())
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.status_code, Json(self.body)).into_response()
    }
}
