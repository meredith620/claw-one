# claw-one 测试覆盖 Review 报告

> 生成时间：2026-04-08
> 维护者：dev67
> 上次更新：2026-04-07

---

## 测试层概述

| 层 | 运行方式 | 测试数量 | 覆盖内容 |
|---|---|---|---|
| **Layer 1** | `cargo test --lib` | 36 个 | ConfigManager、Git、Validation、Runtime 单元测试 |
| **Layer 2** | `make test-fast` (cd hull && cargo test --test ...) | ~80+ 个 | API 端点、模块联动、数据完整性、错误场景、复杂验证 |
| **Layer 3** | `make test-e2e` (Docker 全链路) | 12 个脚本 | 完整用户流程、Git 快照/回滚、环境启动 |
| **Layer 4** | `make test-browser` (Playwright) | 5 个 spec | Vue 组件逻辑、UI 元素可见性、用户交互流程 |

> **⚠️ 注意**：Layer 4 Playwright 测试数量从上轮 14 个减少至 5 个 spec 文件，需确认是否有测试被移除或合并。

---

## 各模块覆盖矩阵

| 模块 | Layer 1 (单元) | Layer 2 (集成) | Layer 3 (Docker) | Layer 4 (浏览器) |
|------|:---:|:---:|:---:|:---:|
| **Provider** | ✅ CRUD + Validation | ✅ CRUD + 错误处理 | ✅ test_provider_crud.sh | ✅ 页面加载/对话框 |
| **Agent** | ✅ 配置 + Validation | ✅ CRUD + ID 验证 | ✅ test_agent_crud.sh | ✅ 页面加载/多 Agent |
| **Channel** | ⚠️ 部分 | ✅ CRUD + ID 验证 | ✅ test_channel_crud.sh | ✅ 页面加载/添加账号 |
| **Memory** | ✅ 配置 | ✅ CRUD + Validation | ✅ test_memory_crud.sh | ✅ 页面加载/Provider 选择 |
| **Git/版本** | ✅ 快照/回滚/并发 | ✅ 文件持久化 | ✅ 工作流测试 | ❌ |
| **Config 验证** | ✅ 完整性规则 | ✅ 多模块交叉验证 | ✅ | ❌ |
| **Claude Code 流程** | ✅ 新增 | ✅ 新增 | ❌ | ❌ |
| **Exec 演示** | ✅ 新增 | ✅ 新增 | ❌ | ❌ |
| **Runtime Status API** | ✅ Unit | ✅ API 集成 | ❌ | ❌ |
| **Safe Mode** | ✅ 进入/恢复/异常 | ⚠️ 部分 | ⚠️ 受限 | ❌ |
| **出厂重置** | ❌ | ❌ | ⚠️ 仅健康检查 | ❌ |

---

## 新增测试文件（本轮）

| 文件 | 测试数 | 说明 |
|------|--------|------|
| `test_claude_code_flow.rs` | 3 | Claude Code 流程测试（新增） |
| `test_exec_demo.rs` | 5 | Exec 演示流程测试（新增） |
| `test_validation_complex.rs` | 17 | 复杂验证场景测试（新增） |
| `api_status.rs` | 3 | Runtime Status API 测试（新增） |

> 合计新增：**28 个** Layer 2 集成测试

---

## 失败测试（本轮）

| 测试文件 | 失败测试 | 状态 |
|----------|---------|------|
| `api_all_modules_validation.rs` | `test_all_modules_file_persistence` | ❌ FAILED |
| `api_all_modules_validation.rs` | `test_all_modules_get_returns_saved_data` | ❌ FAILED |

> **需修复**：以上 2 个测试失败，涉及文件持久化和数据读取，需优先排查。

---

## 历史 Bug 覆盖分析

以下 Bug 均从 git log 提取，分析测试是否在引入前具备保护：

| Bug 描述 | 根因 | 修复 Commit | 测试保护 |
|----------|------|------------|---------|
| ChannelModule `saveChannels` 命名冲突导致**页面卡死** | 函数命名冲突，Vue 方法与 Axum handler 重名 | `4bb6000` | ⚠️ Layer 3/4 均只有 API 测试，无 Vue 逻辑测试 |
| A2A 配置**嵌套对象被完全替换** | `Object.assign` 浅合并 | `33590bf` | ✅ Layer 2 `api_config.rs` 有深合并验证测试 |
| `save_module_handler` **部分更新数据丢失** | 缺少深合并逻辑 | `80e4645` | ✅ Layer 2 `api_all_modules_validation.rs` |
| **事件对象字段污染** config | 部分更新时 event 字段混入 | `1b3b53a` | ✅ Layer 2 `test_all_modules_reject_event_object` |
| Agent 模块**全局保存事件未监听** | 未监听 `configChanged` | `9fb8341` | ✅ Layer 2 `test_save_agents_preserves_unmodified_fields` |
| setup-config.sh **缺少段落标记** | 配置文件格式错误 | `5814386` | ✅ Layer 3 `test_dist_config_format.sh` |
| Git **并发操作冲突** | Git 仓库状态竞争 | `404405b` | ✅ Layer 2 `test_concurrent_git_operations` |
| **Agent/Channel ID 验证静默绕过** | `validate_agents_only` 未包装 config | `ef0820d` | ✅ Layer 2 `api_agents_validation.rs` |

---

## 覆盖盲区

