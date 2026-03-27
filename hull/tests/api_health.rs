mod common;

use common::TestServer;

#[tokio::test]
async fn test_health_endpoint() {
    let server = TestServer::new().await;
    
    let response = server.client
        .get(server.url("/api/health"))
        .send()
        .await
        .unwrap();
    
    assert_eq!(response.status(), 200);
    
    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["status"], "ok");
    assert!(body["version"].as_str().is_some());
}
