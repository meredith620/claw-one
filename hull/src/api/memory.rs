use axum::{extract::Extension, Json};
use std::sync::Arc;

use crate::{config::ConfigManager, error::Result};

/// 获取 Memory 配置
/// 如果配置不存在，返回 null (serde_json::Value::Null)
pub async fn get_memory(
    Extension(config_manager): Extension<Arc<ConfigManager>>,
) -> Result<Json<serde_json::Value>> {
    let memory = config_manager.get_memory().await?;
    Ok(Json(memory.unwrap_or(serde_json::Value::Null)))
}

/// 保存 Memory 配置
pub async fn save_memory(
    Extension(config_manager): Extension<Arc<ConfigManager>>,
    Json(data): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>> {
    config_manager.save_memory(&data).await?;

    // 读取当前完整配置并同步到 version-config/
    let config = config_manager.get_config().await?;

    match config_manager
        .sync_to_version_config(&config, Some("Update memory config".to_string()))
        .await
    {
        Ok(Some(commit_id)) => Ok(Json(serde_json::json!({
            "success": true,
            "commit": commit_id,
        }))),
        Ok(None) => Ok(Json(serde_json::json!({"success": true}))),
        Err(e) => {
            tracing::error!("save_memory: sync_to_version_config failed: {}", e);
            Err(e)
        }
    }
}
