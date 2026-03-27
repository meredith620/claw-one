mod common;

use common::TestServer;

#[tokio::test]
async fn test_get_memory_initial() {
    let server = TestServer::new().await;
    
    let response = server.client
        .get(server.url("/api/memory"))
        .send()
        .await
        .unwrap();
    
    assert_eq!(response.status(), 200);
    
    let body: serde_json::Value = response.json().await.unwrap();
    // Initial memory config should be null since baseline doesn't include it
    assert!(body.is_null() || body.is_object());
}

#[tokio::test]
async fn test_save_memory() {
    let server = TestServer::new().await;
    
    let memory_config = serde_json::json!({
        "enabled": true,
        "provider": "ollama",
        "model": "test-memory-model"
    });
    
    let response = server.client
        .post(server.url("/api/memory"))
        .json(&memory_config)
        .send()
        .await
        .unwrap();
    
    assert_eq!(response.status(), 200);
    
    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["success"], true);
    
    // Verify the memory config was saved
    let get_response = server.client
        .get(server.url("/api/memory"))
        .send()
        .await
        .unwrap();
    
    assert_eq!(get_response.status(), 200);
    
    let saved: serde_json::Value = get_response.json().await.unwrap();
    assert_eq!(saved["enabled"], true);
    assert_eq!(saved["provider"], "ollama");
    assert_eq!(saved["model"], "test-memory-model");
}
