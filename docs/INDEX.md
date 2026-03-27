# Claw One 文档导航

**最后更新**: 2026-03-27

---

## 快速开始

- [README.md](./README.md) - 项目概述、快速开始

## 设计文档

| 文档 | 内容 | 状态 |
|------|------|------|
| [FINAL_DESIGN.md](./FINAL_DESIGN.md) | 架构、API、部署设计 | 后端已实现 |
| [VISUAL_CONFIG_DESIGN.md](./VISUAL_CONFIG_DESIGN.md) | 前端配置界面设计 | 前端待开发 |
| [CONFIG_MODULAR_RESEARCH.md](./CONFIG_MODULAR_RESEARCH.md) | 配置模块化调研 | 参考 |

## 工程文档

| 文档 | 内容 |
|------|------|
| [BUILD_STRATEGY.md](./BUILD_STRATEGY.md) | 构建策略（musl 静态链接）|
| [roadmap.md](./roadmap.md) | 实施路线图与待办清单 |

## 测试文档

| 文档 | 内容 |
|------|------|
| [TEST_FRAMEWORK.md](../TEST_FRAMEWORK.md) | 测试框架（项目根目录）|

---

## 文档维护原则

1. **DRY** - 信息不重复，一处修改处处生效
2. **单一来源** - 每个主题只在一份文档中详细描述
3. **链接优先** - 相关文档通过链接引用，不复制内容
4. **及时更新** - 实现状态变化时同步更新文档

---

## 已删除/合并的文档

| 原文档 | 处理方式 | 原因 |
|--------|---------|------|
| `TODO.md` | 合并到 `roadmap.md` | 内容重复 |
| `E2E_TEST_PLAN.md` | 删除 | 内容已整合到 `TEST_FRAMEWORK.md` |
| `openclaw-docker-run.md` | 删除 | 属于 OpenClaw 而非 claw-one |
