use crate::domain::DomainError;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use thiserror::Error;

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
    Domain(#[from] DomainError),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code, message) = match self {
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, "not_found", msg),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, "bad_request", msg),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, "conflict", msg),
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, "unauthorized", msg),
            AppError::Domain(DomainError::InvalidTransition { from, to }) => (
                StatusCode::CONFLICT,
                "invalid_transition",
                format!("Cannot transition from {} to {}", from, to),
            ),
            AppError::Domain(DomainError::NotFound(msg)) => {
                (StatusCode::NOT_FOUND, "not_found", msg)
            }
            AppError::Domain(DomainError::ValidationError(msg)) => {
                (StatusCode::BAD_REQUEST, "bad_request", msg)
            }
            AppError::Domain(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "domain_error",
                e.to_string(),
            ),
            AppError::Internal(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal_error",
                "An internal server error occurred".to_string(),
            ),
        };

        let body = Json(serde_json::json!({
            "code": code,
            "message": message,
        }));

        (status, body).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::LicenseStatus;

    #[test]
    fn not_found_maps_to_404() {
        let error = AppError::NotFound("Movie 1".to_string());
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn bad_request_maps_to_400() {
        let error = AppError::BadRequest("Invalid input".to_string());
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn conflict_maps_to_409() {
        let error = AppError::Conflict("Duplicate".to_string());
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::CONFLICT);
    }

    #[test]
    fn unauthorized_maps_to_401() {
        let error = AppError::Unauthorized("Bad token".to_string());
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn domain_invalid_transition_maps_to_409() {
        let error = AppError::Domain(DomainError::InvalidTransition {
            from: LicenseStatus::Draft,
            to: LicenseStatus::Approved,
        });
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::CONFLICT);
    }

    #[test]
    fn domain_validation_error_maps_to_400() {
        let error = AppError::Domain(DomainError::ValidationError("bad".to_string()));
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn domain_not_found_maps_to_404() {
        let error = AppError::Domain(DomainError::NotFound("thing".to_string()));
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn internal_error_maps_to_500() {
        let error = AppError::Internal(anyhow::anyhow!("db connection failed"));
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn error_display_messages() {
        assert_eq!(
            AppError::NotFound("x".into()).to_string(),
            "Resource not found: x"
        );
        assert_eq!(
            AppError::BadRequest("x".into()).to_string(),
            "Invalid request: x"
        );
        assert_eq!(AppError::Conflict("x".into()).to_string(), "Conflict: x");
        assert_eq!(
            AppError::Unauthorized("x".into()).to_string(),
            "Unauthorized: x"
        );
    }
}
