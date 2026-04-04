mod common;

use common::TestServer;
use std::time::Duration;
use tokio::time::sleep;

/// 测试：API 保存 memory 配置后，文件系统上的 openclaw.json 实际内容是否正确
///
/// 这个测试用于暴露 Bug #1: 页面修改 memory 配置点击保存后，
/// 目标 openclaw.json 配置文件没有变化
#[tokio::test]
async fn test_memory_config_file_content_matches_api_save() {
    let server = TestServer::new().await;

    // 构造要保存的 memory 配置
    let memory_config = serde_json::json!({
        "enabled": true,
        "provider": "ollama",
        "remote": {
            "baseUrl": "http://localhost:11434"
        },
        "model": "test-memory-model-unique-id",
        "fallback": "none",
        "sources": ["memory", "sessions"]
    });

    // 1. 通过 API 保存 memory 配置
    let response = server
        .client
        .post(server.url("/api/memory"))
        .json(&memory_config)
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200, "API 保存请求应该成功");

    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["success"], true, "API 应该返回 success=true");

    // 等待文件系统同步（给系统一点时间写入）
    sleep(Duration::from_millis(100)).await;

    // 2. 通过 API 读取配置，验证 API 层数据正确
    let get_response = server
        .client
        .get(server.url("/api/memory"))
        .send()
        .await
        .unwrap();

    assert_eq!(get_response.status(), 200);
    let api_memory: serde_json::Value = get_response.json().await.unwrap();
    assert_eq!(
        api_memory["model"], "test-memory-model-unique-id",
        "API 返回的 model 应该匹配"
    );

    // 3. ✅ 关键验证：直接读取文件系统上的 openclaw.json
    let config_file_path = server.temp_dir.path().join("openclaw.json");
    let file_content = tokio::fs::read_to_string(&config_file_path)
        .await
        .expect("应该能读取 openclaw.json 文件");

    let file_json: serde_json::Value =
        serde_json::from_str(&file_content).expect("openclaw.json 应该是合法的 JSON");

    // 验证文件中的 memory 配置与保存的值一致
    let file_memory = file_json
        .get("agents")
        .and_then(|a| a.get("defaults"))
        .and_then(|d| d.get("memorySearch"));

    assert!(
        file_memory.is_some(),
        "Bug #1 暴露: 文件中的 agents.defaults.memorySearch 不存在！\n文件内容: {}",
        serde_json::to_string_pretty(&file_json).unwrap()
    );

    let file_memory = file_memory.unwrap();
    assert_eq!(
        file_memory["model"], "test-memory-model-unique-id",
        "Bug #1 暴露: 文件中 memory.model 不匹配！\n文件中的值: {:?}\n期望的值: {:?}",
        file_memory["model"], "test-memory-model-unique-id"
    );

    assert_eq!(
        file_memory["enabled"], true,
        "文件中 memory.enabled 应该为 true"
    );

    assert_eq!(
        file_memory["provider"], "ollama",
        "文件中 memory.provider 应该为 ollama"
    );

    println!("✅ test_memory_config_file_content_matches_api_save 通过！");
    println!("   文件路径: {:?}", config_file_path);
    println!("   文件内容验证成功");
}

