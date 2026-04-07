use axum::{
    extract::{Extension, Path},
    Json,
};
use std::sync::Arc;

use crate::{
    config::ConfigManager, error::Result, state::StateManager, types::Config,
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
        "tools" => config
            .get("tools")
            .cloned()
            .unwrap_or(serde_json::json!({})),
        "session" => config
            .get("session")
            .cloned()
            .unwrap_or(serde_json::json!({})),
        "agents" => config
            .get("agents")
            .cloned()
            .unwrap_or(serde_json::json!({})),
        "models" => config
            .get("models")
            .cloned()
            .unwrap_or(serde_json::json!({})),
        "channels" => config
            .get("channels")
            .cloned()
            .unwrap_or(serde_json::json!({})),
        _ => serde_json::json!({}),
    };

    Ok(Json(module_config))
}

/// 递归合并两个JSON对象
fn deep_merge(existing: &serde_json::Value, new: &serde_json::Value) -> serde_json::Value {
    if let (Some(existing_obj), Some(new_obj)) = (existing.as_object(), new.as_object()) {
        let mut merged = existing_obj.clone();
        for (key, value) in new_obj.iter() {
            if let Some(existing_val) = merged.get(key) {
                // 如果两边都是对象，递归合并
                if existing_val.is_object() && value.is_object() {
                    merged.insert(key.clone(), deep_merge(existing_val, value));
                } else {
                    // 否则直接替换
                    merged.insert(key.clone(), value.clone());
                }
            } else {
                // 新key，直接插入
                merged.insert(key.clone(), value.clone());
            }
        }
        serde_json::Value::Object(merged)
    } else {
        // 如果不是对象，直接返回新值
        new.clone()
    }
}

/// 保存指定模块的配置
pub async fn save_module_handler(
    Path(module): Path<String>,
    Extension(config_manager): Extension<Arc<ConfigManager>>,
    Json(module_config): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>> {
    // 获取当前完整配置
    let mut full_config = config_manager.get_config().await?;

    // 根据模块名更新对应的配置（deep merge，保留已有子字段）
    let valid_modules = ["tools", "session", "agents", "models", "channels"];
    if !valid_modules.contains(&module.as_str()) {
        return Ok(Json(serde_json::json!({
            "success": false,
            "message": format!("Unknown module: {}", module),
        })));
    }

    // Deep merge: 递归合并传入的字段到现有配置
    if let Some(existing) = full_config.get(&module).cloned() {
        if existing.is_object() && module_config.is_object() {
            let merged = deep_merge(&existing, &module_config);
            full_config[&module] = merged;
        } else {
            full_config[&module] = module_config;
        }
    } else {
        full_config[&module] = module_config;
    }

    // 使用 sync_to_version_config 完成：保存 + 复制 + diff + Git 提交
    match config_manager
        .sync_to_version_config(&full_config, Some(format!("Update {} module", module)))
        .await
    {
        Ok(Some(commit_id)) => Ok(Json(serde_json::json!({
            "success": true,
            "commit": commit_id,
        }))),
        Ok(None) => Ok(Json(serde_json::json!({
            "success": true,
            "message": format!("{} module saved successfully", module),
        }))),
        Err(e) => {
            tracing::error!(
                "save_module_handler [{}]: sync_to_version_config failed: {}",
                module,
                e
            );
            Err(e)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_deep_merge_simple() {
        let existing = json!({"a": 1, "b": 2});
        let new = json!({"b": 3, "c": 4});

        let merged = deep_merge(&existing, &new);

        assert_eq!(merged["a"], 1);
        assert_eq!(merged["b"], 3); // 被覆盖
        assert_eq!(merged["c"], 4); // 新 key
    }

    #[test]
    fn test_deep_merge_nested() {
        let existing = json!({
            "level1": {
                "a": 1,
                "level2": {
                    "x": "old"
                }
            }
        });
        let new = json!({
            "level1": {
                "b": 2,
                "level2": {
                    "y": "new"
                }
            }
        });

        let merged = deep_merge(&existing, &new);

        // 第一层合并
        assert_eq!(merged["level1"]["a"], 1); // 保留
        assert_eq!(merged["level1"]["b"], 2); // 新增

        // 第二层递归合并
        assert_eq!(merged["level1"]["level2"]["x"], "old"); // 保留
        assert_eq!(merged["level1"]["level2"]["y"], "new"); // 新增
    }

    #[test]
    fn test_deep_merge_overwrite_non_object() {
        let existing = json!({"value": "old string"});
        let new = json!({"value": 42});

        let merged = deep_merge(&existing, &new);

        // 非对象字段直接覆盖
        assert_eq!(merged["value"], 42);
    }

    #[test]
    fn test_deep_merge_new_keys() {
        let existing = json!({"existing_key": "value"});
        let new = json!({"new_key": "new_value"});

        let merged = deep_merge(&existing, &new);

        assert_eq!(merged["existing_key"], "value"); // 保留
        assert_eq!(merged["new_key"], "new_value"); // 新增
    }

    #[test]
    fn test_deep_merge_null_value() {
        let existing = json!({"key": "non-null value"});
        let new = json!({"key": null});

        let merged = deep_merge(&existing, &new);

        // null 值覆盖非 null
        assert!(merged["key"].is_null());
    }

    #[test]
    fn test_deep_merge_array_field() {
        let existing = json!({"items": [1, 2, 3]});
        let new = json!({"items": [4, 5]});

        let merged = deep_merge(&existing, &new);

        // 数组字段直接替换，不合并元素
        assert_eq!(merged["items"], json!([4, 5]));
        assert_ne!(merged["items"], json!([1, 2, 3, 4, 5]));
    }

    #[test]
    fn test_deep_merge_non_object_inputs() {
        // existing 不是对象
        let existing = json!("string");
        let new = json!({"key": "value"});

        let merged = deep_merge(&existing, &new);
        assert_eq!(merged, new);

        // new 不是对象
        let existing2 = json!({"key": "value"});
        let new2 = json!("string");

        let merged2 = deep_merge(&existing2, &new2);
        assert_eq!(merged2, new2);

        // 两者都不是对象
        let existing3 = json!(123);
        let new3 = json!("string");

        let merged3 = deep_merge(&existing3, &new3);
        assert_eq!(merged3, new3);
    }
}
