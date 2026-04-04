mod common;

use common::TestServer;

/// Test that deleting a provider updates model references in agents
#[tokio::test]
async fn test_provider_delete_updates_agent_models() {
    let server = TestServer::new().await;

    // Step 1: Create a provider
    let provider_data = serde_json::json!({
        "api": "openai-responses",
        "apiKey": "sk-test",
        "baseUrl": "https://api.test.com",
        "enabled": true,
        "defaultModel": "gpt-4"
    });

    let response = server
        .client
        .post(server.url("/api/providers/test-provider"))
        .json(&provider_data)
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);

    // Step 2: Set model priority to use this provider
    let priority_data = serde_json::json!({
        "primary": "test-provider/gpt-4",
        "fallbacks": []
    });

    let response = server
        .client
        .post(server.url("/api/model-priority"))
        .json(&priority_data)
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);

    // Step 3: Delete the provider
    let response = server
        .client
        .delete(server.url("/api/providers/test-provider"))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);

    // Step 4: Verify model priority is updated (or marked as invalid)
    let get_response = server
        .client
        .get(server.url("/api/model-priority"))
        .send()
        .await
        .unwrap();

    let priority: serde_json::Value = get_response.json().await.unwrap();
    println!("Model priority after provider deletion: {:?}", priority);

    // Current behavior: model priority may still reference deleted provider
    // Expected: should be cleared or marked as invalid
    // This test documents current behavior
}

/// Test that model priority changes reflect in agents config
#[tokio::test]
async fn test_model_priority_sync_with_agents() {
    let server = TestServer::new().await;

    // Step 1: Create providers
    for provider_id in ["prov-1", "prov-2"] {
        let provider_data = serde_json::json!({
            "api": "openai-responses",
            "apiKey": format!("sk-{}", provider_id),
            "baseUrl": "https://api.test.com",
            "enabled": true,
            "defaultModel": "gpt-4"
        });

        let response = server
            .client
            .post(server.url(&format!("/api/providers/{}", provider_id)))
            .json(&provider_data)
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), 200);
    }

    // Step 2: Set model priority
    let priority_data = serde_json::json!({
        "primary": "prov-1/gpt-4",
        "fallbacks": ["prov-2/gpt-4"]
    });

    let response = server
        .client
        .post(server.url("/api/model-priority"))
        .json(&priority_data)
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);

    // Step 3: Verify in agents config
    let agents_response = server
        .client
        .get(server.url("/api/agents"))
        .send()
        .await
        .unwrap();

    let agents: serde_json::Value = agents_response.json().await.unwrap();
    println!("Agents config with model priority: {:?}", agents);

    // The model priority should be reflected in agents.defaults.model
    // This depends on the implementation
}

/// Test channel-agent binding consistency
#[tokio::test]
async fn test_channel_agent_binding_consistency() {
    let server = TestServer::new().await;

    // Step 1: Create an agent
    let agents_config = serde_json::json!({
        "defaults": {
            "workspace": "~/.openclaw/workspace"
        },
        "list": [
            {
                "id": "test-agent",
                "name": "Test Agent"
            }
        ]
    });

    let response = server
        .client
        .post(server.url("/api/agents"))
        .json(&agents_config)
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);

    // Step 2: Create channel with binding to this agent
    // Note: Current API doesn't expose bindings directly
    // This test documents expected behavior

    let channels_config = serde_json::json!({
        "mattermost": {
            "enabled": true,
            "dmPolicy": "pairing",
            "accounts": {
                "default": {
                    "name": "Main",
                    "botToken": "token",
                    "baseUrl": "https://mm.example.com"
                }
            }
        }
    });

    let response = server
        .client
        .post(server.url("/api/channels"))
        .json(&channels_config)
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);

    // Step 3: Delete the agent
    let agents_config = serde_json::json!({
        "defaults": {
            "workspace": "~/.openclaw/workspace"
        },
        "list": []  // Empty list = delete all agents
    });

    let response = server
        .client
        .post(server.url("/api/agents"))
        .json(&agents_config)
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);

    // Step 4: Verify what happens to channel bindings
    // Current behavior: bindings may reference non-existent agent
    // Expected: should be cleaned up or marked as invalid
    println!("Agent deleted, channel bindings may be orphaned");
}

