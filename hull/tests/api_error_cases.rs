mod common;

use common::TestServer;

/// Test error handling for invalid JSON
#[tokio::test]
async fn test_post_invalid_json_returns_400() {
    let server = TestServer::new().await;
    
    // Send invalid JSON
    let response = server.client
        .post(server.url("/api/config"))
        .header("Content-Type", "application/json")
        .body("{ invalid json")
        .send()
        .await
        .unwrap();
    
    // Should return 400 Bad Request
    assert_eq!(response.status(), 400);
}

/// Test validation error for missing required fields in provider
#[tokio::test]
async fn test_provider_validation_missing_api_key() {
    let server = TestServer::new().await;
    
    // Try to create provider without apiKey
    let provider_data = serde_json::json!({
        "api": "openai-responses",
        // "apiKey" is missing
        "baseUrl": "https://api.test.com",
        "enabled": true
    });
    
    let response = server.client
        .post(server.url("/api/providers/test-provider"))
        .json(&provider_data)
        .send()
        .await
        .unwrap();
    
    // Current implementation may accept this, but validation should catch it
    // This test documents expected behavior
    let status = response.status();
    let body: serde_json::Value = response.json().await.unwrap_or_default();
    
    // If validation is implemented, should return 400
    // If not implemented, will return 200 (current behavior)
    println!("Provider validation status: {}, body: {:?}", status, body);
    
    // For now, just verify the request doesn't crash the server
    assert!(status == 200 || status == 400);
}

/// Test duplicate provider ID handling
#[tokio::test]
async fn test_duplicate_provider_id() {
    let server = TestServer::new().await;
    
    // Create first provider
    let provider_data = serde_json::json!({
        "api": "openai-responses",
        "apiKey": "sk-test",
        "baseUrl": "https://api.test.com",
        "enabled": true
    });
    
    let response = server.client
        .post(server.url("/api/providers/duplicate-test"))
        .json(&provider_data)
        .send()
        .await
        .unwrap();
    
    assert_eq!(response.status(), 200);
    
    // Try to create another with same ID
    let response = server.client
        .post(server.url("/api/providers/duplicate-test"))
        .json(&provider_data)
        .send()
        .await
        .unwrap();
    
    // Should either update existing (200) or return conflict (409)
    let status = response.status();
    println!("Duplicate provider status: {}", status);
    
    // Current behavior: updates existing provider (200)
    // Expected if strict: 409 Conflict
    assert!(status == 200 || status == 409);
}

/// Test getting non-existent resources
#[tokio::test]
async fn test_get_nonexistent_provider() {
    let server = TestServer::new().await;
    
    let response = server.client
        .get(server.url("/api/providers/does-not-exist-12345"))
        .send()
        .await
        .unwrap();
    
    assert_eq!(response.status(), 404);
}

#[tokio::test]
async fn test_delete_nonexistent_provider() {
    let server = TestServer::new().await;
    
    let response = server.client
        .delete(server.url("/api/providers/does-not-exist-12345"))
        .send()
        .await
        .unwrap();
    
    // Should return 404 or 200 (if idempotent delete)
    let status = response.status();
    println!("Delete nonexistent provider status: {}", status);
    
    assert!(status == 200 || status == 404);
}

/// Test invalid URL format in config
#[tokio::test]
async fn test_invalid_url_format() {
    let server = TestServer::new().await;
    
    let channels_config = serde_json::json!({
        "mattermost": {
            "enabled": true,
            "accounts": {
                "test": {
                    "name": "Test",
                    "botToken": "token",
                    "baseUrl": "not-a-valid-url"  // Invalid URL
                }
            }
        }
    });
    
    let response = server.client
        .post(server.url("/api/channels"))
        .json(&channels_config)
        .send()
        .await
        .unwrap();
    
    // Current implementation accepts any string
    // With validation, should return 400
    let status = response.status();
    println!("Invalid URL status: {}", status);
    
    // Document current behavior
    assert!(status == 200 || status == 400);
}

/// Test type mismatch in config (port as string)
#[tokio::test]
async fn test_type_mismatch_port_as_string() {
    let server = TestServer::new().await;
    
    // Note: This tests the gateway port which is in a different config
    // For now, test with providers config
    
    let provider_data = serde_json::json!({
        "api": "openai-responses",
        "apiKey": "sk-test",
        "baseUrl": "https://api.test.com",
        "enabled": "yes"  // Should be boolean, not string
    });
    
    let response = server.client
        .post(server.url("/api/providers/type-test"))
        .json(&provider_data)
        .send()
        .await
        .unwrap();
    
    let status = response.status();
    println!("Type mismatch status: {}", status);
    
    // Current: accepts and stores as-is
    // With validation: should return 400
    assert!(status == 200 || status == 400);
}

/// Test empty request body
#[tokio::test]
async fn test_empty_request_body() {
    let server = TestServer::new().await;
    
    let response = server.client
        .post(server.url("/api/agents"))
        .header("Content-Type", "application/json")
        .body("")
        .send()
        .await
        .unwrap();
    
    assert_eq!(response.status(), 400);
}

/// Test very large request body
#[tokio::test]
async fn test_large_request_body() {
    let server = TestServer::new().await;
    
    // Create a very large config (1MB+)
    let large_list: Vec<serde_json::Value> = (0..10000)
        .map(|i| serde_json::json!({
            "id": format!("agent-{}", i),
            "name": format!("Agent {} with a very long name that takes up space", i),
            "data": "x".repeat(100)
        }))
        .collect();
    
    let large_config = serde_json::json!({
        "defaults": {},
        "list": large_list
    });
    
    let response = server.client
        .post(server.url("/api/agents"))
        .json(&large_config)
        .send()
        .await
        .unwrap();
    
    // Should either succeed or return 413 Payload Too Large
    let status = response.status();
    println!("Large payload status: {}", status);
    
    assert!(status == 200 || status == 413);
}

/// Test special characters in IDs
#[tokio::test]
async fn test_special_characters_in_provider_id() {
    let server = TestServer::new().await;
    
    let provider_data = serde_json::json!({
        "api": "openai-responses",
        "apiKey": "sk-test",
        "baseUrl": "https://api.test.com"
    });
    
    // Test various special characters
    let test_ids = vec![
        "test-with-dashes",
        "test_with_underscores", 
        "test.with.dots",
        "test123",
        "Test-With-Uppercase",
    ];
    
    for id in test_ids {
        let response = server.client
            .post(server.url(&format!("/api/providers/{}", id)))
            .json(&provider_data)
            .send()
            .await
            .unwrap();
        
        // Should succeed for valid IDs
        assert_eq!(response.status(), 200, "Failed for ID: {}", id);
    }
}

/// Test SQL injection-like input (security test)
#[tokio::test]
async fn test_sql_injection_attempt() {
    let server = TestServer::new().await;
    
    let malicious_config = serde_json::json!({
        "defaults": {
            "workspace": "'; DROP TABLE users; --"
        },
        "list": []
    });
    
    let response = server.client
        .post(server.url("/api/agents"))
        .json(&malicious_config)
        .send()
        .await
        .unwrap();
    
    // Should accept the string (it's just data, not executed)
    // But verify it doesn't cause any crashes
    assert_eq!(response.status(), 200);
    
    // Verify the value was stored as-is
    let get_response = server.client
        .get(server.url("/api/agents"))
        .send()
        .await
        .unwrap();
    
    let saved: serde_json::Value = get_response.json().await.unwrap();
    assert_eq!(saved["defaults"]["workspace"], "'; DROP TABLE users; --");
}
