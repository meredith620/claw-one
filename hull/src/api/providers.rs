use axum::{
    extract::{Extension, Path},
    Json,
};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

use crate::{
    config::ConfigManager,
    error::{AppError, Result},
};

// GitHub OAuth Device Flow state
static GITHUB_OAUTH_STATE: Lazy<RwLock<Option<GitHubOAuthState>>> = Lazy::new(|| RwLock::new(None));

#[derive(Clone, Debug, Serialize, Deserialize)]
struct GitHubOAuthState {
    device_code: String,
    user_code: String,
    verification_uri: String,
    pending: bool,
    access_token: Option<String>,
    expires_at: Option<i64>,
}

/// Initialize GitHub Copilot OAuth Device Flow
/// POST /api/providers/github-copilot/init
pub async fn github_copilot_init() -> Result<Json<serde_json::Value>> {
    let client = reqwest::Client::new();
    
    // GitHub Copilot client ID (public GitHub OAuth app)
    let client_id = "Iv1.b507a08c87ecfe98";
    let scope = "read:user";
    
    // Request device code from GitHub
    let response = client
        .post("https://github.com/login/device/code")
        .header("Accept", "application/json")
        .form(&[
            ("client_id", client_id),
            ("scope", scope),
        ])
        .send()
        .await
        .map_err(|e| AppError::BadRequest(format!("GitHub OAuth 请求失败: {}", e)))?;
    
    if !response.status().is_success() {
        return Err(AppError::BadRequest(format!(
            "GitHub OAuth 失败: HTTP {}",
            response.status()
        )));
    }
    
    let body: serde_json::Value = response.json().await
        .map_err(|e| AppError::BadRequest(format!("解析响应失败: {}", e)))?;
    
    let device_code = body.get("device_code")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::BadRequest("缺少 device_code".to_string()))?;
    
    let user_code = body.get("user_code")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::BadRequest("缺少 user_code".to_string()))?;
    
    let verification_uri = body.get("verification_uri")
        .and_then(|v| v.as_str())
        .unwrap_or("https://github.com/login/device")
        .to_string();
    
    let expires_in = body.get("expires_in")
        .and_then(|v| v.as_u64())
        .unwrap_or(900);
    
    let interval = body.get("interval")
        .and_then(|v| v.as_u64())
        .unwrap_or(5);
    
    // Store state
    let state = GitHubOAuthState {
        device_code: device_code.to_string(),
        user_code: user_code.to_string(),
        verification_uri: verification_uri.clone(),
        pending: true,
        access_token: None,
        expires_at: None,
    };
    
    let mut current = GITHUB_OAUTH_STATE.write().await;
    *current = Some(state);
    
    Ok(Json(serde_json::json!({
        "success": true,
        "device_code": device_code,
        "user_code": user_code,
        "verification_uri": verification_uri,
        "expires_in": expires_in,
        "interval": interval,
        "pending": true,
    })))
}

