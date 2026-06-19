use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use thiserror::Error;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("resource not found: {0}")]
    NotFound(String),
    #[error("conflict: {0}")]
    Conflict(String),
    #[error("unauthorized: {0}")]
    Unauthorized(String),
    #[error("validation failed: {0}")]
    Validation(String),
    #[error("configuration error: {0}")]
    Config(String),
    #[error("internal error: {0}")]
    Internal(String),
    #[error(transparent)]
    Database(#[from] sqlx::Error),
    #[error(transparent)]
    Migration(#[from] sqlx::migrate::MigrateError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: &'static str,
    message: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code, message) = match self {
            Self::NotFound(message) => (StatusCode::NOT_FOUND, "not_found", message),
            Self::Conflict(message) => (StatusCode::CONFLICT, "conflict", message),
            Self::Unauthorized(message) => (StatusCode::UNAUTHORIZED, "unauthorized", message),
            Self::Validation(message) => (StatusCode::BAD_REQUEST, "validation_error", message),
            Self::Config(message) => (StatusCode::INTERNAL_SERVER_ERROR, "config_error", message),
            Self::Internal(message) => {
                tracing::error!(message = %message, "internal application error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal_error",
                    "internal server error".to_owned(),
                )
            }
            Self::Database(error) => {
                tracing::error!(error = %error, "database operation failed");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "database_error",
                    "database operation failed".to_owned(),
                )
            }
            Self::Migration(error) => {
                tracing::error!(error = %error, "database migration failed");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "migration_error",
                    "database migration failed".to_owned(),
                )
            }
            Self::Io(error) => {
                tracing::error!(error = %error, "i/o operation failed");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "io_error",
                    "i/o operation failed".to_owned(),
                )
            }
        };

        (status, Json(ErrorResponse { error: code, message })).into_response()
    }
}