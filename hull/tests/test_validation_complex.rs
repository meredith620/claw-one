use serde_json::Value;

// ============================================================================
// 1. ID Format Validation
// ============================================================================

/// Valid ID: only alphanumeric + underscore, length 3-32
fn is_valid_id(id: &str) -> bool {
    if id.len() < 3 || id.len() > 32 {
        return false;
    }
    id.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

// ============================================================================
// 2. Nested Object Validation
// ============================================================================

struct ConfigValidator;

impl ConfigValidator {
    fn validate(config: &Value) -> Result<(), String> {
        // Validate models.providers exists and is an object
        let models = config
            .get("models")
            .ok_or_else(|| "missing 'models' field".to_string())?;

        let providers = models
            .get("providers")
            .ok_or_else(|| "missing 'models.providers' field".to_string())?;

        if !providers.is_object() {
            return Err("'models.providers' must be an object".to_string());
        }

        // Validate agents.list array - each agent must have an 'id' field
        if let Some(agents) = config.get("agents").and_then(|a| a.get("list")) {
            if let Some(arr) = agents.as_array() {
                for (i, agent) in arr.iter().enumerate() {
                    if !agent.get("id").is_some() {
                        return Err(format!("agent at index {} missing 'id' field", i));
                    }
                }
            }
        }

        Ok(())
    }
}

// ============================================================================
// 3. Deep Merge
// ============================================================================

fn deep_merge(base: &mut Value, updates: Value) {
    if let Value::Object(base_map) = base {
        if let Value::Object(updates_map) = updates {
            let keys: Vec<String> = updates_map.keys().cloned().collect();
            for key in keys {
                let value = updates_map.get(&key).cloned().unwrap();
                if let Some(base_entry) = base_map.get_mut(&key) {
                    if base_entry.is_object() && value.is_object() {
                        // Recursively merge nested objects
                        deep_merge(base_entry, value);
                        continue;
                    }
                }
                base_map.insert(key, value);
            }
            return;
        }
    }
    *base = updates;
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests_id_validation {
    use super::*;

    #[test]
    fn test_valid_ids() {
        assert!(is_valid_id("abc"));
        assert!(is_valid_id("user_123"));
        assert!(is_valid_id("Test_User_Name_123"));
        assert!(is_valid_id("abc1"));
        // length 32 (valid max)
        assert!(is_valid_id("abcdefghijklmnopqrstuvwxyz123456"));
    }

    #[test]
    fn test_invalid_id_empty() {
        assert!(!is_valid_id(""));
    }

    #[test]
    fn test_invalid_id_too_short() {
        assert!(!is_valid_id("ab")); // length 2
    }

    #[test]
    fn test_invalid_id_too_long() {
        // length 33
        assert!(!is_valid_id("abcdefghijklmnopqrstuvwxyz1234567"));
    }

    #[test]
    fn test_invalid_id_special_chars() {
        assert!(!is_valid_id("user-name"));
        assert!(!is_valid_id("user.name"));
        assert!(!is_valid_id("user name"));
        assert!(!is_valid_id("user@name"));
        assert!(!is_valid_id("user!name"));
        assert!(!is_valid_id("user#name"));
        assert!(!is_valid_id("user$name"));
        assert!(!is_valid_id("user%name"));
        assert!(!is_valid_id("user/name"));
    }

    #[test]
    fn test_invalid_id_unicode() {
        assert!(!is_valid_id("用户_123"));
        assert!(!is_valid_id("useré"));
    }
}

#[cfg(test)]
mod tests_config_validation {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_valid_config() {
        let config = json!({
            "models": {
                "providers": {
                    "openai": {"api_key": "xxx"},
                    "anthropic": {"api_key": "yyy"}
                }
            },
            "agents": {
                "list": [
                    {"id": "agent1", "name": "Agent One"},
                    {"id": "agent2", "name": "Agent Two"}
                ]
            }
        });
        assert!(ConfigValidator::validate(&config).is_ok());
    }

    #[test]
    fn test_missing_models() {
        let config = json!({
            "agents": {"list": []}
        });
        let result = ConfigValidator::validate(&config);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "missing 'models' field");
    }

    #[test]
    fn test_missing_providers() {
        let config = json!({
            "models": {}
        });
        let result = ConfigValidator::validate(&config);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "missing 'models.providers' field");
    }

    #[test]
    fn test_providers_not_object() {
        let config = json!({
            "models": {
                "providers": ["openai", "anthropic"]
            }
        });
        let result = ConfigValidator::validate(&config);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "'models.providers' must be an object");
    }

    #[test]
    fn test_agent_missing_id() {
        let config = json!({
            "models": {
                "providers": {}
            },
            "agents": {
                "list": [
                    {"id": "agent1", "name": "Agent One"},
                    {"name": "Agent Without ID"}
                ]
            }
        });
        let result = ConfigValidator::validate(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("missing 'id' field"));
    }

    #[test]
    fn test_empty_agents_list() {
        let config = json!({
            "models": {
                "providers": {}
            },
            "agents": {
                "list": []
            }
        });
        assert!(ConfigValidator::validate(&config).is_ok());
    }

    #[test]
    fn test_no_agents_field() {
        let config = json!({
            "models": {
                "providers": {}
            }
        });
        // No agents field is ok (list is optional)
        assert!(ConfigValidator::validate(&config).is_ok());
    }
}

#[cfg(test)]
mod tests_deep_merge {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_merge_preserves_existing_fields() {
        let mut base = json!({
            "name": "test",
            "version": "1.0",
            "nested": {
                "a": 1,
                "b": 2
            }
        });
        let updates = json!({
            "nested": {
                "b": 20,
                "c": 3
            }
        });

        deep_merge(&mut base, updates);

        let expected = json!({
            "name": "test",
            "version": "1.0",
            "nested": {
                "a": 1,
                "b": 20,
                "c": 3
            }
        });
        assert_eq!(base, expected);
    }

    #[test]
    fn test_merge_adds_new_top_level_fields() {
        let mut base = json!({"name": "test"});
        let updates = json!({"version": "2.0"});

        deep_merge(&mut base, updates);

        let expected = json!({"name": "test", "version": "2.0"});
        assert_eq!(base, expected);
    }

    #[test]
    fn test_merge_overwrites_non_object() {
        let mut base = json!({"value": 1});
        let updates = json!({"value": 2});

        deep_merge(&mut base, updates);

        assert_eq!(base, json!({"value": 2}));
    }

    #[test]
    fn test_merge_deep_nested() {
        let mut base = json!({
            "level1": {
                "level2": {
                    "keep": "yes",
                    "old": 1
                }
            }
        });
        let updates = json!({
            "level1": {
                "level2": {
                    "new": 2
                }
            }
        });

        deep_merge(&mut base, updates);

        let expected = json!({
            "level1": {
                "level2": {
                    "keep": "yes",
                    "old": 1,
                    "new": 2
                }
            }
        });
        assert_eq!(base, expected);
    }
}
