# Claw One 项目文档

**版本**: MVP 1.0  
**最后更新**: 2026-03-20

---

## 项目概述

Claw One 是一个便捷管理 OpenClaw 常用功能的 Web 管理工具，解决 OpenClaw 配置复杂、修改易错、门槛高的问题。

## 核心目标

1. **可视化配置**：Web 界面管理配置，无需手写 JSON
2. **配置不锁死**：即使配置错误也能自动恢复，确保服务可用
3. **版本管理**：配置变更自动 Git 快照，支持回滚

## 产品形态

Claw One 是一个独立运行的 Web 服务，提供配置管理、状态监控、版本回滚等功能。

**说明**：
- 纯软件形态，作为 OpenClaw 的管理工具
- toC/toB 共用一套代码，通过配置区分
- 支持 Linux/macOS/Windows

## 文档导航

- [最终设计文档](./FINAL_DESIGN.md) - 完整架构、API、部署设计
- [实施路线图](./roadmap.md) - MVP 计划、里程碑
- [待办清单](./TODO.md) - 执行待办

## 关键设计原则

### 1. 配置不中断服务
```
用户修改配置
    ↓
[Config Guardian] Git 快照管理
    ↓
保存 → 重启 OpenClaw → 健康检查
    ↓（成功）
git commit 记录版本
    ↓（失败，配置错误）
自动回滚 → 进入 Safe Mode（显示已回滚提示）
    ↓（失败，系统错误）
进入 Safe Mode（不回滚，用户决定）
```

### 2. 安全隔离
- **无软件沙盒**：简化实现
- 运行权限受系统限制

### 3. 运行时支持
- **当前**：OpenClaw（Node.js）
- **预留**：PicoClaw（Go）通过 CRI 接口
- **切换**：V2 支持多 Runtime

## 技术栈

| 层级 | 技术 |
|------|------|
| 前端 | Vue 3 + TypeScript + Vite |
| 后端 | Rust + Axum + Tokio |
| 进程管理 | systemd |
| 快照存储 | Git |
| 配置格式 | JSON |

## 快速开始

### 开发环境
```bash
git clone <repo>
cd claw-one
# 详见 FINAL_DESIGN.md
```

### 运行要求
- OpenClaw 已安装并可运行
- Rust 1.70+ / Node.js 18+ 开发环境

---

*文档已按最终设计更新*
