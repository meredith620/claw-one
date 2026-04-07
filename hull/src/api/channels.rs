use axum::{extract::Extension, Json};
use std::sync::Arc;

use crate::{
    config::ConfigManager,
    error::{AppError, Result},
    validation,
};

/// 获取 Channel 配置
pub async fn get_channels(
    Extension(config_manager): Extension<Arc<ConfigManager>>,
) -> Result<Json<serde_json::Value>> {
    let channels = config_manager.get_channels().await?;
    Ok(Json(channels))
}

/// 保存 Channel 配置
pub async fn save_channels(
    Extension(config_manager): Extension<Arc<ConfigManager>>,
    Json(data): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>> {
    // 验证 Channel ID 格式
    let validation_result = validation::validate_channels_only(&data);
    if !validation_result.valid || !validation_result.errors.is_empty() {
        let messages: Vec<String> = validation_result
            .errors
            .iter()
            .map(|e| format!("[{}] {}", e.path, e.message))
            .collect();
        tracing::warn!("Channel validation failed: {:?}", messages);
        return Err(AppError::BadRequest(messages.join("; ")));
    }

    config_manager.save_channels(&data).await?;

    // 读取当前完整配置并同步到 version-config/
    let config = config_manager.get_config().await?;

    match config_manager
        .sync_to_version_config(&config, Some("Update channel config".to_string()))
        .await
    {
        Ok(Some(commit_id)) => Ok(Json(serde_json::json!({
            "success": true,
            "commit": commit_id,
        }))),
        Ok(None) => Ok(Json(serde_json::json!({"success": true}))),
        Err(e) => {
            tracing::error!("save_channels: sync_to_version_config failed: {}", e);
            Err(e)
        }
    }
}
