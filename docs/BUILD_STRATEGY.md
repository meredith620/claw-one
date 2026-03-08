# Claw One 构建方案总结

## 状态更新 (2026-03-08)

### ✅ 需求1: 自解压安装脚本

**已实现**

```bash
# 创建自解压安装脚本
make self-extract

# 输出
dist/claw-one-VERSION-ARCH-install.sh
```

**特性:**
- 单文件分发 (~4.4MB)
- 内嵌 base64 编码的 tar.gz 包
- 支持命令行选项：`--check`, `--target`, `--yes`
- 自动检测环境依赖
- 自动解压并运行安装

**测试状态:** ✅ 容器测试通过

---

### 🔄 需求2: 多发行版兼容 (musl 静态链接)

**方案设计完成，部分实现**

#### 推荐方案: musl 静态链接

**原因:**
1. **MIT 授权** - 无 GPL 污染风险
2. **真正独立** - 零系统依赖
3. **体积优化** - 比 glibc 静态链接小 3-5MB
4. **Alpine 生态** - Docker 主流，工具链成熟

#### 实现方式

由于主机缺少 `musl-gcc`，提供两种构建方式:

**方式A: Docker 构建 (推荐)**
```bash
# 使用 Alpine 容器构建
./scripts/build-musl.sh

# 输出
# - dist/claw-one-VERSION-x86_64-musl-install.sh
```

**方式B: 系统安装 musl-tools**
```bash
# Ubuntu/Debian
sudo apt-get install musl-tools

# 然后使用 Makefile
make musl-self-extract
```

#### 兼容性对比

| 发行版 | glibc 版本 | 动态构建 | musl 静态 |
|--------|-----------|----------|-----------|
| Ubuntu 24.04 | 2.39 | ✅ | ✅ |
| Ubuntu 22.04 | 2.35 | ❌ | ✅ |
| Ubuntu 20.04 | 2.31 | ❌ | ✅ |
| Debian 12 | 2.36 | ❌ | ✅ |
| Debian 11 | 2.31 | ❌ | ✅ |
| CentOS 7 | 2.17 | ❌ | ✅ |
| CentOS 8 | 2.28 | ❌ | ✅ |
| Alpine 3.x | musl | ❌ | ✅ |

---

## Makefile 新增目标

```bashnmake self-extract      # 创建 glibc 自解压脚本
make musl               # 构建 musl 静态二进制（需 musl-tools）
make musl-dist          # 创建 musl 分发包
make musl-self-extract  # 创建 musl 自解压脚本（需 musl-tools）
```

---

## 建议的发布策略

**主发布 (glibc):**
- `claw-one-VERSION-x86_64-install.sh` 
- 适用于现代发行版 (Ubuntu 24.04+, Debian 12+)

**兼容发布 (musl):**
- `claw-one-VERSION-x86_64-musl-install.sh`
- 适用于所有 Linux 发行版
- 推荐用于生产环境

---

## 下一步行动

1. ✅ 自解压脚本 - 已完成并测试
2. 🔄 musl 构建 - 提供 Docker 脚本，待执行测试
3. 📋 CI/CD 集成 - 建议 GitHub Actions 同时构建两种版本