/// 测试：API 保存 agents 配置后，文件系统内容是否正确
#[tokio::test]
async fn test_agents_config_file_content_matches_api_save() {
    let server = TestServer::new().await;

    let agents_config = serde_json::json!({
        "defaults": {
            "workspace": "/custom/workspace-path-test",
            "maxConcurrent": 8
        },
        "list": [
            {"id": "test-agent-1", "name": "Test Agent"}
        ]
    });

    // 1. 通过 API 保存 agents 配置
    let response = server
        .client
        .post(server.url("/api/agents"))
        .json(&agents_config)
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["success"], true);

    sleep(Duration::from_millis(100)).await;

    // 2. 直接读取文件验证
    let config_file_path = server.temp_dir.path().join("openclaw.json");
    let file_content = tokio::fs::read_to_string(&config_file_path)
        .await
        .expect("应该能读取 openclaw.json 文件");

    let file_json: serde_json::Value =
        serde_json::from_str(&file_content).expect("openclaw.json 应该是合法的 JSON");

    let file_workspace = file_json
        .get("agents")
        .and_then(|a| a.get("defaults"))
        .and_then(|d| d.get("workspace"))
        .and_then(|w| w.as_str());

    assert_eq!(
        file_workspace,
        Some("/custom/workspace-path-test"),
        "文件中 agents.defaults.workspace 应该匹配保存的值！\n文件内容: {}",
        serde_json::to_string_pretty(&file_json).unwrap()
    );

    println!("✅ test_agents_config_file_content_matches_api_save 通过！");
}

/// 测试：API 保存 channels 配置后，文件系统内容是否正确
#[tokio::test]
async fn test_channels_config_file_content_matches_api_save() {
    let server = TestServer::new().await;

    let channels_config = serde_json::json!({
        "mattermost": {
            "enabled": true,
            "dmPolicy": "allow"
        }
    });

    // 1. 通过 API 保存 channels 配置
    let response = server
        .client
        .post(server.url("/api/channels"))
        .json(&channels_config)
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["success"], true);

    sleep(Duration::from_millis(100)).await;

    // 2. 直接读取文件验证
    let config_file_path = server.temp_dir.path().join("openclaw.json");
    let file_content = tokio::fs::read_to_string(&config_file_path)
        .await
        .expect("应该能读取 openclaw.json 文件");

    let file_json: serde_json::Value =
        serde_json::from_str(&file_content).expect("openclaw.json 应该是合法的 JSON");

    let mattermost_enabled = file_json
        .get("channels")
        .and_then(|c| c.get("mattermost"))
        .and_then(|m| m.get("enabled"))
        .and_then(|e| e.as_bool());

    assert_eq!(
        mattermost_enabled,
        Some(true),
        "文件中 channels.mattermost.enabled 应该为 true！\n文件内容: {}",
        serde_json::to_string_pretty(&file_json).unwrap()
    );

    println!("✅ test_channels_config_file_content_matches_api_save 通过！");
}

/// 测试：多次保存后，文件内容保持一致
#[tokio::test]
async fn test_multiple_saves_file_consistency() {
    let server = TestServer::new().await;

    // 第一次保存
    let memory_config_1 = serde_json::json!({
        "enabled": true,
        "model": "model-version-1"
    });

    let response = server
        .client
        .post(server.url("/api/memory"))
        .json(&memory_config_1)
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200);

    sleep(Duration::from_millis(50)).await;

    // 第二次保存（更新）
    let memory_config_2 = serde_json::json!({
        "enabled": true,
        "model": "model-version-2"
    });

    let response = server
        .client
        .post(server.url("/api/memory"))
        .json(&memory_config_2)
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200);

    sleep(Duration::from_millis(50)).await;

    // 读取文件验证最终状态
    let config_file_path = server.temp_dir.path().join("openclaw.json");
    let file_content = tokio::fs::read_to_string(&config_file_path)
        .await
        .expect("应该能读取 openclaw.json 文件");

    let file_json: serde_json::Value =
        serde_json::from_str(&file_content).expect("openclaw.json 应该是合法的 JSON");

    let file_model = file_json
        .get("agents")
        .and_then(|a| a.get("defaults"))
        .and_then(|d| d.get("memorySearch"))
        .and_then(|m| m.get("model"))
        .and_then(|m| m.as_str());

    assert_eq!(
        file_model,
        Some("model-version-2"),
        "多次保存后，文件应该包含最后一次保存的值！\n文件内容: {}",
        serde_json::to_string_pretty(&file_json).unwrap()
    );

    println!("✅ test_multiple_saves_file_consistency 通过！");
}
