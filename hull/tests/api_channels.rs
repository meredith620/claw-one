mod common;

use common::TestServer;

#[tokio::test]
async fn test_get_channels() {
    let server = TestServer::new().await;

    let response = server
        .client
        .get(server.url("/api/channels"))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);

    let body: serde_json::Value = response.json().await.unwrap();
    assert!(body.is_object());
}

#[tokio::test]
async fn test_save_channels() {
    let server = TestServer::new().await;

    let channels_config = serde_json::json!({
        "mattermost": {
            "enabled": true,
            "dmPolicy": "allow",
            "accounts": {}
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

    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["success"], true);

    // Verify the channels config was saved
    let get_response = server
        .client
        .get(server.url("/api/channels"))
        .send()
        .await
        .unwrap();

    assert_eq!(get_response.status(), 200);

    let saved: serde_json::Value = get_response.json().await.unwrap();
    assert_eq!(saved["mattermost"]["enabled"], true);
    assert_eq!(saved["mattermost"]["dmPolicy"], "allow");
}
