use crate::adapters::http::router::AppState;
use crate::error::AppError;
use axum::{body::Body, extract::State, http::Request, middleware::Next, response::Response};
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub email: String,
    pub exp: usize,
}

pub fn extract_user_email(auth_header: &str, secret: &str) -> Result<String, AppError> {
    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or_else(|| AppError::Unauthorized("Missing Bearer token".to_string()))?;

    let decoding_key = DecodingKey::from_secret(secret.as_bytes());
    let validation = Validation::new(Algorithm::HS256);

    let token_data = decode::<Claims>(token, &decoding_key, &validation)
        .map_err(|e| AppError::Unauthorized(format!("Invalid token: {}", e)))?;

    Ok(token_data.claims.email)
}

pub async fn auth_middleware(
    State(state): State<Arc<AppState>>,
    req: Request<Body>,
    next: Next,
) -> Result<Response, AppError> {
    let auth_header = req
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| AppError::Unauthorized("Missing Authorization header".to_string()))?;

    let _email = extract_user_email(auth_header, &state.jwt_secret)?;

    Ok(next.run(req).await)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_token(secret: &str, email: &str, exp: usize) -> String {
        let claims = Claims {
            sub: "user123".to_string(),
            email: email.to_string(),
            exp,
        };
        jsonwebtoken::encode(
            &jsonwebtoken::Header::default(),
            &claims,
            &jsonwebtoken::EncodingKey::from_secret(secret.as_bytes()),
        )
        .unwrap()
    }

    #[test]
    fn extract_user_email_valid_token() {
        let secret = "test_secret";
        let exp = chrono::Utc::now().timestamp() as usize + 3600;
        let token = make_token(secret, "user@example.com", exp);
        let auth_header = format!("Bearer {}", token);

        let result = extract_user_email(&auth_header, secret);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "user@example.com");
    }

    #[test]
    fn extract_user_email_missing_bearer_prefix() {
        let result = extract_user_email("SomeToken", "secret");
        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::Unauthorized(msg) => assert!(msg.contains("Missing Bearer")),
            _ => panic!("Expected Unauthorized"),
        }
    }

    #[test]
    fn extract_user_email_invalid_token() {
        let result = extract_user_email("Bearer invalidtoken", "secret");
        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::Unauthorized(msg) => assert!(msg.contains("Invalid token")),
            _ => panic!("Expected Unauthorized"),
        }
    }

    #[test]
    fn extract_user_email_wrong_secret() {
        let secret = "correct_secret";
        let exp = chrono::Utc::now().timestamp() as usize + 3600;
        let token = make_token(secret, "user@example.com", exp);
        let auth_header = format!("Bearer {}", token);

        let result = extract_user_email(&auth_header, "wrong_secret");
        assert!(result.is_err());
    }

    #[test]
    fn extract_user_email_expired_token() {
        let secret = "test_secret";
        let exp = 1; // Already expired
        let token = make_token(secret, "user@example.com", exp);
        let auth_header = format!("Bearer {}", token);

        let result = extract_user_email(&auth_header, secret);
        assert!(result.is_err());
    }

    #[test]
    fn extract_user_email_empty_bearer() {
        let result = extract_user_email("Bearer ", "secret");
        assert!(result.is_err());
    }
}
