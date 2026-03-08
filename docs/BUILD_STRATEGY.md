# Claw One 构建策略

**版本**: 2026-03-08  
**状态**: musl 静态链接作为默认构建方式

---

## 默认构建: musl 静态链接

从 2026-03-08 开始，Claw One **默认使用 musl 静态链接**构建，原因：

1. **最大兼容性** - 单二进制可在所有 Linux 发行版运行
2. **零依赖** - 不依赖系统 glibc 版本
3. **MIT 授权** - 无 GPL 污染风险
4. **体积小** - 比 glibc 静态链接小 3-5MB

---

## 构建方式

### 方式1: Docker 构建 (推荐)

无需安装任何工具链，仅需 Docker：

```bash
make dist
# 或
make install
```

**输出:**
- `dist/claw-one-VERSION-x86_64-install.sh` - 自解压安装脚本
- `dist/claw-one-VERSION-x86_64.tar.gz` - 分发包

### 方式2: 本地构建 (快速测试)

需要 Rust + Node.js 环境：

```bash
make dist-native
# 或
make install-native
```

**注意:** 本地构建使用动态链接，仅适用于当前系统，不推荐分发。

---

## 兼容性矩阵

| 发行版 | glibc 版本 | `make dist-native` | `make dist` (musl) |
|--------|-----------|-------------------|-------------------|
| Ubuntu 24.04 | 2.39 | ✅ | ✅ |
| Ubuntu 22.04 | 2.35 | ❌ | ✅ |
| Ubuntu 20.04 | 2.31 | ❌ | ✅ |
| Debian 12 | 2.36 | ❌ | ✅ |
| Debian 11 | 2.31 | ❌ | ✅ |
| CentOS 7 | 2.17 | ❌ | ✅ |
| CentOS 8 | 2.28 | ❌ | ✅ |
| Alpine 3.x | musl | ❌ | ✅ |

---

## Makefile 目标

| 目标 | 说明 | 构建方式 |
|------|------|----------|
| `make dist` | **默认分发包** (推荐) | Docker musl |
| `make install` | **默认自解压脚本** (推荐) | Docker musl |
| `make dist-native` | 本地分发包 (测试用) | 本地动态链接 |
| `make install-native` | 本地自解压脚本 (测试用) | 本地动态链接 |
| `make dist-check` | 生成校验和 | - |

---

## 技术细节

### 为什么选择 musl?

**与 glibc 静态链接对比:**

| 特性 | glibc 静态 | musl 静态 |
|------|-----------|-----------|
| 授权 | LGPL (风险) | MIT (安全) |
| 体积 | +5-10MB | +1-2MB |
| DNS/NSS | 可能有问题 | 正常工作 |
| 兼容性 | 好 | **极好** |

### 构建流程

```
make dist
    ↓
./scripts/build-musl.sh
    ↓
Docker (rust:alpine)
    ├── 构建 bridge (npm/vite)
    ├── 构建 hull (cargo musl)
    └── 打包输出
```

---

## 下一步

1. 📦 运行 `make dist` 生成分发包
2. 🧪 在 CentOS 7/Ubuntu 20.04 测试
3. 📋 配置 CI/CD 自动构建
