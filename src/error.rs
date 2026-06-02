use actix_session::{SessionGetError, SessionInsertError};
use actix_web::{HttpResponse, ResponseError};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Not found")]
    NotFound,

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Session error: {0}")]
    Session(String),
}

impl From<SessionInsertError> for ApiError {
    fn from(err: SessionInsertError) -> Self {
        ApiError::Session(err.to_string())
    }
}

impl From<SessionGetError> for ApiError {
    fn from(err: SessionGetError) -> Self {
        ApiError::Session(err.to_string())
    }
}

impl ResponseError for ApiError {
    fn error_response(&self) -> HttpResponse {
        match self {
            ApiError::Database(_) => HttpResponse::InternalServerError().json("Internal error"),
            ApiError::NotFound => HttpResponse::NotFound().json("Resource not found"),
            ApiError::BadRequest(msg) => HttpResponse::BadRequest().json(msg),
            ApiError::Unauthorized => HttpResponse::Unauthorized().json("Unauthorized"),
            ApiError::Session(_) => HttpResponse::InternalServerError().json("Session error"),
        }
    }
}