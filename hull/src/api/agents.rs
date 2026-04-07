use axum::{extract::Extension, Json};
use std::sync::Arc;

use crate::{
    config::ConfigManager,
    error::{AppError, Result},
    validation,
};

/// 获取 Agent 配置
pub async fn get_agents(
    Extension(config_manager): Extension<Arc<ConfigManager>>,
) -> Result<Json<serde_json::Value>> {
    let agents = config_manager.get_agents().await?;
    Ok(Json(agents))
}

/// 保存 Agent 配置
pub async fn save_agents(
    Extension(config_manager): Extension<Arc<ConfigManager>>,
    Json(data): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>> {
    // 1. 验证 Agent ID 格式
    let validation_result = validation::validate_agents_only(&data);
    if !validation_result.valid || !validation_result.errors.is_empty() {
        let messages: Vec<String> = validation_result
            .errors
            .iter()
            .map(|e| format!("[{}] {}", e.path, e.message))
            .collect();
        tracing::warn!("Agent validation failed: {:?}", messages);
        return Err(AppError::BadRequest(messages.join("; ")));
    }

    // 2. 保存 agents 配置（这会更新内存中的配置并保存到主文件）
    config_manager.save_agents(&data).await?;

    // 3. 读取当前完整配置
    let config = config_manager.get_config().await?;

    // 4. 同步到 version-config/ 并执行 Git 提交
    match config_manager
        .sync_to_version_config(&config, Some("Update agent config".to_string()))
        .await
    {
        Ok(Some(commit_id)) => Ok(Json(serde_json::json!({
            "success": true,
            "commit": commit_id,
        }))),
        Ok(None) => {
            // 没有变更需要提交
            Ok(Json(serde_json::json!({"success": true})))
        }
        Err(e) => {
            tracing::error!("Git sync_to_version_config failed: {}", e);
            Err(e)
        }
    }
}
