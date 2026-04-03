# claw-one 测试入口地图

**文档位置**: `docs/TESTING.md`  
**更新日期**: 2026-04-03

---

## 测试分层架构

```
┌─────────────────────────────────────────────────────────────┐
│ Layer 4: Browser E2E (Playwright)                           │
│ ├── 21 个测试用例                                           │
│ ├── 覆盖 Vue 组件交互                                       │
│ └── 执行时间: ~5 分钟（容器化运行）                          │
├─────────────────────────────────────────────────────────────┤
│ Layer 3: Docker E2E (Shell Scripts)                         │
│ ├── 10 个测试脚本                                           │
│ ├── 覆盖 API 和用户流程                                     │
│ └── 执行时间: ~3 分钟                                       │
├─────────────────────────────────────────────────────────────┤
│ Layer 2: 集成测试 (Rust)                                    │
│ ├── 37 个测试                                               │
│ ├── 覆盖 API 端点和模块联动                                 │
│ └── 执行时间: ~20 秒                                        │
├─────────────────────────────────────────────────────────────┤
│ Layer 1: 单元测试 (Rust)                                    │
│ ├── 43 个测试                                               │
│ ├── 覆盖 Config/State/Git/Validation                        │
│ └── 执行时间: ~10 秒                                        │
└─────────────────────────────────────────────────────────────┘

总计: 111 个测试
```

---

## 快速命令

### 一键运行所有测试
```bash
make test-full    # Layer 1+2+3+4 (~8-10 分钟)
```

### 分层测试
```bash
# Layer 1 + 2: 快速测试 (~30 秒)
make test-fast

# Layer 3: Docker E2E (~3 分钟)
make test-e2e

# Layer 4: Browser E2E (~5 分钟)
make test-browser
```

### 查看测试环境状态
```bash
make test-env-status
```

### 清理测试环境
```bash
make test-env-down
```

---

## 新环境搭建步骤

### 1. 克隆项目
```bash
git clone <repository-url>
cd claw-one
```

### 2. 安装依赖（可选，本地开发需要）
```bash
# Rust 工具链
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Node.js (用于前端构建和本地 Browser 测试)
npm install -g n
n 20

# 项目依赖
make deps
```

### 3. 安装 Docker 和 Docker Compose
```bash
# Ubuntu/Debian
curl -fsSL https://get.docker.com | sh
sudo usermod -aG docker $USER
# 重新登录使权限生效
```

### 4. 验证环境
```bash
# 检查 Docker
docker --version
docker compose --version

# 运行快速测试验证 Layer 1+2
make test-fast
```

---

## 详细测试说明

### Layer 1: 单元测试

**位置**: `hull/src/*.rs` (内联测试)

**运行方式**:
```bash
cd hull && cargo test --lib
```

**覆盖模块**:
- `config.rs`: ConfigManager CRUD, Git 基础操作 (19 tests)
- `validation.rs`: 配置验证逻辑 (12 tests)
- `state.rs`: StateManager 状态机 (内联)

### Layer 2: 集成测试

**位置**: `hull/tests/`

**运行方式**:
```bash
cd hull && cargo test --test api_health --test api_config ...
```

**测试文件**:
| 文件 | 测试数 | 说明 |
|------|--------|------|
| `api_health.rs` | 1 | Health endpoint |
| `api_config.rs` | 8 | Config API + 数据完整性 |
| `api_providers.rs` | 6 | Provider CRUD |
| `api_agents.rs` | 2 | Agent 读写 |
| `api_memory.rs` | 2 | Memory 读写 |
| `api_channels.rs` | 2 | Channel 读写 |
| `api_error_cases.rs` | 10 | 错误场景测试 |
| `api_module_interaction.rs` | 6 | 模块联动测试 |
| `state_manager_test.rs` | 12 | StateManager 状态机 |

### Layer 3: Docker E2E 测试

**位置**: `e2e/tests/`

**运行方式**:
```bash
make test-e2e
# 或分步执行:
cd e2e && ./scripts/test-env-up.sh
cd e2e && ./scripts/run-e2e-tests.sh
```

