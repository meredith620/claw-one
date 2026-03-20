use axum::{
    extract::Extension,
    Json,
};
use std::sync::Arc;

use crate::{
    config::ConfigManager,
    error::Result,
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
    // 1. 保存 agents 配置（这会更新内存中的配置并保存到主文件）
    config_manager.save_agents(&data).await?;
    
    // 2. 读取当前完整配置
    let config = config_manager.get_config().await?;
    
    // 3. 同步到 version-config/ 并执行 Git 提交
    match config_manager.sync_to_version_config(&config, Some("Update agent config".to_string())).await {
        Ok(Some(commit_id)) => {
            return Ok(Json(serde_json::json!({
                "success": true,
                "commit": commit_id,
            })));
        }
        Ok(None) => {
            // 没有变更需要提交
            return Ok(Json(serde_json::json!({"success": true})));
        }
        Err(e) => {
            tracing::error!("Git sync_to_version_config failed: {}", e);
            return Err(e);
        }
    }
}
