---
# HARNESS METADATA
# type: specification
# part-of: harness-architecture
# scope: testing
# managed-by: harness-system
# do-not-modify-manually: 如需修改请遵循 harness-evolution.spec.md
# version: 1.0
# created: 2026-04-01
---

# 测试开发规范

> **适用于:** 新增或修改测试代码
> 
> ⚠️ **HARNESS 文件**: 本文档属于 Harness 架构，修改需谨慎

## 测试分层架构

```
Layer 1: 单元测试 ─────────────────────────
  范围: 单个模块内部逻辑
  文件: src/*.rs 中的 #[cfg(test)]
  耗时: ~10秒
  命令: cargo test --lib
  
Layer 2: 集成测试 ─────────────────────────
  范围: API 端点 + 模块交互
  文件: tests/*.rs
  耗时: ~20秒
  命令: cargo test --test api_*
  
Layer 3: E2E 测试 ─────────────────────────
  范围: 完整用户流程
  文件: e2e/tests/*.sh
  耗时: ~2-3分钟
  命令: make test-e2e
```

## 单元测试规范

### 测试组织

```rust
// src/config.rs
#[cfg(test)]
mod tests {
    use super::*;
    
    // 每个测试独立
    #[tokio::test]
    async fn test_config_read_write() {
        let temp_dir = TempDir::new().unwrap();
        let manager = ConfigManager::new(temp_dir.path()).await.unwrap();
        
        // Arrange
        let config = json!({ "port": 8080 });
        
        // Act
        manager.save_config(&config).await.unwrap();
        let read = manager.read_config().await.unwrap();
        
        // Assert
        assert_eq!(read, config);
    }
    
    // 错误场景
    #[tokio::test]
    async fn test_config_invalid_json() {
        let manager = create_test_manager().await;
        
        let result = manager.parse_config("{invalid}");
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ConfigError::ParseError(_)));
    }
}
```

### Mock 使用

```rust
// 外部依赖使用 mock
#[async_trait]
pub trait Runtime: Send + Sync {
    async fn health(&self) -> bool;
}

pub struct MockRuntime {
    health_response: bool,
}

#[async_trait]
impl Runtime for MockRuntime {
    async fn health(&self) -> bool {
        self.health_response
    }
}

#[test]
fn test_with_mock() {
    let mock = MockRuntime { health_response: true };
    let service = Service::new(mock);
    
    assert!(service.check().await);
}
```

## 集成测试规范

### 测试服务器

```rust
// tests/common/mod.rs
pub struct TestServer {
    addr: SocketAddr,
    client: reqwest::Client,
    temp_dir: TempDir,
}

impl TestServer {
    pub async fn new() -> Self {
        let temp_dir = TempDir::new().unwrap();
        
        // 初始化 AppState（使用临时目录）
        let config_manager = ConfigManager::new(temp_dir.path()).await.unwrap();
        let state_manager = StateManager::new().await.unwrap();
        let runtime = Arc::new(Mutex::new(MockRuntime::new()));
        
        let app = create_app(config_manager, state_manager, runtime);
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        
        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });
        
        // 等待服务启动
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        TestServer {
            addr,
            client: reqwest::Client::new(),
            temp_dir,
        }
    }
    
    pub async fn get(&self, path: &str) -> Response {
        self.client.get(format!("http://{}{}", self.addr, path))
            .send().await.unwrap()
    }
    
    pub async fn post(&self, path: &str, body: Value) -> Response {
        self.client.post(format!("http://{}{}", self.addr, path))
            .json(&body)
            .send().await.unwrap()
    }
}
```

### 测试文件命名

```
tests/
├── common/
│   └── mod.rs          # TestServer 共享代码
├── api_health.rs       # /api/health 测试
├── api_config.rs       # /api/config 测试
├── api_providers.rs    # Provider CRUD 测试
├── api_error_cases.rs  # 错误场景测试
└── api_module_interaction.rs  # 模块联动测试
```

### API 测试模板

