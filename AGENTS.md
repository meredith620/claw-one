---
# HARNESS METADATA
# type: entry-point
# part-of: harness-architecture
# managed-by: harness-system
# do-not-modify-manually: 如需修改请遵循 harness-evolution.spec.md
# version: 1.0
# created: 2026-04-01
---

# AGENTS.md - Claw One Harness

> **Agent 通用入口文档** —— 适用于 OpenClaw、Claude Code、OpenCode 等任何 Agent 系统
> 
> ⚠️ **HARNESS 文件**: 本文档属于 Harness 架构，修改需谨慎

## 项目概览

**Claw One** 是 OpenClaw 的 Web 配置管理界面，解决配置复杂、易错、门槛高的问题。

| 属性 | 值 |
|------|-----|
| **技术栈** | Rust (Axum/Tokio) + Vue 3 + TypeScript |
| **架构** | 单 crate，前后端分离部署 |
| **核心功能** | 配置版本控制、Safe Mode 自动回滚、Web 管理界面 |
| **数据持久化** | Git 快照 + TOML/JSON 配置 |

```
┌─────────────────────────────────────────┐
│  用户浏览器 → Vue 3 前端 (bridge/)      │
└─────────────────┬───────────────────────┘
                  │ HTTP
┌─────────────────▼───────────────────────┐
│  Axum API (hull/src/api/)               │
│  Config Guardian (hull/src/config.rs)   │
│  State Manager (hull/src/state.rs)      │
└─────────────────────────────────────────┘
```

## 快速开始

### 开发环境

```bash
# 1. 安装依赖
make deps

# 2. 创建开发配置
cp config/claw-one.toml.example hull/config.dev.toml
# 编辑 config.dev.toml 设置 OpenClaw 连接信息

# 3. 启动开发模式
make dev              # 前台运行，Ctrl+C 停止
```

访问 http://localhost:8080

### Harness 安装与设置 ⚙️

**重要**：首次克隆项目后，需要运行以下命令来启用 Harness 的自动检查功能：

```bash
# 1. 链接 Git Pre-Commit Hook（自动运行架构验证和提交前检查）
make harness-install

# 或手动链接：
ln -sf ../../.harness/scripts/pre-commit.sh .git/hooks/pre-commit
chmod +x .git/hooks/pre-commit

# 2. 验证 Harness 安装
./.harness/scripts/list.sh

# 3. 运行架构验证
./.harness/scripts/validate-arch.sh
```

**Harness 工具说明**：

| 命令 | 说明 |
|------|------|
| `./.harness/scripts/list.sh` | 列出所有 Harness 文档和工具 |
| `./.harness/scripts/validate-arch.sh` | 运行架构合规性检查 |
| `./.harness/scripts/pre-commit.sh` | 提交前自动检查（被 git hook 调用） |
| `./.harness/scripts/prepare-commit.sh` | 原子提交辅助工具 |
| `./.harness/scripts/evaluate-guards.sh` | 运行熵防护规则检查 |

**未安装时的手动检查**：

如果 git hook 未链接，提交前请手动运行：
```bash
./.harness/scripts/pre-commit.sh
./.harness/scripts/validate-arch.sh
```

### 常用命令

| 命令 | 说明 |
|------|------|
| `make deps` | 安装前后端依赖 |
| `make dev` | 开发模式（前台）|
| `make build` | 构建前后端（生产）|
| `make test-fast` | 快速测试 (Layer 1+2) |
| `make test-e2e` | E2E 测试 (Layer 3) |
| `make dist` | 构建分发包 (Docker musl) |

## 项目结构

```
claw-one/
├── AGENTS.md                 ⭐ Agent 通用入口（本文档）
├── harness.yaml              ⭐ Harness Manifest
├── .harness/                 📁 Harness 架构目录（与项目代码分离）
│   ├── specs/                📖 规范文档
│   │   ├── api.spec.md
│   │   ├── config.constraint.md
│   │   ├── architecture.constraint.md
│   │   ├── testing.spec.md
│   │   ├── release.spec.md
│   │   └── harness-evolution.spec.md
│   ├── scripts/              🔧 Harness 脚本
│   │   ├── list.sh
│   │   ├── validate-arch.sh
│   │   ├── pre-commit.sh
│   │   └── pre-config-change.sh
│   └── guards/               🛡️ 熵防护规则
│       ├── protect-harness-files.rule
│       ├── no-direct-config-modification.rule
│       ├── require-test-coverage.rule
│       └── enforce-layer-separation.rule
├── hull/                     # Rust 后端
├── bridge/                   # Vue 3 前端
└── ...
```

