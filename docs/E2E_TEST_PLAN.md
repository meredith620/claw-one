# Claw One 端到端集成测试计划

## 测试目标

验证前后端联调、API 完整性、以及完整用户流程的正确性。

## 测试环境

- **后端**: Rust backend (端口 8080)
- **前端**: Vue 3 静态文件 (通过后端 ServeDir 服务)
- **浏览器**: Chrome/Firefox 最新版
- **测试数据**: 使用测试用的 openclaw.json

## 测试范围

### 1. API 功能测试 (curl/Postman)

| 端点 | 方法 | 测试内容 | 预期结果 |
|------|------|----------|----------|
| `/api/health` | GET | 健康检查 | `{"status":"ok","version":"0.1.0"}` |
| `/api/state` | GET | 获取当前状态 | 返回状态、版本、可回滚标志 |
| `/api/config` | GET | 读取配置 | 返回完整 openclaw.json |
| `/api/config` | POST | 应用新配置 | 保存配置并重启服务 |
| `/api/snapshots` | GET | 获取快照列表 | 返回 Git 提交历史 |
| `/api/rollback` | POST | 回滚到指定版本 | 恢复配置并重启 |
| `/api/logs` | GET | 获取日志 | 返回服务日志内容 |
| `/api/restart` | POST | 重启服务 | 调用 systemctl restart |
| `/api/setup/check` | GET | 检查首次启动 | `{"is_first_setup": true/false}` |
| `/api/setup/complete` | POST | 完成初始化 | 标记已初始化 |
| `/api/setup/reset` | POST | 恢复出厂设置 | 重置为默认配置 |

### 2. 前端页面测试

| 页面 | 测试项 | 验证点 |
|------|--------|--------|
| **SetupWizard** | 4步向导流程 | 欢迎→模型配置→渠道配置→完成 |
| | 添加/删除模型 | 动态表单增删 |
| | 表单验证 | 必填项校验 |
| | 提交配置 | 调用 `/api/config` POST |
| **StatusView** | 状态显示 | 当前状态、版本、运行时间 |
| | 日志查看 | 实时/历史日志 |
| | 快照列表 | Git 历史、回滚按钮 |
| | 重启按钮 | 调用 `/api/restart` |
| **ConfigView** | JSON 编辑器 | 读取/编辑配置 |
| | 保存配置 | 提交后进入 Applying 状态 |
| **SafeModeView** | 错误显示 | Config/System 错误分类 |
| | 回滚选项 | 自动回滚、手动回滚、强制应用 |

### 3. 完整用户流程测试

#### 流程 A: 首次启动配置
```
1. 访问 http://localhost:8080
2. 检查是否跳转到 SetupWizard
3. 完成4步向导配置
4. 验证配置已保存到 openclaw.json
5. 验证 Git 提交了初始配置
6. 验证页面跳转到 StatusView
```

#### 流程 B: 正常配置更新
```
1. 从 StatusView 进入 ConfigView
2. 修改配置（如添加模型）
3. 保存配置
4. 验证状态变为 ApplyingConfig
5. 验证服务重启
6. 验证状态恢复为 Normal
7. 验证 Git 有新提交
```

#### 流程 C: 配置错误回滚
```
1. 故意提交错误配置（无效JSON）
2. 验证进入 SafeMode
3. 验证错误类型为 ConfigError
4. 点击"回滚到上一版本"
5. 验证配置恢复
6. 验证状态恢复为 Normal
```

#### 流程 D: 出厂设置
```
1. 在 SetupWizard 或 Settings 中触发"恢复出厂设置"
2. 验证配置重置为默认
3. 验证 Git 有新提交（出厂设置）
4. 验证页面行为（根据设计跳转或刷新）
```

## 测试执行清单

- [ ] 后端编译通过
- [ ] 前端构建通过
- [ ] 后端服务启动正常
- [ ] 前端页面访问正常
- [ ] 所有 API 端点响应正确
- [ ] 流程 A: 首次启动配置
- [ ] 流程 B: 正常配置更新
- [ ] 流程 C: 配置错误回滚
- [ ] 流程 D: 出厂设置

## 测试数据准备

创建 `test-config/` 目录，包含：

### Claw One 配置文件
- `claw-one.toml` - Claw One 自身运行配置（位于 ~/claw-one/config/）
  - 配置 OpenClaw 连接信息（service_name, openclaw_home, health_port）
  - 配置 Claw One 服务参数（host, port, log_level）

### OpenClaw 配置文件（被管理）
- `minimal.json` - OpenClaw 最小可用配置（位于 {openclaw_home}/）
- `with-models.json` - 包含模型配置的 OpenClaw 配置
- `with-channels.json` - 包含渠道配置的 OpenClaw 配置
- `invalid.json` - 故意错误的配置（用于测试 SafeMode）

### 配置关系说明
```
~/claw-one/config/claw-one.toml  (Claw One 自身配置)
    ├── openclaw_home = "~/.openclaw"
    ├── service_name = "openclaw"
    └── health_port = 18790
        
~/.openclaw/openclaw.json  (OpenClaw 被管理的配置)
    ├── models: [...]
    └── channels: [...]
```

## 自动化测试方案 (可选)

### 后端 API 测试 (Rust)
使用 `tokio-test` 编写集成测试：
```rust
#[tokio::test]
async fn test_health_endpoint() {
    // 启动测试服务器
    // 调用 /api/health
    // 验证响应
}
```

### 前端 E2E 测试 (Playwright/Cypress)
```typescript
// 示例: 测试首次启动向导
test('first-time setup wizard', async ({ page }) => {
  await page.goto('http://localhost:8080');
  await expect(page).toHaveURL(/.*setup/);
  // 完成向导步骤...
});
```

## 已知限制

1. **cargo 不可用**: 服务器环境缺少 Rust 工具链，后端测试需在本地执行
2. **vue-tsc 兼容性问题**: Node 22 下类型检查有警告，但不影响功能
3. **进程管理**: 部分功能需要 OpenClaw 运行才能完整测试

## 下一步行动

1. [ ] 准备测试配置文件
2. [ ] 在开发环境部署并执行测试
3. [ ] 记录测试结果和发现的问题
