mod common;
use common::TestServer;
use serde_json::Value;

#[tokio::test]
async fn test_get_status_endpoint() {
    let server = TestServer::new().await;

    let response = server
        .client
        .get(server.url("/api/status"))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200, "Status endpoint should return 200");

    let body: Value = response.json().await.unwrap();

    // 验证响应结构
    assert!(body.get("service").is_some(), "Response should have 'service' field");
    assert!(body.get("healthy").is_some(), "Response should have 'healthy' field");
    assert!(body.get("pid").is_some(), "Response should have 'pid' field");

    // 验证 service 状态是已知的类型之一
    let service = body.get("service").unwrap();
    let valid_statuses = [
        "running",
        "stopped", 
        "starting",
        "stopping",
        "failed",
        "unknown"
    ];
    
    if let Some(status) = service.get("status") {
        let status_str = status.as_str().unwrap();
        assert!(
            valid_statuses.contains(&status_str),
            "Service status '{}' is not a valid status",
            status_str
        );
    } else if let Some(status_str) = service.as_str() {
        // 可能直接是字符串
        assert!(
            valid_statuses.contains(&status_str),
            "Service status '{}' is not a valid status",
            status_str
        );
    }

    // 验证 healthy 是布尔值
    let healthy = body.get("healthy").unwrap();
    assert!(
        healthy.is_boolean(),
        "Healthy field should be a boolean"
    );

    // pid 可以是数字或 null
    let pid = body.get("pid").unwrap();
    assert!(
        pid.is_number() || pid.is_null(),
        "PID should be a number or null"
    );
}

#[tokio::test]
async fn test_status_response_structure() {
    let server = TestServer::new().await;

    let response = server
        .client
        .get(server.url("/api/status"))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200);

    let body: Value = response.json().await.unwrap();

    // 完整结构验证
    let service = &body["service"];
    
    // service 应该是对象形式（当 status = failed 时有 message 字段）
    // 或字符串形式（其他状态）
    if service.is_object() {
        assert!(service.get("status").is_some(), "service object should have status field");
        // 如果是 failed 状态，应该有 message
        if service.get("status").unwrap().as_str() == Some("failed") {
            assert!(service.get("message").is_some(), "failed status should have message");
        }
    } else if service.is_string() {
        let status_str = service.as_str().unwrap();
        assert!(
            ["running", "stopped", "starting", "stopping", "unknown"].contains(&status_str),
            "Invalid string status: {}",
            status_str
        );
    } else {
        panic!("service field should be either object or string");
    }
}

#[tokio::test]
async fn test_status_endpoint_no_auth_required() {
    let server = TestServer::new().await;

    // 状态端点不需要认证，应该直接返回 200
    let response = server
        .client
        .get(server.url("/api/status"))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
}
