use axum::{
    extract::{Extension, Path},
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

/// 获取指定模块的配置
pub async fn get_module_handler(
    Path(module): Path<String>,
    Extension(config_manager): Extension<Arc<ConfigManager>>,
) -> Result<Json<serde_json::Value>> {
    let config = config_manager.get_config().await?;
    
    // 根据模块名返回对应的配置
    let module_config = match module.as_str() {
        "tools" => config.get("tools").cloned().unwrap_or(serde_json::json!({})),
        "session" => config.get("session").cloned().unwrap_or(serde_json::json!({})),
        "agents" => config.get("agents").cloned().unwrap_or(serde_json::json!({})),
        "models" => config.get("models").cloned().unwrap_or(serde_json::json!({})),
        "channels" => config.get("channels").cloned().unwrap_or(serde_json::json!({})),
        _ => serde_json::json!({}),
    };
    
    Ok(Json(module_config))
}

/// 保存指定模块的配置
#[derive(serde::Deserialize)]
pub struct SaveModuleRequest {
    pub config: serde_json::Value,
}

pub async fn save_module_handler(
    Path(module): Path<String>,
    Extension(config_manager): Extension<Arc<ConfigManager>>,
    Extension(state_manager): Extension<Arc<StateManager>>,
    Json(request): Json<SaveModuleRequest>,
) -> Result<Json<serde_json::Value>> {
    // 获取当前完整配置
    let mut full_config = config_manager.get_config().await?;
    
    // 根据模块名更新对应的配置
    match module.as_str() {
        "tools" => {
            full_config["tools"] = request.config;
        }
        "session" => {
            full_config["session"] = request.config;
        }
        "agents" => {
            full_config["agents"] = request.config;
        }
        "models" => {
            full_config["models"] = request.config;
        }
        "channels" => {
            full_config["channels"] = request.config;
        }
        _ => {
            return Ok(Json(serde_json::json!({
                "success": false,
                "message": format!("Unknown module: {}", module),
            })));
        }
    }
    
    // 应用完整配置
    state_manager.apply_config(full_config, Some(format!("Update {} module", module))).await?;
    
    Ok(Json(serde_json::json!({
        "success": true,
        "message": format!("{} module saved successfully", module),
    })))
}