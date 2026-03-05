# Claw One

OpenClaw 配置守护程序 - 开箱即用的配置管理解决方案

## 功能特性

- 🛡️ **配置版本控制** - 基于 Git 的配置历史管理
- 🔄 **安全模式** - 配置错误自动回滚，永不锁死
- 🖥️ **Web 管理界面** - 现代化的 Vue 3 前端
- 📦 **一键部署** - 支持 systemd 服务管理
- 🚀 **首次启动向导** - 零配置开箱即用

## 快速开始

### 开发环境

```bash
# 1. 安装依赖
make deps

# 2. 编译
make compile

# 3. 调试运行
make run
```

访问 http://localhost:8080 进入管理界面

### 系统安装

```bash
# 完整构建
make compile

# 系统级安装 (需要 root)
sudo make install-system

# 启动服务
sudo systemctl enable --now claw-one
```

### 分发包安装

```bash
# 在编译机器上生成分发包
make dist

# 复制到目标机器
scp claw-one-0.1.0-install.sh user@target-host:/tmp/

# 在目标机器上执行安装
ssh user@target-host 'sudo bash /tmp/claw-one-0.1.0-install.sh'
```

## Makefile 功能说明

### 开发工作流 (顺序依赖)

| 命令 | 依赖 | 功能 |
|------|------|------|
| `make deps` | - | 安装依赖 (npm + cargo) |
| `make compile` | deps | 编译项目 (前端 + 后端) |
| `make run` | compile | 调试运行后端 |
| `make install` | run | 本地开发安装 |

### 系统级操作

| 命令 | 权限 | 功能 |
|------|------|------|
| `sudo make install-system` | root | 安装到 `/usr/local` |
| `sudo make uninstall` | root | 从系统卸载 |

### 分发打包

| 命令 | 输出 | 说明 |
|------|------|------|
| `make dist` | `.tar.gz` + 自解压脚本 | 通用 Linux 分发 |
| `make deb` | `.deb` 包 | Debian/Ubuntu 专用 |

### 开发辅助

| 命令 | 功能 |
|------|------|
| `make dev` | 开发模式 (热重载) |
| `make test` | 运行测试 |
| `make check` | 代码检查 |
| `make clean` | 清理构建产物 |

### 变量配置

```bash
# 自定义安装路径
sudo make install-system PREFIX=/opt/claw-one

# 自定义版本号打包
make dist VERSION=1.0.0
```

## 项目结构

```
claw-one/
├── backend/           # Rust 后端 (Axum)
│   └── src/
├── frontend/          # Vue 3 前端
│   └── src/
├── static/            # 构建产物目录
│   └── dist/
├── scripts/           # 安装脚本和服务配置
│   ├── install.sh
│   ├── uninstall.sh
│   └── claw-one.service
├── config/            # 配置模板
├── Makefile           # 构建脚本
└── README.md
```

## 配置说明

### 环境变量

| 变量 | 默认值 | 说明 |
|------|--------|------|
| `CLAW_ONE_CONFIG` | `/etc/claw-one/openclaw.json` | 主配置文件路径 |
| `CLAW_ONE_STATIC` | `/usr/local/share/claw-one` | 静态文件路径 |
| `RUST_LOG` | `info` | 日志级别 |

### 数据目录

- **系统配置**: `/etc/claw-one/`
- **用户数据**: `~/.config/claw-one/`
- **Git 仓库**: `~/.config/claw-one/config.git/`

## 部署架构

```
┌─────────────────────────────────────────────────────┐
│                   目标机器 (N100)                    │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────┐  │
│  │ systemd      │  │ claw-one     │  │ openclaw │  │
│  │  service     │──│  backend     │──│  gateway │  │
│  └──────────────┘  └──────────────┘  └──────────┘  │
│                           │                         │
│                      ┌────┴────┐                    │
│                      │  Vue 3  │                    │
│                      │   UI    │                    │
│                      └─────────┘                    │
└─────────────────────────────────────────────────────┘
```

## 分发安装方案

本项目支持多种分发安装方式：

### 方案 1: 自解压脚本 (推荐)

适用于通用 Linux 环境，无需依赖管理器。

```bash
# 编译机器
make dist
# 输出: claw-one-0.1.0-install.sh

# 目标机器
sudo bash claw-one-0.1.0-install.sh
```

**优点**: 单文件、无依赖、可离线安装  
**缺点**: 无版本管理、需手动更新

### 方案 2: DEB/RPM 包

适用于 Debian/Ubuntu 或 RHEL/CentOS。

```bash
# Debian/Ubuntu
make deb
sudo dpkg -i claw-one-0.1.0.deb

# 更新
sudo dpkg -i claw-one-0.2.0.deb
```

**优点**: 系统集成、依赖自动处理、版本管理  
**缺点**: 平台特定、构建复杂

### 方案 3: Docker 镜像

适用于容器化部署。

```bash
# 构建镜像
docker build -t claw-one:latest .

# 运行
docker run -d \
  -v /etc/claw-one:/etc/claw-one \
  -v ~/.config/claw-one:/var/lib/claw-one \
  -p 8080:8080 \
  claw-one:latest
```

**优点**: 环境隔离、易于扩展、跨平台  
**缺点**: 需要 Docker、资源占用

### 方案 4: 静态二进制 + 配置管理工具

适用于大规模部署（Ansible/Puppet/Chef）。

```yaml
# playbook.yml
- hosts: openclaw_servers
  tasks:
    - name: Install Claw One
      ansible.builtin.shell: |
        curl -fsSL https://releases.claw.one/install.sh | sudo bash
```

**优点**: 自动化、可扩展、集中管理  
**缺点**: 需要配置管理基础设施

### 推荐方案

| 场景 | 推荐方案 |
|------|---------|
| 单机/边缘设备 (N100) | 自解压脚本 |
| 小规模集群 (<10台) | DEB 包 + 脚本 |
| 大规模部署 (>10台) | Ansible + 静态二进制 |
| 云原生环境 | Docker |

## 升级流程

```bash
# 1. 备份配置
cp -r ~/.config/claw-one ~/.config/claw-one.backup

# 2. 停止服务
sudo systemctl stop claw-one

# 3. 安装新版本
sudo bash claw-one-new-version-install.sh

# 4. 重启服务
sudo systemctl start claw-one

# 5. 验证
systemctl status claw-one
curl http://localhost:8080/api/health
```

## 许可证

MIT License