```rust
// tests/api_config.rs
mod common;
use common::TestServer;

#[tokio::test]
async fn test_get_config_success() {
    let server = TestServer::new().await;
    
    let response = server.get("/api/config").await;
    
    assert_eq!(response.status(), 200);
    let config: Value = response.json().await.unwrap();
    assert!(config.get("gateway").is_some());
}

#[tokio::test]
async fn test_post_config_validation_error() {
    let server = TestServer::new().await;
    let invalid_config = json!({ "port": "not-a-number" });
    
    let response = server.post("/api/config", invalid_config).await;
    
    assert_eq!(response.status(), 400);
    let error: Value = response.json().await.unwrap();
    assert!(error.get("error").unwrap().as_str().unwrap().contains("port"));
}

#[tokio::test]
async fn test_config_rollback() {
    let server = TestServer::new().await;
    
    // 1. 获取初始配置
    let initial = server.get("/api/config").await.json::<Value>().await.unwrap();
    
    // 2. 应用新配置
    let new_config = json!({ ... });
    server.post("/api/config", new_config).await;
    
    // 3. 回滚
    let response = server.post("/api/rollback", json!({"version": "HEAD~1"})).await;
    
    // 4. 验证配置恢复
    let restored = server.get("/api/config").await.json::<Value>().await.unwrap();
    assert_eq!(initial, restored);
}
```

## E2E 测试规范

### 测试环境管理

```bash
# e2e/scripts/test-env-up.sh
#!/bin/bash
set -e

echo "启动 E2E 测试环境..."

# 1. 构建测试镜像
docker-compose -f docker-compose.test.yml build

# 2. 启动服务
docker-compose -f docker-compose.test.yml up -d

# 3. 等待健康检查
for i in {1..30}; do
    if curl -s http://localhost:8080/api/health | grep -q "ok"; then
        echo "✓ 服务就绪"
        exit 0
    fi
    sleep 1
done

echo "✗ 服务启动超时"
docker-compose -f docker-compose.test.yml logs
exit 1
```

### E2E 测试脚本

```bash
# e2e/tests/test_workflow_a_first_setup.sh
#!/bin/bash
set -e

API="http://localhost:8080/api"

echo "测试: 首次启动配置流程"

# 1. 检查初始状态
echo "  检查初始状态..."
STATE=$(curl -s $API/state | jq -r '.state')
assert_eq "$STATE" "normal" "初始状态应为 normal"

# 2. 提交初始配置
echo "  提交初始配置..."
curl -s -X POST $API/config \
    -H "Content-Type: application/json" \
    -d @fixtures/baseline-openclaw.json \
    | jq -r '.success' | grep -q "true"

# 3. 验证配置生效
echo "  验证配置..."
CONFIG=$(curl -s $API/config)
assert_contains "$CONFIG" "openai" "配置应包含 openai provider"

# 4. 验证 Git 快照
echo "  验证 Git 快照..."
SNAPSHOTS=$(curl -s $API/snapshots | jq length)
assert "[ $SNAPSHOTS -gt 0 ]" "应有配置快照"

echo "✓ 首次启动配置测试通过"
```

## 测试质量要求

### 覆盖率目标

| 模块 | 目标覆盖率 | 关键路径 |
|------|----------|----------|
| config.rs | 80%+ | save/apply/rollback |
| state.rs | 60%+ | enter/exit safe mode |
| validation.rs | 90%+ | 所有验证规则 |
| api/*.rs | 70%+ | 所有端点 |

### 测试数据

```rust
// 使用 fixtures
pub fn load_fixture(name: &str) -> Value {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name);
    let content = fs::read_to_string(path).unwrap();
    serde_json::from_str(&content).unwrap()
}

// fixtures/valid-config.json
{
    "gateway": { "port": 18790 },
    "models": [{ "id": "gpt-4", "provider": "openai" }],
    "channels": [{ "id": "telegram", "type": "telegram" }]
}

// fixtures/invalid-config-missing-port.json
{
    "gateway": {},
    "models": []
}
```

### 测试命名

```rust
// ✅ 描述行为，而非实现
test_config_saves_with_git_snapshot
test_config_rollback_restores_previous_state
test_api_returns_400_for_invalid_json

// ❌ 避免
test_config_1
test_api_post
```

## 测试检查清单

新增功能时必须：

- [ ] 单元测试覆盖核心逻辑（`--lib`）
- [ ] 集成测试覆盖 API 端点（`tests/api_*.rs`）
- [ ] 如有状态变更，添加模块联动测试
- [ ] 错误场景单独测试
- [ ] `make test-fast` 通过
- [ ] `make test-e2e` 通过（如相关）
