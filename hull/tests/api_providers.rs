mod common;

use common::TestServer;

#[tokio::test]
async fn test_list_providers_empty() {
    let server = TestServer::new().await;

    let response = server
        .client
        .get(server.url("/api/providers"))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);

    let body: Vec<serde_json::Value> = response.json().await.unwrap();
    assert!(body.is_empty());
}

#[tokio::test]
async fn test_create_provider() {
    let server = TestServer::new().await;

    let provider_data = serde_json::json!({
        "api": "openai-responses",
        "apiKey": "sk-test123",
        "baseUrl": "https://api.test.com",
        "enabled": true,
        "defaultModel": "gpt-4"
    });

    let response = server
        .client
        .post(server.url("/api/providers/test-prov"))
        .json(&provider_data)
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);

    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["success"], true);
    assert_eq!(body["provider_id"], "test-prov");
}

#[tokio::test]
async fn test_get_provider() {
    let server = TestServer::new().await;

    // First create a provider
    let provider_data = serde_json::json!({
        "api": "openai-responses",
        "apiKey": "sk-test123",
        "baseUrl": "https://api.test.com",
        "enabled": true,
        "defaultModel": "gpt-4"
    });

    let create_response = server
        .client
        .post(server.url("/api/providers/get-test-prov"))
        .json(&provider_data)
        .send()
        .await
        .unwrap();

    assert_eq!(create_response.status(), 200);

    // Now get the provider
    let response = server
        .client
        .get(server.url("/api/providers/get-test-prov"))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);

    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["id"], "get-test-prov");
}

#[tokio::test]
async fn test_update_provider() {
    let server = TestServer::new().await;

    // First create a provider
    let provider_data = serde_json::json!({
        "api": "openai-responses",
        "apiKey": "sk-original",
        "baseUrl": "https://api.original.com",
        "enabled": true,
        "defaultModel": "gpt-3.5"
    });

    let _ = server
        .client
        .post(server.url("/api/providers/update-test-prov"))
        .json(&provider_data)
        .send()
        .await
        .unwrap();

    // Update the provider
    let updated_data = serde_json::json!({
        "api": "anthropic-messages",
        "apiKey": "sk-updated",
        "baseUrl": "https://api.updated.com",
        "enabled": true,
        "defaultModel": "claude-3"
    });

    let update_response = server
        .client
        .post(server.url("/api/providers/update-test-prov"))
        .json(&updated_data)
        .send()
        .await
        .unwrap();

    assert_eq!(update_response.status(), 200);

    // Verify the update
    let get_response = server
        .client
        .get(server.url("/api/providers/update-test-prov"))
        .send()
        .await
        .unwrap();

    let body: serde_json::Value = get_response.json().await.unwrap();
    assert_eq!(body["api"], "anthropic-messages");
}

#[tokio::test]
async fn test_delete_provider() {
    let server = TestServer::new().await;

    // First create a provider
    let provider_data = serde_json::json!({
        "api": "openai-responses",
        "apiKey": "sk-test",
        "baseUrl": "https://api.test.com",
        "enabled": true,
        "defaultModel": "gpt-4"
    });

    let _ = server
        .client
        .post(server.url("/api/providers/delete-test-prov"))
        .json(&provider_data)
        .send()
        .await
        .unwrap();

    // Verify it exists
    let list_response = server
        .client
        .get(server.url("/api/providers"))
        .send()
        .await
        .unwrap();

    let providers: Vec<serde_json::Value> = list_response.json().await.unwrap();
    assert!(!providers.is_empty());

    // Delete the provider
    let delete_response = server
        .client
        .delete(server.url("/api/providers/delete-test-prov"))
        .send()
        .await
        .unwrap();

    assert_eq!(delete_response.status(), 200);

    let body: serde_json::Value = delete_response.json().await.unwrap();
    assert_eq!(body["success"], true);

    // Verify it's deleted
    let list_response2 = server
        .client
        .get(server.url("/api/providers"))
        .send()
        .await
        .unwrap();

    let providers2: Vec<serde_json::Value> = list_response2.json().await.unwrap();
    assert!(providers2.is_empty());
}

#[tokio::test]
async fn test_get_nonexistent_provider() {
    let server = TestServer::new().await;

    let response = server
        .client
        .get(server.url("/api/providers/nonexistent"))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 404);
}
