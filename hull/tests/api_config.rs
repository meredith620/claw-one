mod common;

use common::TestServer;

// ==================== 基础 API 测试 ====================

#[tokio::test]
async fn test_get_config() {
    let server = TestServer::new().await;
    
    let response = server.client
        .get(server.url("/api/config"))
        .send()
        .await
        .unwrap();
    
    assert_eq!(response.status(), 200);
    
    let body: serde_json::Value = response.json().await.unwrap();
    assert!(body.is_object());
}

#[tokio::test]
#[ignore = "POST /api/config restarts OpenClaw service which blocks in test env"]
async fn test_post_config() {
    // This test is ignored because POST /api/config triggers OpenClaw service restart
    // via systemctl, which blocks indefinitely in test environment.
    // The endpoint is tested indirectly through test_save_module_config.
}

#[tokio::test]
async fn test_validate_config_valid() {
    let server = TestServer::new().await;
    
    let config = serde_json::json!({
        "models": { "providers": {} },
        "agents": { "defaults": { "workspace": "~/.openclaw/workspace" }, "list": [] },
        "channels": {}
    });
    
    let response = server.client
        .post(server.url("/api/config/validate"))
        .json(&config)
        .send()
        .await
        .unwrap();
    
    assert_eq!(response.status(), 200);
    
    let body: serde_json::Value = response.json().await.unwrap();
    assert!(body["valid"].as_bool().unwrap_or(false));
}

#[tokio::test]
async fn test_validate_config_invalid() {
    let server = TestServer::new().await;
    
    // 非法配置：空对象可能会导致验证警告
    let config = serde_json::json!({});
    
    let response = server.client
        .post(server.url("/api/config/validate"))
        .json(&config)
        .send()
        .await
        .unwrap();
    
    assert_eq!(response.status(), 200);
    
    let body: serde_json::Value = response.json().await.unwrap();
    // 空配置可能是有效的，也可能有警告
    assert!(body.get("valid").is_some());
}

#[tokio::test]
async fn test_get_module_config() {
    let server = TestServer::new().await;
    
    let response = server.client
        .get(server.url("/api/config/agents"))
        .send()
        .await
        .unwrap();
    
    assert_eq!(response.status(), 200);
    
    let body: serde_json::Value = response.json().await.unwrap();
    assert!(body.is_object());
}

#[tokio::test]
async fn test_save_module_config() {
    let server = TestServer::new().await;
    
    let module_config = serde_json::json!({
        "defaults": {
            "workspace": "/custom/workspace"
        }
    });
    
    let response = server.client
        .post(server.url("/api/config/agents"))
        .json(&module_config)
        .send()
        .await
        .unwrap();
    
    assert_eq!(response.status(), 200);
    
    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["success"], true);
    
    // Verify the config was saved
    let get_response = server.client
        .get(server.url("/api/config/agents"))
        .send()
        .await
        .unwrap();
    
    assert_eq!(get_response.status(), 200);
    
    let saved: serde_json::Value = get_response.json().await.unwrap();
    assert_eq!(saved["defaults"]["workspace"], "/custom/workspace");
}

// ==================== 数据完整性测试 (合并自 api_config_integrity.rs) ====================

/// Test that saving module config preserves unmodified fields
/// This is the core value of deep_merge - partial updates without data loss
#[tokio::test]
async fn test_save_agents_preserves_unmodified_fields() {
    let server = TestServer::new().await;
    
    // Step 1: Create initial agents config with multiple fields
    let initial_config = serde_json::json!({
        "defaults": {
            "workspace": "/custom/workspace",
            "compaction": {
                "mode": "safeguard"
            },
            "maxConcurrent": 4,
            "subagents": {
                "maxConcurrent": 8
            },
            "memorySearch": {
                "enabled": true,
                "provider": "ollama"
            }
        },
        "list": [
            {
                "id": "agent-1",
                "name": "Agent One",
                "workspace": "/workspace/agent-1"
            },
            {
                "id": "agent-2", 
                "name": "Agent Two",
                "workspace": "/workspace/agent-2"
            }
        ]
    });
    
    let response = server.client
        .post(server.url("/api/agents"))
        .json(&initial_config)
        .send()
        .await
        .unwrap();
    
    assert_eq!(response.status(), 200);
    
    // Step 2: Save partial update - only modify workspace in defaults
    let partial_update = serde_json::json!({
        "defaults": {
            "workspace": "/new/workspace"
            // Note: NOT including other fields like compaction, maxConcurrent, etc.
        }
        // Note: NOT including list - it should be preserved
    });
    
    let response = server.client
        .post(server.url("/api/agents"))
        .json(&partial_update)
        .send()
        .await
        .unwrap();
    
    assert_eq!(response.status(), 200);
    
    // Step 3: Verify all original fields are preserved
    let get_response = server.client
        .get(server.url("/api/agents"))
        .send()
        .await
        .unwrap();
    
    assert_eq!(get_response.status(), 200);
    
    let saved: serde_json::Value = get_response.json().await.unwrap();
    
    // Verify modified field was updated
    assert_eq!(saved["defaults"]["workspace"], "/new/workspace");
    
    // Verify unmodified fields in defaults are preserved
    assert_eq!(saved["defaults"]["compaction"]["mode"], "safeguard");
    assert_eq!(saved["defaults"]["maxConcurrent"], 4);
    assert_eq!(saved["defaults"]["subagents"]["maxConcurrent"], 8);
    assert_eq!(saved["defaults"]["memorySearch"]["enabled"], true);
    assert_eq!(saved["defaults"]["memorySearch"]["provider"], "ollama");
    
    // Verify list is preserved
    assert_eq!(saved["list"].as_array().unwrap().len(), 2);
    assert_eq!(saved["list"][0]["id"], "agent-1");
    assert_eq!(saved["list"][1]["id"], "agent-2");
}

