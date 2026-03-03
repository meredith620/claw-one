# Claw One 待解决问题清单

**最后更新**: 2026-03-03  
**状态**: D1-D8 决策完成，进入开发阶段

---

## ✅ 已确认决策（D1-D8）

| 决策 | 选择 |
|------|------|
| **D1** | 单 crate |
| **D2** | 抽象 CRI（OpenClaw + PicoClaw 预留）|
| **D3** | systemd + git 快照 + 30s 超时 + 自动/手动回滚 |
| **D4** | 无隔离（硬件隔离）|
| **D5** | Safe Mode 状态标记，三场景不同按钮 |
| **D6** | Vue 3 + Vite（纯 Web，浏览器访问 Box IP）|
| **D7** | JSON 格式 |
| **D8** | 仅 Box 形态，toC/toB 代码一致，仅 Linux |

**核心设计文档**: [FINAL_DESIGN.md](./FINAL_DESIGN.md)

---

## 🔴 P0 - 阻塞级（已完成）

- [x] P0-1: 技术选型确定（Rust + Axum + Vue 3）
- [x] P0-2: 事务性配置策略（Git 快照 + systemd）
- [x] P0-3: 产品定位（仅 Box，toC/toB）

---

## 🟡 P1 - 重要级（开发中）

### P1-1. Config Guardian 实现 ⏳
- [ ] Git 仓库自动初始化
- [ ] 配置保存/读取
- [ ] 快照列表（git log 解析）
- [ ] 回滚逻辑（git checkout）

### P1-2. CRI 接口实现 ⏳
- [ ] `ClawRuntime` trait 定义
- [ ] `OpenClawAdapter` 实现
- [ ] PicoClaw 预留接口

### P1-3. systemd 集成 ⏳
- [ ] `systemctl start/stop/restart` 封装
- [ ] 服务状态查询
- [ ] 健康检查（30s 超时）

### P1-4. Safe Mode 实现 ⏳
- [ ] 状态机（Normal/SafeMode）
- [ ] 错误分类（配置错误 vs 系统错误）
- [ ] 自动回滚逻辑
- [ ] 三场景 UI 按钮

### P1-5. N100 性能验证 📦 **待硬件**
- [ ] 采购 N100 样机（2-3 家厂商）
- [ ] 测试 OpenClaw 资源占用
- [ ] 验证 30s 超时是否足够

### P1-6. 前端开发 ⏳
- [ ] Vue 3 项目搭建
- [ ] 配置向导页面
- [ ] 状态监控页面
- [ ] Safe Mode 页面

---

## 🟢 P2 - 中级（V2 规划）

- [ ] P2-1: 多 Runtime 切换（PicoClaw 支持）
- [ ] P2-2: Git 快照自动清理策略
- [ ] P2-3: toB 企业功能（SSO、审计）

---

## 📋 执行计划

### 本周（Week 0）✅
- [x] 完成 D1-D8 决策
- [ ] 采购 N100 开发样机
- [ ] 初始化代码仓库

### Week 1
- [ ] Rust 项目骨架（Axum + 静态文件服务）
- [ ] Vue 3 项目搭建（Vite + Element Plus）
- [ ] 开发环境联调

### Week 2
- [ ] Config Guardian 基础（Git 操作）
- [ ] 配置保存/读取 API
- [ ] 配置向导前端页面

### Week 3
- [ ] CRI trait + OpenClaw Adapter
- [ ] systemd 集成
- [ ] 状态监控前端页面

### Week 4
- [ ] 健康检查 + 自动回滚
- [ ] Safe Mode 后端逻辑
- [ ] Safe Mode 前端页面

### Week 5
- [ ] 首次启动向导
- [ ] 出厂设置功能
- [ ] 日志查看功能

### Week 6-7
- [ ] N100 实机部署测试
- [ ] 集成测试
- [ ] Bug 修复

### Week 8
- [ ] 文档完善
- [ ] MVP 发布

---

## 📁 文档清单

| 文档 | 说明 |
|------|------|
| [FINAL_DESIGN.md](./FINAL_DESIGN.md) | **核心设计文档**（架构、API、部署）|
| [README.md](./README.md) | 项目概述、快速开始 |
| [roadmap.md](./roadmap.md) | 实施路线图、里程碑 |
| [TODO.md](./TODO.md) | 本文件，执行待办 |

---

*文档已按最终设计更新*
