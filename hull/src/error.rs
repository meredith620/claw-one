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
    
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Bad request: {0}")]
    BadRequest(String),
    
    #[error("Runtime error: {0}")]
    Runtime(String),
    
    #[error("Git error: {0}")]
    Git(String),
    
    #[error("Internal error: {0}")]
    Internal(String),
}

impl AppError {
    /// 获取详细的错误信息，包括源代码位置
    pub fn detailed_message(&self) -> String {
        format!("{} [type: {:?}]", self, std::mem::discriminant(self))
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        // 记录详细错误信息
        let error_detail = format!("{:#?}", self);
        tracing::error!("Handler error: {}\nDetail: {}", self, error_detail);
        
        let (status, message) = match &self {
            AppError::ConfigNotFound => (StatusCode::NOT_FOUND, self.to_string()),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            AppError::Runtime(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            AppError::Git(msg) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Git error: {}", msg)),
            AppError::Io(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("IO error: {}", e)),
            AppError::Json(e) => (StatusCode::BAD_REQUEST, format!("JSON error: {}", e)),
            AppError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Internal error: {}", msg)),
        };

        // 确保错误消息不为空
        let message = if message.is_empty() {
            format!("Unknown error: {:?}", self)
        } else {
            message
        };

        let body = Json(json!({
            "error": message,
            "error_type": format!("{:?}", std::mem::discriminant(&self)),
        }));

        (status, body).into_response()
    }
}
