use axum::{extract::Extension, Json};
use std::sync::Arc;

use crate::{config::ConfigManager, error::Result};

/// 检查是否是首次配置
pub async fn check_handler(
    Extension(config_manager): Extension<Arc<ConfigManager>>,
) -> Result<Json<serde_json::Value>> {
    let is_first = config_manager.is_first_setup().await?;

    Ok(Json(serde_json::json!({
        "is_first_setup": is_first,
    })))
}

/// 完成初始化
pub async fn complete_handler(
    Extension(config_manager): Extension<Arc<ConfigManager>>,
) -> Result<Json<serde_json::Value>> {
    config_manager.mark_initialized().await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Initialization completed",
    })))
}

/// 恢复出厂设置
pub async fn reset_handler(
    Extension(config_manager): Extension<Arc<ConfigManager>>,
) -> Result<Json<serde_json::Value>> {
    let config = config_manager.reset_to_factory().await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Reset to factory settings",
        "config": config,
    })))
}
