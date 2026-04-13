# 实施路线图与待办清单

**最后更新**: 2026-03-27

---

## 总体计划

MVP 阶段（6-8 周）完成 Claw One 核心功能。

```
Phase 1: MVP (6-8周)
├─ Config Guardian (Git 快照)
├─ OpenClaw Adapter (CRI)
├─ Web UI (Vue 3)
├─ systemd 集成
└─ Safe Mode
```

---

## 已确认决策（D1-D8）

| 决策 | 选择 |
|------|------|
| **D1** | 单 crate |
| **D2** | 抽象 CRI（OpenClaw + PicoClaw 预留）|
| **D3** | systemd + git 快照 + 30s 超时 + 自动/手动回滚 |
| **D4** | 无隔离（硬件隔离）|
| **D5** | Safe Mode 状态标记，三场景不同按钮 |
| **D6** | Vue 3 + Vite（纯 Web）|
| **D7** | JSON 格式 |
| **D8** | 纯软件形态，toC/toB 代码一致 |

---

## 执行状态

### ✅ 已完成

| 模块 | 状态 | 说明 |
|------|------|------|
| 测试框架 | ✅ | 100+ 测试覆盖四层架构 |
| Config Guardian | ✅ | Git 快照、回滚、提交 |
| Provider API | ✅ | CRUD 完整 |
| Agent API | ✅ | CRUD 完整 |
| Channel API | ✅ | CRUD 完整 |
| Memory API | ✅ | GET/POST，无 DELETE（设计如此）|
| Runtime Status API | ✅ | GET /api/status 实现 |
| State API | ✅ | GET /api/state（Claw One 内部状态）|

### 🔄 进行中/待完善

| 模块 | 状态 | 说明 |
|------|------|------|
| Safe Mode 自动触发 | ⚠️ | 需 OpenClaw 实际运行验证 |
| 出厂设置 | ⚠️ | `/api/setup/reset` 待完善 |
| Vue 3 前端 | ⏳ | 待开发 |
| systemd 集成 | ⏳ | 待完整验证 |
| systemd 集成测试 | 📋 | 暂不实现，方案见下方待测试项 |
| 健康检查测试 | 📋 | 暂不实现，方案见下方待测试项 |
| 回滚流程测试 | 📋 | 暂不实现，方案见下方待测试项 |

---

## 待测试项（暂不实现）

以下测试项已设计但暂不实现，记录于此以备后续参考。

### systemd 集成测试

**测试目标：** 验证 claw-one 服务通过 systemd 正确管理 OpenClaw 生命周期

| 测试编号 | 测试场景 | 预期结果 |
|----------|----------|----------|
| T1 | `systemctl --user start claw-one` | OpenClaw 服务启动，状态变为 active |
| T2 | `systemctl --user stop claw-one` | OpenClaw 服务停止，状态变为 inactive |
| T3 | `systemctl --user restart claw-one` | 服务重启，进程 PID 变化 |
| T4 | 开机自启 enable/disable | 重启后服务按配置自动运行/不运行 |
| T5 | 服务失败后自动重启 | systemctl restart 策略生效 |

**环境要求：** 需要真实的 systemd 用户环境，建议在主机环境执行（非 Docker）

---

### 健康检查测试

**测试目标：** 验证 30s 超时健康检查机制

| 测试编号 | 测试场景 | 预期结果 |
|----------|----------|----------|
| H1 | OpenClaw 正常运行 | 健康检查返回 `true`，响应时间 < 30s |
| H2 | OpenClaw 进程崩溃 | 健康检查超时（30s），返回 `false` |
| H3 | OpenClaw 配置错误 | 健康检查失败，返回 `false`，进入 Safe Mode |
| H4 | 端口被占用 | 健康检查返回 `false`，错误分类为 System |
| H5 | 并发健康检查 | 多个请求同时检查，不产生竞争条件 |

**测试策略：**
- Layer 1: 测试 `RuntimeManager::health_check()` 逻辑（使用 mock）
- Layer 2: 测试 `/api/restart` 后的健康检查流程
- Layer 3: 模拟 OpenClaw 服务无响应场景，验证 30s 超时

---

### 完整回滚流程测试

**测试目标：** 验证 Safe Mode 自动回滚机制

| 测试编号 | 测试场景 | 预期结果 |
|----------|----------|----------|
| R1 | 配置错误触发自动回滚 | 切换到上一个正常版本，状态标记 `auto_rolled_back: true` |
| R2 | 配置错误无历史版本 | 进入 Safe Mode，提示无可用回滚版本 |
| R3 | 系统错误不自动回滚 | 进入 Safe Mode，保留当前错误配置 |
| R4 | 回滚后服务恢复 | 健康检查通过，状态恢复 Normal |

---

## Phase 1: MVP 详细计划

### Week 1-2: 项目基础
- [x] Rust 项目骨架（Axum + 静态文件服务）
- [x] 测试框架搭建
- [ ] Vue 3 项目搭建（Vite + Element Plus）

### Week 2-3: Config Guardian
- [x] Git 仓库自动初始化
- [x] 配置保存/读取
- [x] 快照列表（git log 解析）
- [x] 回滚逻辑（git checkout）

### Week 3-4: Runtime Adapter
- [x] `ClawRuntime` trait 定义
- [x] `OpenClawAdapter` 基础实现
- [ ] systemd 完整集成验证
- [ ] 健康检查（30s 超时）

### Week 4-5: Web UI
- [ ] 配置向导页面
- [ ] 状态监控页面
- [ ] Safe Mode 页面

### Week 5-6: Safe Mode
- [x] 状态机（Normal/SafeMode）
- [x] 错误分类逻辑
- [ ] 自动回滚完整验证
- [ ] 三场景 UI 按钮

### Week 6-8: 测试与发布
- [x] 单元测试 + 集成测试
- [x] E2E 测试框架
- [ ] 跨平台测试
- [ ] 文档完善
- [ ] MVP 发布

---

## 后续规划（V2+）

| 版本 | 目标 | 关键功能 |
|------|------|---------|
| **V2** | 多 Runtime | PicoClaw 支持 |
| **V3** | toB 功能 | SSO、审计日志 |
| **V4** | 企业级 | 集群部署 |

---

## 文档导航

| 文档 | 说明 |
|------|------|
| [FINAL_DESIGN.md](./FINAL_DESIGN.md) | 架构、API、部署设计 |
| [VISUAL_CONFIG_DESIGN.md](./VISUAL_CONFIG_DESIGN.md) | 前端配置界面设计 |
| [CONFIG_MODULAR_RESEARCH.md](./CONFIG_MODULAR_RESEARCH.md) | 配置模块化调研 |
| [BUILD_STRATEGY.md](./BUILD_STRATEGY.md) | 构建策略 |
| [TEST_FRAMEWORK.md](../TEST_FRAMEWORK.md) | 测试框架（项目根目录）|

---

*本文档合并了原 roadmap.md 和 TODO.md，遵循 DRY 原则*
