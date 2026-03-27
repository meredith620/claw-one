# Claw One 构建策略

**版本**: 2026-03-27  
**状态**: musl 静态链接作为默认构建方式

---

## 快速开始

```bash
# 推荐：Docker 构建（无需本地工具链）
make dist

# 或本地构建（需要 Rust + Node.js）
make dist-native
```

---

## 构建方式对比

| 方式 | 命令 | 输出 | 用途 |
|------|------|------|------|
| **Docker (推荐)** | `make dist` | 自解压脚本 | 分发 |
| 本地 | `make dist-native` | 动态链接包 | 测试 |

---

## 兼容性

| 发行版 | `make dist` (musl) |
|--------|-------------------|
| Ubuntu 24.04/22.04/20.04 | ✅ |
| Debian 12/11 | ✅ |
| CentOS 7/8 | ✅ |
| Alpine 3.x | ✅ |

---

## Makefile 目标

| 目标 | 说明 |
|------|------|
| `make dist` | 默认分发包 (Docker musl) |
| `make install` | 自解压脚本 |
| `make dist-native` | 本地构建 (测试用) |
| `make test-fast` | 运行测试 |
| `make test-e2e` | E2E 测试 |

---

## 为什么选择 musl?

| 特性 | glibc | musl |
|------|-------|------|
| 授权 | LGPL | MIT |
| 体积 | +5-10MB | +1-2MB |
| 兼容性 | 好 | **极好** |

---

## 相关文档

- [TEST_FRAMEWORK.md](../TEST_FRAMEWORK.md) - 测试框架
- [FINAL_DESIGN.md](./FINAL_DESIGN.md) - 架构设计
