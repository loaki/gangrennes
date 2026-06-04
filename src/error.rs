use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("database error")]
    Database(#[from] sqlx::Error),
    #[error("template error")]
    Template(#[from] askama::Error),
    #[error("bad request: {0}")]
    BadRequest(String),
    #[error("internal server error")]
    Internal,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            Self::BadRequest(message) => (StatusCode::BAD_REQUEST, message).into_response(),
            Self::Internal => {
                (StatusCode::INTERNAL_SERVER_ERROR, "internal server error").into_response()
            }
            other => {
                tracing::error!(error = %other, "request failed");
                (StatusCode::INTERNAL_SERVER_ERROR, "internal server error").into_response()
            }
        }
    }
}