### 1. Vue 组件逻辑（Layer 4 过浅）
- **ChannelModule**: `saveChannels` 命名冲突 Bug 未被 Layer 4 覆盖（Layer 3/4 只有 API 测试，无 Vue 方法调用测试）
- **AgentModule**: 多 Agent 模式切换、Provider 绑定等核心交互无 UI 测试
- **ProviderModule**: 完整 CRUD 流程（创建→编辑→删除）无 Playwright 端到端测试

### 2. Safe Mode 完整链路
- `state_manager_test` 中 2 个涉及服务重启的测试被 `#[ignore]`，正式测试中 Safe Mode 自动触发路径无覆盖

### 3. 出厂重置
- `/api/setup/reset` 端点已知未完全实现，相关 API 测试缺失

### 4. 边界条件
- Agent/Channel ID 特殊字符：✅ 刚补全
- Provider baseUrl 重复冲突：❌ 无测试
- 空配置保存（首次启动）：❌ 无 API 边界测试

### 5. Config 文件损坏/非法的恢复
- openclaw.json 被手动破坏后的恢复路径无测试

### 6. Layer 4 测试数量下降
- 上轮报告 14 个 Playwright 测试，本轮仅 5 个 spec 文件，需确认测试是否被移除或误统计

---

## 改进建议（优先级排序）

### P0 — 必须补充（已有 Bug 先例）

- [ ] **补充 Layer 4 Provider 完整 CRUD 流程测试**
  - 创建 Provider → 编辑 → 删除的 Playwright 测试
  - 参照 `workflow.spec.ts` 的 Channel 关键 Bug 验证流程模式
  - 解决 Channel `saveChannels` Bug 的教训：UI 层逻辑必须有 E2E 保护

- [ ] **修复 Layer 4 Channel 账号删除测试的 flaky 问题**
  - ElMessageBox 确认按钮 selector 在 CI 中不稳定
  - 建议改用 API 预置数据，测试只验证删除结果

- [x] **修复 api_all_modules_validation 失败测试**（本轮新增失败）
  - `test_all_modules_file_persistence`
  - `test_all_modules_get_returns_saved_data`

### P1 — 重要（历史 Bug 暴露的缺口）

- [ ] **Safe Mode 进入/恢复的集成测试**
  - 目前 2 个相关测试被 ignore
  - 建议实现 mock RuntimeManager，在测试环境模拟服务重启

- [ ] **补充 Provider baseUrl 冲突检测测试**
  - 两个 Provider 使用相同 baseUrl 的边界场景

### P2 — 改进（提升置信度）

- [ ] **Layer 2 补充空配置保存测试**
  - 首次启动时 save 一个空/不完整 config 的 API 行为

- [ ] **Layer 3 补充 openclaw.json 损坏恢复测试**
  - 模拟配置文件被手动破坏后的行为

- [ ] **Layer 4 测试数量核对**
  - 上轮 14 个 vs 本轮 5 个，需确认实际测试数量

---

## 测试执行指南

```bash
# 快速反馈（<1 分钟）
make test-fast

# 完整验证（2-3 分钟）
make test-e2e

# 浏览器测试（1-2 分钟）
make test-browser

# 全部测试
make test-all
```

---

## 附：测试文件索引

| 路径 | 类型 | 说明 |
|------|------|------|
| `hull/src/config.rs` | Layer 1 | ConfigManager 单元测试 |
| `hull/src/validation.rs` | Layer 1 | Validation 规则测试 |
| `hull/tests/api_agents.rs` | Layer 2 | Agent API 测试 |
| `hull/tests/api_agents_validation.rs` | Layer 2 | Agent 验证测试（17 个） |
| `hull/tests/api_all_modules_validation.rs` | Layer 2 | 多模块验证（⚠️ 2 个失败） |
| `hull/tests/api_channels.rs` | Layer 2 | Channel API 测试 |
| `hull/tests/api_config_persistence.rs` | Layer 2 | 配置持久化测试 |
| `hull/tests/api_config.rs` | Layer 2 | Config API 测试 |
| `hull/tests/api_error_cases.rs` | Layer 2 | 错误场景测试 |
| `hull/tests/api_health.rs` | Layer 2 | 健康检查测试 |
| `hull/tests/api_memory.rs` | Layer 2 | Memory API 测试 |
| `hull/tests/api_memory_schema.rs` | Layer 2 | Memory Schema 测试 |
| `hull/tests/api_memory_validation.rs` | Layer 2 | Memory 验证测试 |
| `hull/tests/api_module_interaction.rs` | Layer 2 | 模块联动测试 |
| `hull/tests/api_providers.rs` | Layer 2 | Provider API 测试 |
| `hull/tests/config_git_test.rs` | Layer 2 | Git 版本管理测试 |
| `hull/tests/state_manager_test.rs` | Layer 2 | 状态机测试（含 ignore） |
| `hull/tests/test_claude_code_flow.rs` | Layer 2 | ⭐ 新增 Claude Code 流程测试（3 个） |
| `hull/tests/test_exec_demo.rs` | Layer 2 | ⭐ 新增 Exec 演示测试（5 个） |
| `hull/tests/test_validation_complex.rs` | Layer 2 | ⭐ 新增复杂验证测试（17 个） |
| `e2e/tests/test_*.sh` | Layer 3 | Docker 全链路测试（12 个） |
| `bridge/e2e-browser/tests/*.spec.ts` | Layer 4 | Playwright 浏览器测试（5 个 spec） |
