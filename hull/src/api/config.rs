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

/// 灵活的配置请求，支持两种格式：
/// 1. 标准格式: `{ "config": {...}, "message": "..." }`
/// 2. 直接格式: `{ ...配置内容... }` (自动检测，将整个对象作为配置)
#[derive(serde::Deserialize)]
#[serde(untagged)]
pub enum FlexibleConfigRequest {
    /// 标准格式，包含 config 和 message 字段
    Standard {
        config: Config,
        #[serde(default)]
        message: Option<String>,
    },
    /// 直接格式，整个对象就是配置
    Direct(Config),
}

pub async fn post_handler(
    Extension(state_manager): Extension<Arc<StateManager>>,
    Json(request): Json<FlexibleConfigRequest>,
) -> Result<Json<serde_json::Value>> {
    // 根据请求格式提取配置和消息
    let (config, message) = match request {
        FlexibleConfigRequest::Standard { config, message } => (config, message),
        FlexibleConfigRequest::Direct(config) => (config, None),
    };

    // 使用 StateManager 进行事务性配置应用
    state_manager.apply_config(config, message).await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Config applied successfully",
    })))
}

/// 验证配置请求 - 支持两种格式：
/// 1. 标准格式: `{ "config": {...} }`
/// 2. 直接格式: `{ ...配置内容... }` (自动检测)
#[derive(serde::Deserialize)]
#[serde(untagged)]
pub enum FlexibleValidateRequest {
    /// 标准格式，包含 config 字段
    Standard { config: Config },
    /// 直接格式，整个对象就是配置
    Direct(Config),
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
    Json(request): Json<FlexibleValidateRequest>,
) -> Result<Json<ValidateConfigResponse>> {
    // 根据请求格式提取配置
    let config = match request {
        FlexibleValidateRequest::Standard { config } => config,
        FlexibleValidateRequest::Direct(config) => config,
    };
    
    let result = validate_config(&config);
    
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
pub async fn save_module_handler(
    Path(module): Path<String>,
    Extension(config_manager): Extension<Arc<ConfigManager>>,
    Json(module_config): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>> {
    // 获取当前完整配置
    let mut full_config = config_manager.get_config().await?;
    
    // 根据模块名更新对应的配置
    match module.as_str() {
        "tools" => {
            full_config["tools"] = module_config;
        }
        "session" => {
            full_config["session"] = module_config;
        }
        "agents" => {
            full_config["agents"] = module_config;
        }
        "models" => {
            full_config["models"] = module_config;
        }
        "channels" => {
            full_config["channels"] = module_config;
        }
        _ => {
            return Ok(Json(serde_json::json!({
                "success": false,
                "message": format!("Unknown module: {}", module),
            })));
        }
    }
    
    // 保存配置
    config_manager.save_config(&full_config).await?;
    
    // 创建 Git 提交（如果没有变更则不提交）
    config_manager.ensure_git_repo().await?;
    config_manager.git_add(".").await?;
    if config_manager.has_changes().await? {
        let commit_id = config_manager.git_commit(&format!("Update {} module", module)).await?;
        return Ok(Json(serde_json::json!({
            "success": true,
            "commit": commit_id,
        })));
    }
    
    Ok(Json(serde_json::json!({
        "success": true,
        "message": format!("{} module saved successfully", module),
    })))
}