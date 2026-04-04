mod common;

use common::TestServer;

#[tokio::test]
async fn test_get_agents() {
    let server = TestServer::new().await;

    let response = server
        .client
        .get(server.url("/api/agents"))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);

    let body: serde_json::Value = response.json().await.unwrap();
    assert!(body.is_object());
    assert!(body.get("defaults").is_some());
    assert!(body.get("list").is_some());
}

#[tokio::test]
async fn test_save_agents() {
    let server = TestServer::new().await;

    let agents_config = serde_json::json!({
        "defaults": {
            "workspace": "/custom/agents/workspace"
        },
        "list": [
            {
                "id": "agent-1",
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

    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["success"], true);

    // Verify the agents config was saved
    let get_response = server
        .client
        .get(server.url("/api/agents"))
        .send()
        .await
        .unwrap();

    assert_eq!(get_response.status(), 200);

    let saved: serde_json::Value = get_response.json().await.unwrap();
    assert_eq!(saved["defaults"]["workspace"], "/custom/agents/workspace");
    assert_eq!(saved["list"][0]["id"], "agent-1");
}