## 代码规范

### Rust

- **错误处理**: 使用 `thiserror` 定义错误类型，`anyhow` 进行上下文传播
- **异步**: Tokio 运行时，axum 处理 HTTP
- **日志**: tracing 结构化日志
- **测试**: 三层测试架构（见 [`.harness/specs/testing.spec.md`](.harness/specs/testing.spec.md)，详细文档见 [TEST_FRAMEWORK.md](TEST_FRAMEWORK.md)）

### 关键模式

```rust
// 1. ConfigManager - 配置 CRUD + Git 快照
pub struct ConfigManager { git_repo: Repository, ... }

// 2. StateManager - Safe Mode 状态机
pub struct StateManager { state: AppState, ... }

// 3. Runtime Adapter - OpenClaw 进程管理
pub struct OpenClawAdapter { ... }
impl ClawRuntime for OpenClawAdapter { ... }
```

## 测试策略

```
Layer 1: 单元测试 (43个)    → cargo test --lib
Layer 2: 集成测试 (37个)    → cargo test --test api_*
Layer 3: E2E 测试 (10个)    → make test-e2e
```

### 测试分层

| 层级 | 范围 | 依赖 | 位置 |
|------|------|------|------|
| Layer 1 | 单个模块内部逻辑 | 无外部依赖 | `hull/src/*.rs` 中的 `#[cfg(test)]` |
| Layer 2 | API 端点 + 模块交互 | Mock Runtime | `hull/tests/*.rs` |
| Layer 3 | 完整用户流程 | Docker + OpenClaw | `e2e/tests/*.sh` |

### E2E 测试辅助

E2E 测试使用 `e2e/tests/assert.sh` 中的标准断言函数：

```bash
# 引入辅助函数
source "$(dirname "$0")/assert.sh"

# 使用断言
assert_eq "$STATE" "normal" "初始状态应为 normal"
assert_contains "$CONFIG" "openai" "配置应包含 provider"
assert_json_eq "$RESP" ".status" "ok" "状态应为 ok"
```

### 运行测试前必须检查

1. `cargo check` 无编译错误
2. `cargo clippy` 无警告
3. `make test-fast` 通过

> 详细测试规范、覆盖率目标、fixtures 使用见 [`.harness/specs/testing.spec.md`](.harness/specs/testing.spec.md)

## 架构约束

1. **单实例控制**: 使用文件锁确保只有一个 claw-one 进程运行
2. **事务性配置**: 任何配置变更必须先写 Git 快照
3. **Safe Mode**: 配置错误时进入安全模式，禁止自动恢复，必须用户确认
4. **无 root 权限**: 用户级 systemd 服务，不依赖 root

## 依赖 Harness 文档

Agent 应根据任务类型加载对应规范：

| 任务类型 | 加载文档 |
|----------|----------|
| API 开发 | `.harness/specs/api.spec.md` |
| 配置变更 | `.harness/specs/config.constraint.md` |
| 架构修改 | `.harness/specs/architecture.constraint.md` |
| 添加测试 | `.harness/specs/testing.spec.md` |
| 发布构建 | `.harness/specs/release.spec.md` |
| 修改 Harness | `.harness/specs/harness-evolution.spec.md` |

## 外部资源

- **OpenClaw Config Schema**: `hull/schemas/memory-config.json`
- **API 文档**: 运行时访问 `/api/health` 获取 OpenAPI (如有)
- **日志位置**: `~/.openclaw/logs/`

## 约束与红线

⛔ **永远不要:**
- 直接修改 `~/.openclaw/openclaw.json` 而不经过 claw-one API
- 在生产环境运行 `make dist-native` (动态链接，不兼容)
- 跳过测试直接提交
- 删除 `hull/target/debug/` 中的测试产物（用于调试）

---

*Harness Version: 1.0 | Last Updated: 2026-04-01*
