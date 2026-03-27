# Claw One

OpenClaw 的 Web 管理工具 —— 可视化配置、版本管理、自动回滚。

---

## 核心功能

| 功能 | 说明 |
|------|------|
| **可视化配置** | Web 界面管理 openclaw.json，无需手写 JSON |
| **配置安全** | 错误配置自动回滚，服务不中断 |
| **版本管理** | Git 快照，支持回滚到任意版本 |

## 技术栈

- **前端**: Vue 3 + TypeScript + Vite
- **后端**: Rust + Axum + Tokio
- **进程管理**: systemd
- **版本控制**: Git

## 快速开始

```bash
# 开发模式
git clone <repo>
cd claw-one
make dev

# 运行测试
make test-fast    # 单元 + 集成测试
make test-e2e     # E2E 测试
```

## 文档

| 文档 | 内容 |
|------|------|
| [docs/FINAL_DESIGN.md](docs/FINAL_DESIGN.md) | 架构、API、部署设计 |
| [docs/roadmap.md](docs/roadmap.md) | 实施路线图 |
| [docs/VISUAL_CONFIG_DESIGN.md](docs/VISUAL_CONFIG_DESIGN.md) | 前端配置界面设计 |
| [TEST_FRAMEWORK.md](TEST_FRAMEWORK.md) | 测试框架文档 |

## 项目状态

- ✅ 后端 API 实现完成
- ✅ 测试框架 90 个测试覆盖
- 🔄 Vue 3 前端开发中

---

*简洁版文档，详见 docs/ 目录*
