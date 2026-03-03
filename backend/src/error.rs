use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, AppError>;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    
    #[error("Config not found")]
    ConfigNotFound,
    
    #[error("Runtime error: {0}")]
    Runtime(String),
    
    #[error("Git error: {0}")]
    Git(String),
    
    #[error("Internal error: {0}")]
    Internal(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::ConfigNotFound => (StatusCode::NOT_FOUND, self.to_string()),
            AppError::Runtime(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            AppError::Git(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };

        let body = Json(json!({
            "error": message,
        }));

        (status, body).into_response()
    }
}
