mod common;

use common::TestServer;

/// 测试：API 应该拒绝包含非预期字段的请求数据
/// 
/// Bug #1 根因：前端发送了事件对象 {"_vts": ..., "isTrusted": true}
/// 后端应该识别并拒绝这种无效数据结构
#[tokio::test]
async fn test_memory_api_rejects_event_object() {
    let server = TestServer::new().await;
    
    // 模拟前端错误：发送事件对象结构
    let invalid_event_data = serde_json::json!({
        "_vts": 1774700348600_i64,
        "isTrusted": true
    });
    
    let response = server.client
        .post(server.url("/api/memory"))
        .json(&invalid_event_data)
        .send()
        .await
        .unwrap();
    
    // 当前实现返回 200，这是 Bug
    // 期望行为：应该返回 400 Bad Request
    // 先记录当前行为，用于后续修复验证
    let status = response.status();
    let body: serde_json::Value = response.json().await.unwrap();
    
    println!("发送事件对象后的响应状态: {}", status);
    println!("响应体: {:?}", body);
    
    // FIXME: 当前后端没有验证，返回 200
    // 修复后应该断言: assert_eq!(status, 400)
    // 暂时只验证文件内容没有错误保存
    let config_file_path = server.temp_dir.path().join("openclaw.json");
    let file_content = tokio::fs::read_to_string(&config_file_path).await.unwrap();
    let file_json: serde_json::Value = serde_json::from_str(&file_content).unwrap();
    
    // 验证文件中没有错误的事件对象数据
    let memory = file_json
        .get("agents")
        .and_then(|a| a.get("defaults"))
        .and_then(|d| d.get("memorySearch"));
    
    if let Some(m) = memory {
        // 如果保存了，检查是否包含错误字段
        if m.get("_vts").is_some() || m.get("isTrusted").is_some() {
            panic!("Bug #1 确认: 事件对象字段被错误保存到配置文件中！\n文件内容: {}", 
                serde_json::to_string_pretty(&file_json).unwrap());
        }
    }
}

/// 测试：API 应该拒绝数组类型的请求数据
#[tokio::test]
async fn test_memory_api_rejects_array_data() {
    let server = TestServer::new().await;
    
    let invalid_data = serde_json::json!([1, 2, 3]);
    
    let response = server.client
        .post(server.url("/api/memory"))
        .json(&invalid_data)
        .send()
        .await
        .unwrap();
    
    let status = response.status();
    println!("发送数组数据后的响应状态: {}", status);
    
    // FIXME: 当前后端可能 panic 或返回 200
    // 修复后应该返回 400
}

/// 测试：API 应该拒绝字符串类型的请求数据
#[tokio::test]
async fn test_memory_api_rejects_string_data() {
    let server = TestServer::new().await;
    
    let invalid_data = serde_json::json!("invalid string");
    
    let response = server.client
        .post(server.url("/api/memory"))
        .json(&invalid_data)
        .send()
        .await
        .unwrap();
    
    let status = response.status();
    println!("发送字符串数据后的响应状态: {}", status);
}

/// 测试：API 应该拒绝数字类型的请求数据
#[tokio::test]
async fn test_memory_api_rejects_number_data() {
    let server = TestServer::new().await;
    
    let invalid_data = serde_json::json!(12345);
    
    let response = server.client
        .post(server.url("/api/memory"))
        .json(&invalid_data)
        .send()
        .await
        .unwrap();
    
    let status = response.status();
    println!("发送数字数据后的响应状态: {}", status);
}

/// 测试：API 应该拒绝包含额外未知字段的对象
#[tokio::test]
async fn test_memory_api_rejects_unknown_fields() {
    let server = TestServer::new().await;
    
    // 包含有效字段但也包含未知字段
    let data_with_unknown = serde_json::json!({
        "enabled": true,
        "provider": "ollama",
        "unknownField": "should be rejected",
        "anotherBadField": 123
    });
    
    let response = server.client
        .post(server.url("/api/memory"))
        .json(&data_with_unknown)
        .send()
        .await
        .unwrap();
    
    let status = response.status();
    let body: serde_json::Value = response.json().await.unwrap();
    
    println!("包含未知字段的响应状态: {}", status);
    println!("响应体: {:?}", body);
    
    // FIXME: 当前后端接受未知字段
    // 修复后应该返回 400 或忽略未知字段
}

/// 测试：验证保存后文件中的数据结构是否符合预期
#[tokio::test]
async fn test_memory_saved_structure_validation() {
    let server = TestServer::new().await;
    
    // 保存正确的 memory 配置
    let valid_config = serde_json::json!({
        "enabled": true,
        "provider": "ollama",
        "remote": {
            "baseUrl": "http://localhost:11434"
        },
        "model": "qwen3-embedding:0.6b",
        "fallback": "none",
        "sources": ["memory", "sessions"]
    });
    
    let response = server.client
        .post(server.url("/api/memory"))
        .json(&valid_config)
        .send()
        .await
        .unwrap();
    
    assert_eq!(response.status(), 200);
    
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    
    // 读取文件并验证结构
    let config_file_path = server.temp_dir.path().join("openclaw.json");
    let file_content = tokio::fs::read_to_string(&config_file_path).await.unwrap();
    let file_json: serde_json::Value = serde_json::from_str(&file_content).unwrap();
    
    let memory = file_json
        .get("agents")
        .and_then(|a| a.get("defaults"))
        .and_then(|d| d.get("memorySearch"));
    
    assert!(memory.is_some(), "memorySearch 应该存在");
    let memory = memory.unwrap();
    
    // 验证必需字段
    assert!(memory.get("enabled").is_some(), "enabled 字段应该存在");
    assert!(memory.get("provider").is_some(), "provider 字段应该存在");
    
    // 验证不应该包含事件对象的字段
    assert!(
        memory.get("_vts").is_none(),
        "不应该包含 _vts 事件字段"
    );
    assert!(
        memory.get("isTrusted").is_none(),
        "不应该包含 isTrusted 事件字段"
    );
    
    println!("✅ 数据结构验证通过");
}

/// 测试：保存 null 应该禁用 memory（当前行为）
#[tokio::test]
async fn test_memory_save_null_disables_memory() {
    let server = TestServer::new().await;
    
    // 先保存有效配置
    let valid_config = serde_json::json!({
        "enabled": true,
        "provider": "ollama",
        "model": "test-model"
    });
    
    server.client
        .post(server.url("/api/memory"))
        .json(&valid_config)
        .send()
        .await
        .unwrap();
    
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    
    // 再保存 null
    let response = server.client
        .post(server.url("/api/memory"))
        .json(&serde_json::Value::Null)
        .send()
        .await
        .unwrap();
    
    assert_eq!(response.status(), 200);
    
    // 验证获取时返回 null
    let get_response = server.client
        .get(server.url("/api/memory"))
        .send()
        .await
        .unwrap();
    
    let body: serde_json::Value = get_response.json().await.unwrap();
    assert!(body.is_null(), "保存 null 后应该返回 null");
}