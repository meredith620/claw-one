mod common;

use common::TestServer;

// ============================================================
// Agent ID 格式验证测试
// ============================================================

/// Agent ID 包含非法字符：空格
#[tokio::test]
async fn test_agent_id_with_space() {
    let server = TestServer::new().await;

    let agents_config = serde_json::json!({
        "defaults": {},
        "list": [
            {"id": "agent with space", "name": "Test Agent"}
        ]
    });

    let response = server
        .client
        .post(server.url("/api/agents"))
        .json(&agents_config)
        .send()
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        400,
        "Agent ID with space should return 400"
    );
}

/// Agent ID 包含非法字符：双引号
#[tokio::test]
async fn test_agent_id_with_double_quote() {
    let server = TestServer::new().await;

    let agents_config = serde_json::json!({
        "defaults": {},
        "list": [
            {"id": "agent\"injected", "name": "Test Agent"}
        ]
    });

    let response = server
        .client
        .post(server.url("/api/agents"))
        .json(&agents_config)
        .send()
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        400,
        "Agent ID with double quote should return 400"
    );
}

/// Agent ID 包含非法字符：单引号
#[tokio::test]
async fn test_agent_id_with_single_quote() {
    let server = TestServer::new().await;

    let agents_config = serde_json::json!({
        "defaults": {},
        "list": [
            {"id": "agent'injected", "name": "Test Agent"}
        ]
    });

    let response = server
        .client
        .post(server.url("/api/agents"))
        .json(&agents_config)
        .send()
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        400,
        "Agent ID with single quote should return 400"
    );
}

/// Agent ID 包含非法字符：正斜杠
#[tokio::test]
async fn test_agent_id_with_forward_slash() {
    let server = TestServer::new().await;

    let agents_config = serde_json::json!({
        "defaults": {},
        "list": [
            {"id": "agent/path", "name": "Test Agent"}
        ]
    });

    let response = server
        .client
        .post(server.url("/api/agents"))
        .json(&agents_config)
        .send()
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        400,
        "Agent ID with forward slash should return 400"
    );
}

/// Agent ID 包含非法字符：反斜杠
#[tokio::test]
async fn test_agent_id_with_backslash() {
    let server = TestServer::new().await;

    let agents_config = serde_json::json!({
        "defaults": {},
        "list": [
            {"id": "agent\\backslash", "name": "Test Agent"}
        ]
    });

    let response = server
        .client
        .post(server.url("/api/agents"))
        .json(&agents_config)
        .send()
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        400,
        "Agent ID with backslash should return 400"
    );
}

/// Agent ID 超长（> 256 字符）
#[tokio::test]
async fn test_agent_id_too_long() {
    let server = TestServer::new().await;

    let long_id = "a".repeat(257);
    let agents_config = serde_json::json!({
        "defaults": {},
        "list": [
            {"id": long_id, "name": "Test Agent"}
        ]
    });

    let response = server
        .client
        .post(server.url("/api/agents"))
        .json(&agents_config)
        .send()
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        400,
        "Agent ID > 256 chars should return 400"
    );
}

/// Agent ID 恰好 256 字符（边界测试，应该通过）
#[tokio::test]
async fn test_agent_id_max_length_valid() {
    let server = TestServer::new().await;

    let max_id = "a".repeat(256);
    let agents_config = serde_json::json!({
        "defaults": {},
        "list": [
            {"id": max_id, "name": "Test Agent"}
        ]
    });

    let response = server
        .client
        .post(server.url("/api/agents"))
        .json(&agents_config)
        .send()
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        200,
        "Agent ID of exactly 256 chars should succeed"
    );
}

/// Agent ID 为空字符串
#[tokio::test]
async fn test_agent_id_empty() {
    let server = TestServer::new().await;

    let agents_config = serde_json::json!({
        "defaults": {},
        "list": [
            {"id": "", "name": "Test Agent"}
        ]
    });

    let response = server
        .client
        .post(server.url("/api/agents"))
        .json(&agents_config)
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 400, "Empty Agent ID should return 400");
}

/// Agent ID 缺少 id 字段（应返回 400）
#[tokio::test]
async fn test_agent_id_missing() {
    let server = TestServer::new().await;

    let agents_config = serde_json::json!({
        "defaults": {},
        "list": [
            {"name": "Test Agent"}
        ]
    });

    let response = server
        .client
        .post(server.url("/api/agents"))
        .json(&agents_config)
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 400, "Missing Agent ID should return 400");
}

