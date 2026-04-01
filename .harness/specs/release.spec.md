---
# HARNESS METADATA
# type: specification
# part-of: harness-architecture
# scope: release
# managed-by: harness-system
# do-not-modify-manually: 如需修改请遵循 harness-evolution.spec.md
# version: 1.0
# created: 2026-04-01
---

# 发布构建规范

> **适用于:** 版本发布、分发包构建、安装脚本修改
> 
> ⚠️ **HARNESS 文件**: 本文档属于 Harness 架构，修改需谨慎

## 构建流程

```
┌─────────────┐    ┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│ 检查环境    │ → │ 构建编译镜像 │ → │ 编译应用    │ → │ 打包分发    │
└─────────────┘    └─────────────┘    └─────────────┘    └─────────────┘
   make deps         make builder       make dist         dist/*.tar.gz
```

## 两阶段 Docker 构建

### 阶段 1: 编译环境镜像

**目的:** 避免每次构建都重新下载依赖

```dockerfile
# scripts/Dockerfile.builder
FROM rust:1.75-alpine3.19

# 安装构建依赖
RUN apk add --no-cache \
    musl-dev \
    openssl-dev \
    openssl-libs-static \
    pkgconfig \
    git \
    nodejs \
    npm

# 预编译依赖（缓存层）
WORKDIR /build
COPY hull/Cargo.toml hull/Cargo.lock ./
RUN mkdir -p src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
```

**镜像命名:** `claw-one-builder:{VERSION}`

### 阶段 2: 应用构建

```bash
# scripts/build-dist.sh
#!/bin/bash
set -e

# 1. 使用编译环境镜像构建
docker run --rm \
    -v "$(pwd):/build" \
    -w /build \
    "claw-one-builder:${VERSION}" \
    sh -c '
        # 构建 hull (Rust)
        cd hull && cargo build --release --target x86_64-unknown-linux-musl
        
        # 构建 bridge (Vue)
        cd ../bridge && npm install && npm run build
    '

# 2. 打包
cp hull/target/x86_64-unknown-linux-musl/release/claw-one dist/bin/
cp -r static/dist/* dist/share/static/
```

## 构建命令

### 开发构建

```bash
# 本地快速构建（动态链接）
make build

# 仅后端
make hull

# 仅前端
make bridge
```

### 分发构建

```bash
# 完整分发包（推荐）
make dist
# 输出: dist/claw-one-{VERSION}-{ARCH}.tar.gz

# 包含自解压脚本
make install
# 输出: dist/claw-one-{VERSION}-{ARCH}-install.sh
```

### 本地测试构建

```bash
# ⚠️ 仅本地测试，不用于分发
make dist-native
# 输出: dist/claw-one-{VERSION}-{ARCH}.tar.gz（动态链接）
```

**为什么不用于分发:**
- 动态链接 glibc，不同 Linux 发行版版本不兼容
- 需要目标系统安装兼容的库版本

## 版本管理

### 版本号规则

使用语义化版本：`MAJOR.MINOR.PATCH`

| 版本变化 | 说明 |
|----------|------|
| MAJOR | 不兼容的 API 变更 |
| MINOR | 向后兼容的功能添加 |
| PATCH | 向后兼容的问题修复 |

### 版本标记

```bash
# 1. 更新版本号（Cargo.toml）
vim hull/Cargo.toml  # version = "0.2.0"

# 2. 提交
git add -A
git commit -m "bump version to 0.2.0"

# 3. 打标签
git tag -a v0.2.0 -m "Release version 0.2.0"
git push origin v0.2.0
```

### 版本获取

```makefile
VERSION := $(shell git describe --tags --always --dirty 2>/dev/null || echo "0.1.0")
# v0.2.0-3-gabc123-dirty
```

## 分发包内容

```
claw-one-{VERSION}-{ARCH}.tar.gz
├── bin/
│   └── claw-one              # 可执行文件 (musl 静态链接)
├── share/
│   ├── static/               # Vue 前端构建产物
│   │   ├── index.html
│   │   └── assets/
│   └── config/
│       └── claw-one.toml.template  # 配置模板
├── scripts/
│   ├── install.sh            # 安装脚本
│   └── uninstall.sh          # 卸载脚本
└── README.md                 # 说明文档
```

## 安装脚本规范

### 安装流程

```bash
# install.sh
1. 检测环境依赖 (check-env.sh)
   - systemd 是否可用
   - 端口是否被占用
   - 磁盘空间

2. 创建目录结构
   ~/claw-one/
   ├── bin/claw-one
   ├── share/static/
   └── config/

3. 复制文件

4. 创建 systemd 用户服务
   ~/.config/systemd/user/claw-one.service

5. 提示用户编辑配置
```

### 安装路径

| 路径 | 用途 | 说明 |
|------|------|------|
| `~/claw-one/` | 安装目录 | 用户级，无需 root |
| `~/.config/claw-one/` | 数据目录 | Git 仓库、快照 |
| `~/.config/systemd/user/` | 服务配置 | systemd 用户服务 |

### 检查脚本

```bash
# scripts/check-env.sh 必须检查:
- [ ] systemd --user 可用
- [ ] 8080 端口可用（或可配置）
- [ ] OpenClaw 已安装
- [ ] 磁盘空间 > 100MB
- [ ] git 已安装
```

## CI/CD 集成

### GitHub Actions 示例

```yaml
# .github/workflows/release.yml
name: Release

on:
  push:
    tags: ['v*']

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Build
        run: make dist
      
      - name: Generate checksums
        run: make dist-check
      
      - name: Create Release
        uses: softprops/action-gh-release@v1
        with:
          files: |
            dist/*-install.sh
            dist/SHA256SUMS
```

## 发布检查清单

发布新版本前必须确认：

- [ ] 版本号已更新（`hull/Cargo.toml`）
- [ ] CHANGELOG.md 已更新
- [ ] `make test-all` 通过
- [ ] `make dist` 成功
- [ ] 安装脚本在新环境测试通过
- [ ] 卸载脚本测试通过
- [ ] 升级路径测试（从上一版本）
- [ ] Git tag 已打并推送

## 升级兼容性

### 配置迁移

```rust
// src/config.rs
pub async fn migrate_config(config: &mut Value) -> Result<(), Error> {
    let version = config.get("version").and_then(|v| v.as_str())
        .unwrap_or("0.1.0");
    
    if version < "0.2.0" {
        // 从 0.1.x 迁移到 0.2.x
        migrate_v0_1_to_v0_2(config).await?;
    }
    
    Ok(())
}
```

### 数据库/状态迁移

- 配置：Git 快照自动管理版本
- 状态：无持久化状态（从 OpenClaw 获取）
- 日志：保留在 ~/.openclaw/logs/
