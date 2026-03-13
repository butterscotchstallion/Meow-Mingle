use crate::models::status::Status;
use axum::http::StatusCode;
use axum::Json;
use serde_json::{json, Value};

pub fn get_unauthorized_response() -> (StatusCode, Json<Value>) {
    (
        StatusCode::UNAUTHORIZED,
        Json(json!({
            "status": Status::Error,
            "message": "You are not logged in or your account is inactive"
        })),
    )
}