/// 多个 Agent 中有一个 ID 非法（应该整体返回 400）
#[tokio::test]
async fn test_agent_list_one_invalid_id() {
    let server = TestServer::new().await;

    let agents_config = serde_json::json!({
        "defaults": {},
        "list": [
            {"id": "valid-agent", "name": "Valid Agent"},
            {"id": "invalid agent", "name": "Invalid Agent"}
        ]
    });

    let response = server
        .client
        .post(server.url("/api/agents"))
        .json(&agents_config)
        .send()
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        400,
        "One invalid ID in list should cause 400"
    );
}

/// 正常 Agent ID（字母数字连字符下划线）应该成功
#[tokio::test]
async fn test_agent_id_valid_formats() {
    let server = TestServer::new().await;

    let valid_ids = vec![
        "agent-1",
        "agent_2",
        "agent.3",
        "TestAgent",
        "a",
        "agent123",
    ];

    for id in valid_ids {
        let agents_config = serde_json::json!({
            "defaults": {},
            "list": [
                {"id": id, "name": "Test Agent"}
            ]
        });

        let response = server
            .client
            .post(server.url("/api/agents"))
            .json(&agents_config)
            .send()
            .await
            .unwrap();

        assert_eq!(
            response.status(),
            200,
            "Valid Agent ID '{}' should succeed",
            id
        );
    }
}

// ============================================================
// Channel ID 格式验证测试
// ============================================================

/// Channel ID 包含非法字符：空格
#[tokio::test]
async fn test_channel_id_with_space() {
    let server = TestServer::new().await;

    let channels_config = serde_json::json!({
        "mattermost with space": {
            "enabled": false
        }
    });

    let response = server
        .client
        .post(server.url("/api/channels"))
        .json(&channels_config)
        .send()
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        400,
        "Channel ID with space should return 400"
    );
}

/// Channel ID 包含非法字符：双引号
#[tokio::test]
async fn test_channel_id_with_double_quote() {
    let server = TestServer::new().await;

    let channels_config = serde_json::json!({
        "mattermost\"injected": {
            "enabled": false
        }
    });

    let response = server
        .client
        .post(server.url("/api/channels"))
        .json(&channels_config)
        .send()
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        400,
        "Channel ID with double quote should return 400"
    );
}

/// Channel ID 包含非法字符：正斜杠
#[tokio::test]
async fn test_channel_id_with_forward_slash() {
    let server = TestServer::new().await;

    let channels_config = serde_json::json!({
        "mattermost/injected": {
            "enabled": false
        }
    });

    let response = server
        .client
        .post(server.url("/api/channels"))
        .json(&channels_config)
        .send()
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        400,
        "Channel ID with forward slash should return 400"
    );
}

/// Channel ID 超长（> 256 字符）
#[tokio::test]
async fn test_channel_id_too_long() {
    let server = TestServer::new().await;

    let long_id = "channel".to_string() + &"x".repeat(250); // total 257
    let channels_config = serde_json::json!({
        long_id: {
            "enabled": false
        }
    });

    let response = server
        .client
        .post(server.url("/api/channels"))
        .json(&channels_config)
        .send()
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        400,
        "Channel ID > 256 chars should return 400"
    );
}

/// Channel ID 为空（空 key 在 JSON 中会被忽略，所以测试空字符串）
#[tokio::test]
async fn test_channel_id_empty() {
    let server = TestServer::new().await;

    // 空字符串作为 key
    let mut map = serde_json::Map::new();
    map.insert("".to_string(), serde_json::json!({"enabled": false}));
    let channels_config = serde_json::json!(map);

    let response = server
        .client
        .post(server.url("/api/channels"))
        .json(&channels_config)
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 400, "Empty Channel ID should return 400");
}

/// 正常 Channel ID（mattermost、feishu）应该成功
#[tokio::test]
async fn test_channel_id_valid_formats() {
    let server = TestServer::new().await;

    let valid_configs = vec![
        serde_json::json!({"mattermost": {"enabled": false}}),
        serde_json::json!({"feishu": {"enabled": false}}),
        serde_json::json!({"custom-channel": {"enabled": false}}),
        serde_json::json!({"my_channel_1": {"enabled": false}}),
    ];

    for config in valid_configs {
        let response = server
            .client
            .post(server.url("/api/channels"))
            .json(&config)
            .send()
            .await
            .unwrap();

        assert_eq!(
            response.status(),
            200,
            "Valid Channel config should succeed: {:?}",
            config
        );
    }
}