/// Check GitHub Copilot OAuth status
/// GET /api/providers/github-copilot/status
pub async fn github_copilot_status() -> Result<Json<serde_json::Value>> {
    let state_guard = GITHUB_OAUTH_STATE.read().await;
    let state = state_guard.as_ref()
        .ok_or_else(|| AppError::BadRequest("OAuth 流程未初始化".to_string()))?;
    
    if !state.pending {
        // Authorization completed
        return Ok(Json(serde_json::json!({
            "success": true,
            "pending": false,
            "access_token": state.access_token,
            "expires_at": state.expires_at,
        })));
    }
    
    // Try to exchange device code for token
    let client = reqwest::Client::new();
    let client_id = "Iv1.b507a08c87ecfe98";
    
    let response = client
        .post("https://github.com/login/oauth/access_token")
        .header("Accept", "application/json")
        .form(&[
            ("client_id", client_id),
            ("device_code", &state.device_code),
            ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
        ])
        .send()
        .await
        .map_err(|e| AppError::BadRequest(format!("检查状态失败: {}", e)))?;
    
    let body: serde_json::Value = response.json().await
        .map_err(|e| AppError::BadRequest(format!("解析响应失败: {}", e)))?;
    
    // Check if we got an access token
    if let Some(access_token) = body.get("access_token").and_then(|v| v.as_str()) {
        let expires_in = body.get("expires_in").and_then(|v| v.as_u64());
        let expires_at = expires_in.map(|e| {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64 + e as i64
        });
        
        // Update state with token
        drop(state_guard);
        let mut current = GITHUB_OAUTH_STATE.write().await;
        if let Some(ref mut s) = *current {
            s.pending = false;
            s.access_token = Some(access_token.to_string());
            s.expires_at = expires_at;
        }
        
        return Ok(Json(serde_json::json!({
            "success": true,
            "pending": false,
            "access_token": access_token,
            "expires_at": expires_at,
        })));
    }
    
    // Check for error
    let error = body.get("error").and_then(|v| v.as_str());
    let error_description = body.get("error_description").and_then(|v| v.as_str());
    
    if error == Some("authorization_pending") {
        // Still waiting for user authorization
        return Ok(Json(serde_json::json!({
            "success": true,
            "pending": true,
            "error": null,
        })));
    }
    
    if error == Some("authorization_declined") {
        return Err(AppError::BadRequest("用户拒绝了授权请求".to_string()));
    }
    
    if error == Some("expired_token") {
        return Err(AppError::BadRequest("设备码已过期，请重新发起授权".to_string()));
    }
    
    // Still pending or other error
    Ok(Json(serde_json::json!({
        "success": true,
        "pending": true,
        "error": error,
        "error_description": error_description,
    })))
}

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
    tracing::info!(
        "save_provider called: provider_id={}, data={:?}",
        provider_id,
        data
    );

    // 检查 Provider ID 是否冲突
    let providers = match config_manager.get_providers().await {
        Ok(p) => p,
        Err(e) => {
            tracing::error!("get_providers failed: {}", e);
            return Err(e);
        }
    };

    let exists = providers
        .iter()
        .any(|p| p.get("id").and_then(|id| id.as_str()) == Some(&provider_id));

    tracing::info!("Provider exists: {}", exists);

    // 从请求数据中获取实际的 provider ID（用于判断是否更改 ID）
    let data_id = data
        .get("id")
        .and_then(|id| id.as_str())
        .unwrap_or(&provider_id);

    // 如果 URL 中的 ID 和数据中的 ID 不同，表示要重命名 provider
    // 需要检查新 ID 是否已被其他 provider 使用
    if data_id != provider_id {
        let new_id_exists = providers.iter().any(|p| {
            let pid = p.get("id").and_then(|id| id.as_str());
            pid == Some(data_id) && pid != Some(&provider_id)
        });

        if new_id_exists {
            return Err(AppError::BadRequest(format!(
                "Provider ID '{}' already exists",
                data_id
            )));
        }
    }

    // 确定是更新还是创建
    let is_update = exists;

    // 保存 Provider
    tracing::info!("Calling save_provider for: {}", provider_id);
    if let Err(e) = config_manager.save_provider(&provider_id, &data).await {
        tracing::error!("save_provider failed: {}", e);
        return Err(e);
    }

    // 读取当前完整配置并同步到 version-config/
    let config = match config_manager.get_config().await {
        Ok(c) => c,
        Err(e) => {
            tracing::error!("get_config failed: {}", e);
            return Err(e);
        }
    };

    let commit_msg = if is_update {
        format!("Update provider: {}", provider_id)
    } else {
        format!("Add provider: {}", provider_id)
    };

    match config_manager
        .sync_to_version_config(&config, Some(commit_msg))
        .await
    {
        Ok(Some(commit_id)) => {
            tracing::info!("Provider saved successfully with commit: {}", commit_id);
            Ok(Json(serde_json::json!({
                "success": true,
                "provider_id": provider_id,
                "commit": commit_id,
            })))
        }
        Ok(None) => {
            tracing::info!("Provider saved successfully (no changes)");
            Ok(Json(serde_json::json!({
                "success": true,
                "provider_id": provider_id,
            })))
        }
        Err(e) => {
            tracing::error!("sync_to_version_config failed: {}", e);
            Err(e)
        }
    }
}

