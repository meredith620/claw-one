# Claw Box 项目文档

## 项目概述

Claw Box 是一个让 OpenClaw 开箱即用的产品解决方案，面向非程序员背景的个人用户和小微企业，解决 OpenClaw 部署复杂、配置易错、门槛高的问题。

## 核心目标

1. **零门槛部署**：用户无需懂代码、无需配置环境
2. **配置不锁死**：即使配置错误也能自动恢复，无需命令行
3. **toC/toB 演进**：toC 起步，toB 在相同内核上扩展

## 产品形态

| 层级 | 产品 | 定价 | 目标用户 |
|------|------|------|---------|
| L1 | Claw Desktop（软件） | **免费** | 个人 Geek、分析师 |
| L2 | Claw Box（N100 小主机） | **¥999-1499** 一次性 | 普通用户、家庭 |
| L3 | Claw Enterprise | **配套销售** | 小微企业 |

## 文档导航

- [架构设计](./architecture.md) - 三层架构、核心组件
- [商业模式](./business-model.md) - 定价、成本、盈利分析
- [产品形态](./product-forms.md) - Desktop / Box / Enterprise 详细设计
- [技术规范](./tech-spec.md) - 接口定义、数据模型、安全策略
- [实施路线图](./roadmap.md) - MVP 计划、里程碑、风险

## 关键设计原则

### 1. 配置不中断服务
```
用户修改配置
    ↓
[Config Guardian] 事务性处理
    ↓
backup → validate → apply → healthcheck → commit
    ↓（失败）
自动 rollback → 进入 Safe Mode（端口 18790）
```

### 2. 四层安全隔离（可选）
- **L3 硬件隔离**：独立 N100 设备（Box 版）
- **L2 系统容器**：Systemd-nspawn / LXC
- **L1 用户沙盒**：专用用户 + 权限限制（默认）
- **L0 裸机**：无隔离（高级用户）

### 3. 多运行时支持
- OpenClaw（Node.js，功能最全）
- PicoClaw（Go，轻量）
- Nanobot（Rust，安全）
- 统一 CRI 接口，可一键切换

## 快速开始

### 开发环境
```bash
git clone <repo>
cd claw-one
# 详见各子项目 README
```

### 硬件采购
- N100 小主机开发样机（详见 business-model.md）
- 建议先采购 2-3 台不同厂商的样机对比

---

*最后更新：2026-03-02*
