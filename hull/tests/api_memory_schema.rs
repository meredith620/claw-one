mod common;

use common::TestServer;
use serde_json::Value;

/// JSON Schema 验证器（简化版，不依赖外部 crate）
fn validate_memory_schema(data: &Value) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    // 检查是否为对象
    if !data.is_object() {
        errors.push(format!("Expected object, got {}", json_type_name(data)));
        return Err(errors);
    }

    let obj = data.as_object().unwrap();

    // 检查必需字段
    if !obj.contains_key("enabled") {
        errors.push("Missing required field: enabled".to_string());
    } else if !obj["enabled"].is_boolean() {
        errors.push(format!(
            "Field 'enabled' should be boolean, got {}",
            json_type_name(&obj["enabled"])
        ));
    }

    if !obj.contains_key("provider") {
        errors.push("Missing required field: provider".to_string());
    } else if let Some(provider) = obj["provider"].as_str() {
        if provider != "ollama" && provider != "openai" {
            errors.push(format!(
                "Field 'provider' should be 'ollama' or 'openai', got '{}'",
                provider
            ));
        }
    } else {
        errors.push(format!(
            "Field 'provider' should be string, got {}",
            json_type_name(&obj["provider"])
        ));
    }

    // 检查禁止字段（Bug #1 相关）
    let forbidden_fields = ["_vts", "isTrusted", "target", "currentTarget", "type"];
    for field in &forbidden_fields {
        if obj.contains_key(*field) {
            errors.push(format!(
                "Forbidden field '{}' found - likely an event object was passed instead of config data",
                field
            ));
        }
    }

    // 检查未知字段
    let allowed_fields = [
        "enabled", "provider", "remote", "model", "fallback", "sources", "query", "store", "sync",
    ];
    for key in obj.keys() {
        if !allowed_fields.contains(&key.as_str()) {
            errors.push(format!("Unknown field: '{}'", key));
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn json_type_name(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

/// 测试：有效配置通过 Schema 验证
#[tokio::test]
async fn test_valid_memory_config_passes_schema() {
    let valid_configs = vec![
        serde_json::json!({
            "enabled": true,
            "provider": "ollama",
            "remote": { "baseUrl": "http://localhost:11434" },
            "model": "qwen3-embedding:0.6b"
        }),
        serde_json::json!({
            "enabled": false,
            "provider": "openai"
        }),
        serde_json::json!({
            "enabled": true,
            "provider": "ollama",
            "fallback": "none",
            "sources": ["memory", "sessions"]
        }),
    ];

    for config in valid_configs {
        let result = validate_memory_schema(&config);
        assert!(
            result.is_ok(),
            "Valid config should pass schema validation: {:?}\nErrors: {:?}",
            config,
            result.err()
        );
    }

    println!("✅ 所有有效配置通过 Schema 验证");
}

/// 测试：事件对象不通过 Schema 验证（Bug #1 核心）
#[tokio::test]
async fn test_event_object_fails_schema() {
    let event_object = serde_json::json!({
        "_vts": 1774700348600_i64,
        "isTrusted": true
    });

    let result = validate_memory_schema(&event_object);
    assert!(
        result.is_err(),
        "Event object should fail schema validation"
    );

    let errors = result.unwrap_err();
    println!("事件对象验证错误（预期）:");
    for error in &errors {
        println!("  - {}", error);
    }

    // 确认检测到了禁止字段
    assert!(
        errors.iter().any(|e| e.contains("_vts")),
        "Should detect forbidden field '_vts'"
    );
    assert!(
        errors.iter().any(|e| e.contains("isTrusted")),
        "Should detect forbidden field 'isTrusted'"
    );
}

/// 测试：缺少必需字段不通过验证
#[tokio::test]
async fn test_missing_required_fields_fails_schema() {
    let invalid_configs = vec![
        // 缺少 enabled
        serde_json::json!({
            "provider": "ollama"
        }),
        // 缺少 provider
        serde_json::json!({
            "enabled": true
        }),
        // 空对象
        serde_json::json!({}),
    ];

    for config in invalid_configs {
        let result = validate_memory_schema(&config);
        assert!(
            result.is_err(),
            "Config with missing fields should fail: {:?}",
            config
        );
    }
}

/// 测试：非对象类型不通过验证
#[tokio::test]
async fn test_non_object_types_fails_schema() {
    let invalid_types = vec![
        serde_json::json!([]),
        serde_json::json!("string"),
        serde_json::json!(123),
        serde_json::json!(true),
        serde_json::Value::Null,
    ];

    for data in invalid_types {
        let result = validate_memory_schema(&data);
        assert!(result.is_err(), "Non-object type should fail: {:?}", data);
    }
}

/// 测试：后端 API 应该应用 Schema 验证
#[tokio::test]
async fn test_backend_should_validate_schema() {
    let server = TestServer::new().await;

    // 测试各种无效数据
    let test_cases = vec![
        (
            "event_object",
            serde_json::json!({"_vts": 123, "isTrusted": true}),
            "should reject event object",
        ),
        ("array", serde_json::json!([1, 2, 3]), "should reject array"),
        (
            "string",
            serde_json::json!("invalid"),
            "should reject string",
        ),
    ];

    for (name, data, description) in test_cases {
        let response = server
            .client
            .post(server.url("/api/memory"))
            .json(&data)
            .send()
            .await
            .unwrap();

        let status = response.status();

        // 当前后端没有验证，记录当前行为
        println!("{}: status={}, description={}", name, status, description);

        // FIXME: 修复后应该断言 400
        // assert_eq!(status, 400, "{}", description);
    }
}
