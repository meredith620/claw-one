use axum::{
    extract::Extension,
    Json,
};
use std::sync::Arc;

use crate::{
    config::ConfigManager,
    error::Result,
    state::StateManager,
    types::{ApplyConfigRequest, Config},
    validation::validate_config,
};

pub async fn get_handler(
    Extension(config_manager): Extension<Arc<ConfigManager>>,
) -> Result<Json<Config>> {
    let config = config_manager.get_config().await?;
    Ok(Json(config))
}

pub async fn post_handler(
    Extension(state_manager): Extension<Arc<StateManager>>,
    Json(request): Json<ApplyConfigRequest>,
) -> Result<Json<serde_json::Value>> {
    // 使用 StateManager 进行事务性配置应用
    state_manager.apply_config(request.config, request.message).await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Config applied successfully",
    })))
}

/// 验证配置请求
#[derive(serde::Deserialize)]
pub struct ValidateConfigRequest {
    pub config: Config,
}

/// 验证配置响应
#[derive(serde::Serialize)]
pub struct ValidateConfigResponse {
    pub valid: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationError>,
}

#[derive(serde::Serialize)]
pub struct ValidationError {
    pub path: String,
    pub message: String,
}

/// 验证配置处理函数
pub async fn validate_handler(
    Json(request): Json<ValidateConfigRequest>,
) -> Result<Json<ValidateConfigResponse>> {
    let result = validate_config(&request.config);
    
    let errors: Vec<ValidationError> = result
        .errors
        .into_iter()
        .map(|e| ValidationError {
            path: e.path,
            message: e.message,
        })
        .collect();
    
    let warnings: Vec<ValidationError> = result
        .warnings
        .into_iter()
        .map(|e| ValidationError {
            path: e.path,
            message: e.message,
        })
        .collect();
    
    Ok(Json(ValidateConfigResponse {
        valid: result.valid && errors.is_empty(),
        errors,
        warnings,
    }))
}