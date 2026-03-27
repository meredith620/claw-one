use std::sync::Arc;
use tempfile::TempDir;
use tokio::net::TcpListener;

pub struct TestServer {
    pub base_url: String,
    pub client: reqwest::Client,
    pub temp_dir: TempDir,
    // keep the server handle alive
    _server_handle: tokio::task::JoinHandle<()>,
}

impl TestServer {
    pub async fn new() -> Self {
        // 1. 创建临时目录
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("openclaw.json");
        
        // 2. 写入基线配置
        let baseline = serde_json::json!({
            "models": { "providers": {} },
            "agents": { "defaults": { "workspace": "~/.openclaw/workspace" }, "list": [] },
            "channels": {}
        });
        std::fs::write(&config_path, serde_json::to_string_pretty(&baseline).unwrap()).unwrap();
        
        // 3. 创建 ConfigManager（用 with_config_path）
        let config_manager = Arc::new(claw_one::ConfigManager::with_config_path(config_path));
        
        // 4. 创建 StateManager
        // StateManager 需要 RuntimeManager，但测试中不需要真实的 openclaw 服务
        // 用默认的 OpenClawConfig
        let openclaw_config = claw_one::settings::OpenClawConfig::default();
        let state_manager = Arc::new(claw_one::StateManager::new(config_manager.clone(), &openclaw_config));
        
        // 5. 构建路由
        let app = claw_one::build_api_router(config_manager, state_manager);
        
        // 6. 绑定随机端口
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let base_url = format!("http://127.0.0.1:{}", addr.port());
        
        // 7. 启动 server
        let handle = tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });
        
        // 8. 等待 server 就绪
        let client = reqwest::Client::new();
        let mut server_ready = false;
        for i in 0..50 {
            if client.get(format!("{}/api/health", &base_url)).send().await.is_ok() {
                server_ready = true;
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        }
        
        if !server_ready {
            panic!("Server failed to start within 2.5 seconds");
        }
        
        Self {
            base_url: base_url.to_string(),
            client,
            temp_dir,
            _server_handle: handle,
        }
    }
    
    pub fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }
}