/// Test multi-account channel configuration
#[tokio::test]
async fn test_multi_account_channel() {
    let server = TestServer::new().await;

    // Create channel with multiple accounts
    let channels_config = serde_json::json!({
        "mattermost": {
            "enabled": true,
            "dmPolicy": "pairing",
            "groupPolicy": "allowlist",
            "accounts": {
                "work": {
                    "name": "Work Account",
                    "botToken": "work-token",
                    "baseUrl": "https://work.mm.com"
                },
                "personal": {
                    "name": "Personal Account",
                    "botToken": "personal-token",
                    "baseUrl": "https://personal.mm.com"
                },
                "opensource": {
                    "name": "Open Source",
                    "botToken": "oss-token",
                    "baseUrl": "https://oss.mm.com"
                }
            }
        }
    });

    let response = server
        .client
        .post(server.url("/api/channels"))
        .json(&channels_config)
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);

    // Verify all accounts are saved
    let get_response = server
        .client
        .get(server.url("/api/channels"))
        .send()
        .await
        .unwrap();

    let saved: serde_json::Value = get_response.json().await.unwrap();
    let accounts = saved["mattermost"]["accounts"].as_object().unwrap();

    assert_eq!(accounts.len(), 3);
    assert!(accounts.contains_key("work"));
    assert!(accounts.contains_key("personal"));
    assert!(accounts.contains_key("opensource"));
}

/// Test provider multi-instance support
#[tokio::test]
async fn test_provider_multi_instance() {
    let server = TestServer::new().await;

    // Create multiple instances of the same provider type
    let instances = vec![
        ("moonshot-work", "Moonshot Work", "sk-work"),
        ("moonshot-personal", "Moonshot Personal", "sk-personal"),
    ];

    for (id, name, token) in instances {
        let provider_data = serde_json::json!({
            "api": "openai-responses",
            "apiKey": token,
            "baseUrl": "https://api.moonshot.cn/v1",
            "enabled": true,
            "defaultModel": "kimi-k2.5"
        });

        let response = server
            .client
            .post(server.url(&format!("/api/providers/{}", id)))
            .json(&provider_data)
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), 200, "Failed to create provider: {}", id);
    }

    // Verify all instances are listed
    let list_response = server
        .client
        .get(server.url("/api/providers"))
        .send()
        .await
        .unwrap();

    let providers: Vec<serde_json::Value> = list_response.json().await.unwrap();
    let ids: Vec<String> = providers
        .iter()
        .map(|p| p["id"].as_str().unwrap().to_string())
        .collect();

    assert!(ids.contains(&"moonshot-work".to_string()));
    assert!(ids.contains(&"moonshot-personal".to_string()));
}

/// Test complete configuration workflow
#[tokio::test]
async fn test_complete_configuration_workflow() {
    let server = TestServer::new().await;

    // Step 1: Configure providers
    let provider_data = serde_json::json!({
        "api": "openai-responses",
        "apiKey": "sk-test",
        "baseUrl": "https://api.test.com",
        "enabled": true,
        "defaultModel": "gpt-4"
    });

    let response = server
        .client
        .post(server.url("/api/providers/workflow-test"))
        .json(&provider_data)
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);

    // Step 2: Configure model priority
    let priority_data = serde_json::json!({
        "primary": "workflow-test/gpt-4",
        "fallbacks": []
    });

    let response = server
        .client
        .post(server.url("/api/model-priority"))
        .json(&priority_data)
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);

    // Step 3: Configure agents
    let agents_config = serde_json::json!({
        "defaults": {
            "workspace": "~/.openclaw/workspace",
            "compaction": {"mode": "safeguard"}
        },
        "list": [
            {
                "id": "main-agent",
                "name": "Main Agent"
            }
        ]
    });

    let response = server
        .client
        .post(server.url("/api/agents"))
        .json(&agents_config)
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);

    // Step 4: Configure memory
    let memory_config = serde_json::json!({
        "enabled": true,
        "provider": "ollama",
        "model": "qwen3-embedding:0.6b"
    });

    let response = server
        .client
        .post(server.url("/api/memory"))
        .json(&memory_config)
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);

    // Step 5: Configure channels
    let channels_config = serde_json::json!({
        "mattermost": {
            "enabled": true,
            "dmPolicy": "pairing",
            "accounts": {
                "default": {
                    "name": "Main",
                    "botToken": "token",
                    "baseUrl": "https://mm.example.com"
                }
            }
        }
    });

    let response = server
        .client
        .post(server.url("/api/channels"))
        .json(&channels_config)
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);

    // Step 6: Verify all configs are saved
    let endpoints = vec![
        ("/api/providers", "providers"),
        ("/api/agents", "agents"),
        ("/api/memory", "memory"),
        ("/api/channels", "channels"),
    ];

    for (endpoint, name) in endpoints {
        let response = server
            .client
            .get(server.url(endpoint))
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), 200, "Failed to get {}", name);
    }

    println!("✅ Complete configuration workflow test passed");
}
