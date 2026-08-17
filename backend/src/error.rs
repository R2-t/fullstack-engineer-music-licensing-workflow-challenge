use thiserror::Error;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Invalid request: {0}")]
    BadRequest(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Internal server error")]
    Internal(#[from] anyhow::Error),

    #[error("Domain error: {0}")]
    Domain(#[from] crate::domain::DomainError),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code, message) = match self {
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, "not_found", msg),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, "bad_request", msg),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, "conflict", msg),
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, "unauthorized", msg),
            AppError::Domain(crate::domain::DomainError::InvalidTransition { from, to }) => {
                (StatusCode::CONFLICT, "invalid_transition", format!("Cannot transition from {} to {}", from, to))
            },
            AppError::Domain(e) => (StatusCode::BAD_REQUEST, "domain_error", e.to_string()),
            AppError::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, "internal_error", "An internal server error occurred".to_string()),
        };

        let body = Json(serde_json::json!({
            "code": code,
            "message": message,
        }));

        (status, body).into_response()
    }
}
