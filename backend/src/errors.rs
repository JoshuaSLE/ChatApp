use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;
use thiserror::Error;
use tracing::warn;
use validator::ValidationErrors;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Validation failed")]
    Validation(#[from] ValidationErrors),

    #[error("Cryptography error")]
    Cryptography(#[from] argon2::password_hash::Error),

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Invalid or expired token")]
    InvalidToken,

    #[error("Missing authorization credentials")]
    MissingCredentials,

    #[error("Token creation error")]
    TokenCreation(#[from] jsonwebtoken::errors::Error),

    #[error("User not found")]
    UserNotFound,

    #[error("Bad Request: {0}")]
    BadRequest(String),

    #[error("Database error")]
    Database(#[from] sqlx::Error),

    #[error("Internal server error")]
    Internal,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message): (StatusCode, String) = match self {
            AppError::Validation(e) => {
                warn!("Validation error: {:?}", e);
                let body = Json(json!({ "errors": e.field_errors() }));
                return (StatusCode::BAD_REQUEST, body).into_response();
            }

            AppError::Cryptography(e) => {
                warn!("Hashing error: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error".to_string(),
                )
            }

            AppError::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                "Invalid username or password".to_string(),
            ),

            AppError::InvalidToken => (StatusCode::UNAUTHORIZED, "Invalid token".to_string()),

            AppError::MissingCredentials => {
                (StatusCode::BAD_REQUEST, "Missing credentials".to_string())
            }

            AppError::TokenCreation(e) => {
                tracing::error!("JWT creation failed: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error".to_string(),
                )
            }

            AppError::UserNotFound => (StatusCode::NOT_FOUND, "User not found".to_string()),

            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),

            AppError::Database(e) => match e {
                sqlx::Error::RowNotFound => {
                    warn!("Row not found");
                    (StatusCode::NOT_FOUND, "Resource not found".to_string())
                }
                sqlx::Error::Database(ref db_err) if db_err.is_unique_violation() => {
                    warn!("Unique constraint violation: {:?}", e);
                    (StatusCode::CONFLICT, "Resource already exists".to_string())
                }
                sqlx::Error::Database(ref db_err) if db_err.is_foreign_key_violation() => {
                    warn!("Foreign key violation: {:?}", e);
                    (
                        StatusCode::UNPROCESSABLE_ENTITY,
                        "Related resource not found".to_string(),
                    )
                }
                sqlx::Error::Database(ref db_err) if db_err.is_check_violation() => {
                    warn!("Check constraint violation: {:?}", e);
                    (
                        StatusCode::BAD_REQUEST,
                        "Data failed constraint check".to_string(),
                    )
                }
                sqlx::Error::PoolTimedOut => {
                    warn!("Database pool timed out");
                    (
                        StatusCode::SERVICE_UNAVAILABLE,
                        "Service temporarily unavailable".to_string(),
                    )
                }
                _ => {
                    warn!("Unexpected database error: {:?}", e);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Internal server error".to_string(),
                    )
                }
            },

            AppError::Internal => {
                warn!("Internal server error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error".to_string(),
                )
            }
        };

        let body = Json(json!({ "error": message }));
        (status, body).into_response()
    }
}
