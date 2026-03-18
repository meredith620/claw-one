use serde_json::Value;
use crate::error::Result;

/// 配置验证错误
#[derive(Debug, Clone)]
pub struct ValidationError {
    pub path: String,
    pub message: String,
}

/// 配置验证结果
#[derive(Debug)]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationError>,
}

impl ValidationResult {
    pub fn new() -> Self {
        Self {
            valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    pub fn add_error(&mut self, path: &str, message: &str) {
        self.valid = false;
        self.errors.push(ValidationError {
            path: path.to_string(),
            message: message.to_string(),
        });
    }

    pub fn add_warning(&mut self, path: &str, message: &str) {
        self.warnings.push(ValidationError {
            path: path.to_string(),
            message: message.to_string(),
        });
    }
}

/// 验证配置
pub fn validate_config(config: &Value) -> ValidationResult {
    let mut result = ValidationResult::new();

    // 验证顶层结构
    validate_structure(config, &mut result);
    
    // 验证 models 配置
    validate_models(config, &mut result);
    
    // 验证 agents 配置
    validate_agents(config, &mut result);
    
    // 验证 channels 配置
    validate_channels(config, &mut result);

    result
}

/// 仅验证 agents 配置（用于增量保存）
pub fn validate_agents_only(config: &Value) -> ValidationResult {
    let mut result = ValidationResult::new();
    
    // 只验证 agents 配置
    validate_agents(config, &mut result);
    
    result
}

/// 验证顶层结构
fn validate_structure(config: &Value, result: &mut ValidationResult) {
    // 检查是否为对象
    if !config.is_object() {
        result.add_error("", "配置必须是 JSON 对象");
        return;
    }

    // 检查必需字段
    let required_fields = vec!["models", "agents", "channels"];
    for field in required_fields {
        if config.get(field).is_none() {
            result.add_error("", &format!("缺少必需字段: {}", field));
        }
    }
}

/// 验证 models 配置
fn validate_models(config: &Value, result: &mut ValidationResult) {
    let Some(models) = config.get("models") else {
        return;
    };

    if !models.is_object() {
        result.add_error("models", "models 必须是对象");
        return;
    }

    // 验证 providers
    if let Some(providers) = models.get("providers") {
        if !providers.is_object() {
            result.add_error("models.providers", "providers 必须是对象");
            return;
        }

        // 验证每个 provider
        if let Some(providers_obj) = providers.as_object() {
            for (provider_id, provider_config) in providers_obj {
                validate_provider(provider_id, provider_config, result);
            }
        }
    } else {
        result.add_warning("models.providers", "未配置任何 Provider");
    }
}

/// 验证单个 Provider
fn validate_provider(id: &str, config: &Value, result: &mut ValidationResult) {
    let base_path = format!("models.providers.{}", id);

    if !config.is_object() {
        result.add_error(&base_path, "Provider 配置必须是对象");
        return;
    }

    let provider = config.as_object().unwrap();

    // 检查必需字段
    if !provider.contains_key("apiKey") {
        result.add_error(&base_path, "缺少 apiKey");
    }

    if !provider.contains_key("baseUrl") {
        result.add_error(&base_path, "缺少 baseUrl");
    }

    // 检查 baseUrl 格式
    if let Some(base_url) = provider.get("baseUrl").and_then(|v| v.as_str()) {
        if !base_url.starts_with("http://") && !base_url.starts_with("https://") {
            result.add_error(&base_path, "baseUrl 必须以 http:// 或 https:// 开头");
        }
    }

    // 检查 enabled 字段
    if let Some(enabled) = provider.get("enabled") {
        if !enabled.is_boolean() {
            result.add_error(&base_path, "enabled 必须是布尔值");
        }
    }
}

/// 验证 agents 配置
fn validate_agents(config: &Value, result: &mut ValidationResult) {
    let Some(agents) = config.get("agents") else {
        return;
    };

    if !agents.is_object() {
        result.add_error("agents", "agents 必须是对象");
        return;
    }

    // 验证 defaults
    if let Some(defaults) = agents.get("defaults") {
        if !defaults.is_object() {
            result.add_error("agents.defaults", "defaults 必须是对象");
            return;
        }

        // 验证 workspace
        if let Some(workspace) = defaults.get("workspace") {
            if !workspace.is_string() {
                result.add_error("agents.defaults.workspace", "workspace 必须是字符串");
            }
        }

        // 验证 memorySearch
        if let Some(memory_search) = defaults.get("memorySearch") {
            validate_memory_search(memory_search, result);
        }
    } else {
        result.add_warning("agents.defaults", "未配置默认 Agent 设置");
    }

    // 验证自定义 agent 列表
    if let Some(list) = agents.get("list") {
        if !list.is_array() {
            result.add_error("agents.list", "list 必须是数组");
            return;
        }

        if let Some(agents_array) = list.as_array() {
            for (i, agent) in agents_array.iter().enumerate() {
                validate_agent_item(i, agent, result);
            }
        }
    }
}

/// 验证 Memory 搜索配置
fn validate_memory_search(config: &Value, result: &mut ValidationResult) {
    let base_path = "agents.defaults.memorySearch";

    if !config.is_object() {
        result.add_error(base_path, "memorySearch 必须是对象");
        return;
    }

    // 检查 provider
    if let Some(provider) = config.get("provider") {
        if let Some(provider_str) = provider.as_str() {
            if provider_str != "ollama" && provider_str != "openai" {
                result.add_warning(
                    base_path,
                    &format!("provider '{}' 不是标准值 (ollama/openai)", provider_str),
                );
            }
        } else {
            result.add_error(base_path, "provider 必须是字符串");
        }
    }

    // 检查 model
    if config.get("model").is_none() {
        result.add_warning(base_path, "未配置 model");
    }
}

/// 验证单个 Agent 项
fn validate_agent_item(index: usize, config: &Value, result: &mut ValidationResult) {
    let base_path = format!("agents.list[{}]", index);

    if !config.is_object() {
        result.add_error(&base_path, "Agent 必须是对象");
        return;
    }

    let agent = config.as_object().unwrap();

    // 检查必需字段
    if !agent.contains_key("id") {
        result.add_error(&base_path, "缺少 id");
    }

    if !agent.contains_key("name") {
        result.add_error(&base_path, "缺少 name");
    }
}

/// 验证 channels 配置
fn validate_channels(config: &Value, result: &mut ValidationResult) {
    let Some(channels) = config.get("channels") else {
        return;
    };

    if !channels.is_object() {
        result.add_error("channels", "channels 必须是对象");
        return;
    }

    // 验证 mattermost
    if let Some(mattermost) = channels.get("mattermost") {
        validate_mattermost(mattermost, result);
    }

    // 验证 feishu
    if let Some(feishu) = channels.get("feishu") {
        validate_feishu(feishu, result);
    }
}

/// 验证 Mattermost 配置
fn validate_mattermost(config: &Value, result: &mut ValidationResult) {
    let base_path = "channels.mattermost";

    if !config.is_object() {
        result.add_error(base_path, "mattermost 必须是对象");
        return;
    }

    // 检查 enabled
    if let Some(enabled) = config.get("enabled") {
        if !enabled.is_boolean() {
            result.add_error(base_path, "enabled 必须是布尔值");
        }
    }

    // 如果启用，检查 accounts
    if config.get("enabled").and_then(|v| v.as_bool()).unwrap_or(false) {
        if let Some(accounts) = config.get("accounts") {
            if !accounts.is_object() {
                result.add_error(&format!("{}.accounts", base_path), "accounts 必须是对象");
            } else if accounts.as_object().map(|o| o.is_empty()).unwrap_or(true) {
                result.add_warning(&format!("{}.accounts", base_path), "已启用但未配置账号");
            }
        } else {
            result.add_error(&format!("{}.accounts", base_path), "已启用但缺少 accounts 配置");
        }
    }
}

/// 验证飞书配置
fn validate_feishu(config: &Value, result: &mut ValidationResult) {
    let base_path = "channels.feishu";

    if !config.is_object() {
        result.add_error(base_path, "feishu 必须是对象");
        return;
    }

    // 检查 enabled
    if let Some(enabled) = config.get("enabled") {
        if !enabled.is_boolean() {
            result.add_error(base_path, "enabled 必须是布尔值");
        }
    }

    // 如果启用，检查必要字段
    if config.get("enabled").and_then(|v| v.as_bool()).unwrap_or(false) {
        if config.get("appId").is_none() {
            result.add_error(base_path, "已启用但缺少 appId");
        }
        if config.get("appSecret").is_none() {
            result.add_error(base_path, "已启用但缺少 appSecret");
        }
    }
}

/// 验证配置并返回详细的错误信息
pub fn validate_config_with_details(config: &Value) -> std::result::Result<(), Vec<String>> {
    let result = validate_config(config);
    
    if result.valid && result.errors.is_empty() {
        Ok(())
    } else {
        let mut messages = Vec::new();
        
        for error in &result.errors {
            messages.push(format!("[错误] {}: {}", error.path, error.message));
        }
        
        for warning in &result.warnings {
            messages.push(format!("[警告] {}: {}", warning.path, warning.message));
        }
        
        Err(messages)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_validate_empty_config() {
        let config = json!({});
        let result = validate_config(&config);
        assert!(!result.valid);
        assert!(!result.errors.is_empty());
    }

    #[test]
    fn test_validate_valid_config() {
        let config = json!({
            "models": {
                "providers": {
                    "moonshot-test": {
                        "apiKey": "sk-test",
                        "baseUrl": "https://api.example.com",
                        "enabled": true
                    }
                }
            },
            "agents": {
                "defaults": {
                    "workspace": "~/.openclaw/workspace",
                    "memorySearch": {
                        "provider": "ollama",
                        "model": "qwen3-embedding:0.6b"
                    }
                }
            },
            "channels": {
                "mattermost": {
                    "enabled": false
                }
            }
        });
        
        let result = validate_config(&config);
        // Should have no errors, maybe some warnings
        println!("Errors: {:?}", result.errors);
        println!("Warnings: {:?}", result.warnings);
    }
}
