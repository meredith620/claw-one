use axum::{
    extract::{Extension, Path},
    Json,
};
use std::sync::Arc;

use crate::{
    config::ConfigManager,
    error::{AppError, Result},
    types::Config,
};

/// 获取所有 Provider 实例
pub async fn list_providers(
    Extension(config_manager): Extension<Arc<ConfigManager>>,
) -> Result<Json<Vec<serde_json::Value>>> {
    let providers = config_manager.get_providers().await?;
    Ok(Json(providers))
}

/// 获取单个 Provider 实例
pub async fn get_provider(
    Extension(config_manager): Extension<Arc<ConfigManager>>,
    Path(provider_id): Path<String>,
) -> Result<Json<serde_json::Value>> {
    let providers = config_manager.get_providers().await?;
    let provider = providers
        .into_iter()
        .find(|p| p.get("id").and_then(|id| id.as_str()) == Some(&provider_id))
        .ok_or_else(|| AppError::NotFound(format!("Provider {} not found", provider_id)))?;
    Ok(Json(provider))
}

/// 创建/更新 Provider 实例
pub async fn save_provider(
    Extension(config_manager): Extension<Arc<ConfigManager>>,
    Path(provider_id): Path<String>,
    Json(data): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>> {
    // 检查 Provider ID 是否冲突（如果是创建新实例）
    let providers = config_manager.get_providers().await?;
    let exists = providers
        .iter()
        .any(|p| p.get("id").and_then(|id| id.as_str()) == Some(&provider_id));
    
    let is_update = data.get("_update").and_then(|v| v.as_bool()).unwrap_or(false);
    
    if exists && !is_update {
        return Err(AppError::BadRequest(format!(
            "Provider ID '{}' already exists",
            provider_id
        )));
    }
    
    // 保存 Provider
    config_manager.save_provider(&provider_id, &data).await?;
    
    // 创建 Git 提交
    let commit_msg = if is_update {
        format!("Update provider: {}", provider_id)
    } else {
        format!("Add provider: {}", provider_id)
    };
    
    config_manager.ensure_git_repo().await?;
    config_manager.git_add(".").await?;
    if config_manager.has_changes().await? {
        let commit_id = config_manager.git_commit(&commit_msg).await?;
        return Ok(Json(serde_json::json!({
            "success": true,
            "provider_id": provider_id,
            "commit": commit_id,
        })));
    }
    
    Ok(Json(serde_json::json!({
        "success": true,
        "provider_id": provider_id,
    })))
}

/// 删除 Provider 实例
pub async fn delete_provider(
    Extension(config_manager): Extension<Arc<ConfigManager>>,
    Path(provider_id): Path<String>,
) -> Result<Json<serde_json::Value>> {
    config_manager.delete_provider(&provider_id).await?;
    
    // 创建 Git 提交
    config_manager.ensure_git_repo().await?;
    config_manager.git_add(".").await?;
    if config_manager.has_changes().await? {
        let commit_id = config_manager
            .git_commit(&format!("Delete provider: {}", provider_id))
            .await?;
        return Ok(Json(serde_json::json!({
            "success": true,
            "commit": commit_id,
        })));
    }
    
    Ok(Json(serde_json::json!({"success": true})))
}

/// 获取模型优先级
pub async fn get_model_priority(
    Extension(config_manager): Extension<Arc<ConfigManager>>,
) -> Result<Json<serde_json::Value>> {
    let priority = config_manager.get_model_priority().await?;
    Ok(Json(serde_json::json!({
        "primary": priority.primary,
        "fallbacks": priority.fallbacks,
    })))
}

/// 保存模型优先级
pub async fn save_model_priority(
    Extension(config_manager): Extension<Arc<ConfigManager>>,
    Json(data): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>> {
    let primary = data
        .get("primary")
        .and_then(|p| p.as_str())
        .unwrap_or("");
    
    let fallbacks: Vec<String> = data
        .get("fallbacks")
        .and_then(|f| f.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();
    
    config_manager
        .save_model_priority(primary, &fallbacks)
        .await?;
    
    // 创建 Git 提交
    config_manager.ensure_git_repo().await?;
    config_manager.git_add(".").await?;
    if config_manager.has_changes().await? {
        let commit_id = config_manager.git_commit("Update model priority").await?;
        return Ok(Json(serde_json::json!({
            "success": true,
            "commit": commit_id,
        })));
    }
    
    Ok(Json(serde_json::json!({"success": true})))
}