/// 验证 Provider 凭证（预保存验证）
/// POST /api/providers/verify
pub async fn verify_provider(
    Json(data): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>> {
    let api_key = data
        .get("apiKey")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::BadRequest("Missing apiKey".to_string()))?;

    let base_url = data
        .get("baseUrl")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::BadRequest("Missing baseUrl".to_string()))?;

    let api_type = data
        .get("api")
        .and_then(|v| v.as_str())
        .unwrap_or("openai-chat");

    match verify_by_api_type(api_type, base_url, api_key).await {
        Ok(valid) => Ok(Json(serde_json::json!({
            "success": true,
            "valid": valid,
            "message": if valid { "凭证验证通过" } else { "凭证无效" }
        }))),
        Err(e) => Ok(Json(serde_json::json!({
            "success": false,
            "error": e.to_string()
        }))),
    }
}

async fn verify_by_api_type(
    api_type: &str,
    base_url: &str,
    api_key: &str,
) -> Result<bool> {
    match api_type {
        "openai-chat" | "openai-completions" => {
            verify_openai_compatible(base_url, api_key).await
        }
        "anthropic-messages" => {
            verify_anthropic_compatible(base_url, api_key).await
        }
        _ => Ok(true),
    }
}

async fn verify_openai_compatible(base_url: &str, api_key: &str) -> Result<bool> {
    let client = reqwest::Client::new();
    let url = format!("{}/models", base_url.trim_end_matches('/'));

    let response = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .timeout(Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| {
            if e.is_timeout() {
                AppError::BadRequest("验证请求超时，请检查网络连接".to_string())
            } else {
                AppError::BadRequest(format!("验证请求失败: {}", e))
            }
        })?;

    if response.status().is_success() {
        let body: serde_json::Value = response.json().await.map_err(|e| {
            AppError::BadRequest(format!("解析响应失败: {}", e))
        })?;
        let valid = body
            .get("data")
            .and_then(|d| d.as_array())
            .map(|a| !a.is_empty())
            .unwrap_or(false);
        Ok(valid)
    } else {
        Ok(false)
    }
}

async fn verify_anthropic_compatible(base_url: &str, api_key: &str) -> Result<bool> {
    let client = reqwest::Client::new();
    let url = format!("{}/v1/messages", base_url.trim_end_matches('/'));

    let response = client
        .post(&url)
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .json(&serde_json::json!({
            "model": "claude-3-haiku-20240307",
            "max_tokens": 10,
            "messages": [{"role": "user", "content": "hi"}]
        }))
        .timeout(Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| {
            if e.is_timeout() {
                AppError::BadRequest("验证请求超时，请检查网络连接".to_string())
            } else {
                AppError::BadRequest(format!("验证请求失败: {}", e))
            }
        })?;

    if response.status() == 200 {
        let body: serde_json::Value = response.json().await
            .map_err(|e| AppError::BadRequest(format!("解析响应失败: {}", e)))?;
        if body.get("error").is_some() {
            return Ok(false);
        }
        let content_valid = body
            .get("content")
            .and_then(|c| c.as_array())
            .map(|a| !a.is_empty())
            .unwrap_or(false);
        Ok(content_valid)
    } else {
        Ok(false)
    }
}

/// 删除 Provider 实例
pub async fn delete_provider(
    Extension(config_manager): Extension<Arc<ConfigManager>>,
    Path(provider_id): Path<String>,
) -> Result<Json<serde_json::Value>> {
    config_manager.delete_provider(&provider_id).await?;

    // 读取当前完整配置并同步到 version-config/
    let config = config_manager.get_config().await?;
    let commit_msg = format!("Delete provider: {}", provider_id);

    match config_manager
        .sync_to_version_config(&config, Some(commit_msg))
        .await
    {
        Ok(Some(commit_id)) => Ok(Json(serde_json::json!({
            "success": true,
            "commit": commit_id,
        }))),
        Ok(None) => Ok(Json(serde_json::json!({"success": true}))),
        Err(e) => {
            tracing::error!("sync_to_version_config failed: {}", e);
            Err(e)
        }
    }
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
    let primary = data.get("primary").and_then(|p| p.as_str()).unwrap_or("");

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

    // 读取当前完整配置并同步到 version-config/
    let config = config_manager.get_config().await?;

    match config_manager
        .sync_to_version_config(&config, Some("Update model priority".to_string()))
        .await
    {
        Ok(Some(commit_id)) => Ok(Json(serde_json::json!({
            "success": true,
            "commit": commit_id,
        }))),
        Ok(None) => Ok(Json(serde_json::json!({"success": true}))),
        Err(e) => {
            tracing::error!("sync_to_version_config failed: {}", e);
            Err(e)
        }
    }
}
