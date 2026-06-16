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

    #[error("Database error")]
    Database(#[from] sqlx::Error),

    #[error("Cryptography error")]
    Cryptography(#[from] argon2::password_hash::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::Validation(e) => {
                warn!("Validation error: {:?}", e);
                let body = Json(json!({ "errors": e.field_errors() }));
                return (StatusCode::BAD_REQUEST, body).into_response();
            }

            AppError::Cryptography(e) => {
                warn!("Hashing error: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
            }

            AppError::Database(e) => match e {
                sqlx::Error::RowNotFound => {
                    warn!("Row not found");
                    (StatusCode::NOT_FOUND, "Resource not found")
                }
                sqlx::Error::Database(ref db_err) if db_err.is_unique_violation() => {
                    warn!("Unique constraint violation: {:?}", e);
                    (StatusCode::CONFLICT, "Resource already exists")
                }
                sqlx::Error::Database(ref db_err) if db_err.is_foreign_key_violation() => {
                    warn!("Foreign key violation: {:?}", e);
                    (
                        StatusCode::UNPROCESSABLE_ENTITY,
                        "Related resource not found",
                    )
                }
                sqlx::Error::Database(ref db_err) if db_err.is_check_violation() => {
                    warn!("Check constraint violation: {:?}", e);
                    (StatusCode::BAD_REQUEST, "Data failed constraint check")
                }
                sqlx::Error::PoolTimedOut => {
                    warn!("Database pool timed out");
                    (
                        StatusCode::SERVICE_UNAVAILABLE,
                        "Service temporarily unavailable",
                    )
                }
                _ => {
                    warn!("Unexpected database error: {:?}", e);
                    (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
                }
            },
        };

        let body = Json(json!({ "error": message }));
        (status, body).into_response()
    }
}