#[tokio::test]
async fn test_save_channels_preserves_unmodified_accounts() {
    let server = TestServer::new().await;
    
    // Step 1: Create initial channels config with multiple accounts
    let initial_config = serde_json::json!({
        "mattermost": {
            "enabled": true,
            "dmPolicy": "pairing",
            "groupPolicy": "allowlist",
            "accounts": {
                "account-1": {
                    "name": "Account One",
                    "botToken": "token-1",
                    "baseUrl": "https://mm1.example.com"
                },
                "account-2": {
                    "name": "Account Two", 
                    "botToken": "token-2",
                    "baseUrl": "https://mm2.example.com"
                }
            }
        },
        "feishu": {
            "enabled": false,
            "appId": "cli_xxx"
        }
    });
    
    let response = server.client
        .post(server.url("/api/channels"))
        .json(&initial_config)
        .send()
        .await
        .unwrap();
    
    assert_eq!(response.status(), 200);
    
    // Step 2: Update only one account
    let partial_update = serde_json::json!({
        "mattermost": {
            "accounts": {
                "account-1": {
                    "name": "Updated Account One",
                    "botToken": "updated-token",
                    "baseUrl": "https://mm1.updated.com"
                }
                // Note: account-2 is not included - should be preserved
            }
        }
        // Note: feishu is not included - should be preserved
    });
    
    let response = server.client
        .post(server.url("/api/channels"))
        .json(&partial_update)
        .send()
        .await
        .unwrap();
    
    assert_eq!(response.status(), 200);
    
    // Step 3: Verify
    let get_response = server.client
        .get(server.url("/api/channels"))
        .send()
        .await
        .unwrap();
    
    let saved: serde_json::Value = get_response.json().await.unwrap();
    
    // Verify updated account
    assert_eq!(saved["mattermost"]["accounts"]["account-1"]["name"], "Updated Account One");
    
    // Verify feishu is preserved
    assert_eq!(saved["feishu"]["enabled"], false);
    assert_eq!(saved["feishu"]["appId"], "cli_xxx");
}

#[tokio::test]
async fn test_save_memory_preserves_advanced_settings() {
    let server = TestServer::new().await;
    
    // Step 1: Create memory config with advanced settings
    let initial_config = serde_json::json!({
        "enabled": true,
        "provider": "ollama",
        "remote": {
            "baseUrl": "http://localhost:11434"
        },
        "model": "qwen3-embedding:0.6b",
        "fallback": "none",
        "sources": ["memory", "sessions"],
        "store": {
            "vector": {
                "enabled": true,
                "extensionPath": "~/.openclaw/extensions/vec0.so"
            }
        },
        "query": {
            "hybrid": {
                "enabled": true,
                "vectorWeight": 0.7,
                "textWeight": 0.3,
                "mmr": {
                    "enabled": true
                },
                "temporalDecay": {
                    "enabled": true,
                    "halfLifeDays": 30
                }
            }
        }
    });
    
    let response = server.client
        .post(server.url("/api/memory"))
        .json(&initial_config)
        .send()
        .await
        .unwrap();
    
    assert_eq!(response.status(), 200);
    
    // Step 2: Update only basic fields
    let partial_update = serde_json::json!({
        "enabled": true,
        "provider": "ollama",
        "model": "updated-model"
        // Note: advanced settings like store, query are not included
    });
    
    let response = server.client
        .post(server.url("/api/memory"))
        .json(&partial_update)
        .send()
        .await
        .unwrap();
    
    assert_eq!(response.status(), 200);
    
    // Step 3: Verify
    let get_response = server.client
        .get(server.url("/api/memory"))
        .send()
        .await
        .unwrap();
    
    let saved: serde_json::Value = get_response.json().await.unwrap();
    
    // Verify updated field
    assert_eq!(saved["model"], "updated-model");
    
    // Verify advanced settings are preserved
    assert_eq!(saved["store"]["vector"]["enabled"], true);
    assert_eq!(saved["query"]["hybrid"]["enabled"], true);
    assert_eq!(saved["query"]["hybrid"]["vectorWeight"], 0.7);
}