**测试脚本**:
| 脚本 | 说明 |
|------|------|
| `test_health.sh` | Health check |
| `test_frontend.sh` | 前端静态资源 |
| `test_provider_crud.sh` | Provider CRUD |
| `test_agent_crud.sh` | Agent CRUD |
| `test_channel_crud.sh` | Channel CRUD |
| `test_memory_crud.sh` | Memory CRUD |
| `test_workflow_a_first_setup.sh` | 首次启动配置 |
| `test_workflow_b_normal_update.sh` | 正常配置更新 |
| `test_workflow_c_error_rollback.sh` | 错误回滚 |
| `test_workflow_d_factory_reset.sh` | 出厂设置 |

### Layer 4: Browser E2E 测试

**位置**: `bridge/e2e-browser/`

**运行方式**:
```bash
# 方式1: 使用 Makefile（推荐，容器化运行）
make test-browser

# 方式2: 手动 Docker 运行
cd e2e
docker compose -f docker-compose.test.yml up -d openclaw claw-one
docker compose -f docker-compose.test.yml run --rm playwright

# 方式3: 本地运行（需安装浏览器）
cd bridge
npm install
npx playwright install chromium
npm run test:e2e
```

**测试文件**:
| 文件 | 测试数 | 覆盖模块 | 优先级 |
|------|--------|----------|--------|
| `provider.spec.ts` | 4 | Provider CRUD | 中 |
| `agent.spec.ts` | 3 | Agent CRUD | 中 |
| `channel.spec.ts` | 6 | Channel CRUD | **高** |
| `memory.spec.ts` | 3 | Memory 配置 | 低 |
| `workflow.spec.ts` | 5 | 用户工作流 | **高** |

**容器化优势**:
- 预装 Chromium 浏览器，无需本地安装
- 隔离环境，避免依赖冲突
- 结果自动挂载到 `e2e/playwright-report/` 和 `e2e/test-results/`

---

## 测试矩阵覆盖

| 功能模块 | Layer 1 | Layer 2 | Layer 3 | Layer 4 |
|----------|---------|---------|---------|---------|
| Config CRUD | ✅ | ✅ | ✅ | - |
| Provider CRUD | ✅ | ✅ | ✅ | ✅ |
| Agent CRUD | ✅ | ✅ | ✅ | ✅ |
| Channel CRUD | ✅ | ✅ | ✅ | ✅ |
| Memory CRUD | ✅ | ✅ | ✅ | ✅ |
| 数据完整性 | - | ✅ | - | - |
| 错误处理 | - | ✅ | - | - |
| 状态机 | ✅ | - | - | - |
| Git 快照/回滚 | ✅ | - | - | - |
| 用户流程 | - | - | ✅ | ✅ |
| Vue 组件逻辑 | - | - | - | ✅ |
| 前端表单验证 | - | - | - | ✅ |

---

## CI/CD 集成建议

### GitHub Actions 示例
```yaml
name: Test
on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      # Layer 1 + 2
      - name: Run unit and integration tests
        run: make test-fast
      
      # Layer 3
      - name: Run Docker E2E tests
        run: make test-e2e
      
      # Layer 4
      - name: Run Browser E2E tests
        run: make test-browser
```

---

## 故障排除

### Docker 权限问题
```bash
# 将用户加入 docker 组
sudo usermod -aG docker $USER
# 重新登录或执行:
newgrp docker
```

### Playwright 测试失败
```bash
# 查看详细错误
cd e2e/playwright-report
# 打开 index.html 查看报告

# 查看测试追踪
cd e2e/test-results
npx playwright show-trace <trace.zip>
```

### 端口冲突
```bash
# E2E 测试使用非标准端口:
# - claw-one: 28080
# - openclaw: 28789
# 确保这些端口未被占用
```

---

## 相关文档

- [TEST_FRAMEWORK.md](../TEST_FRAMEWORK.md) - 测试框架详细说明
- [bridge/e2e-browser/README.md](../bridge/e2e-browser/README.md) - Browser E2E 测试文档
