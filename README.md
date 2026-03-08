# Claw One

OpenClaw 配置守护程序 - 开箱即用的配置管理解决方案

## 功能特性

- 🛡️ **配置版本控制** - 基于 Git 的配置历史管理
- 🔄 **安全模式** - 配置错误自动回滚，永不锁死
- 🖥️ **Web 管理界面** - 现代化的 Vue 3 前端
- 📦 **用户级安装** - 无需 root，安装到 ~/claw-one
- 🚀 **systemd 用户服务** - 当前用户开机自启动
- 📝 **灵活配置** - 通过 TOML 配置管理任意 OpenClaw 实例

## 快速开始

### 开发环境

```bash
# 1. 安装依赖
make deps

# 2. 创建开发配置
cp config/claw-one.toml.example hull/config.dev.toml
# 编辑 hull/config.dev.toml 设置你的 OpenClaw 连接信息

# 3. 启动开发模式
make dev
```

访问 http://localhost:8080 进入管理界面

### 用户级安装（推荐）

```bash
# 编译并安装到 ~/claw-one
make install

# 按照提示编辑配置文件
nano ~/claw-one/config/claw-one.toml

# 使用 CLI 启动/停止/查看状态
claw-one start      # 后台启动
claw-one status     # 查看状态
claw-one stop       # 停止服务

# 配置开机自启
claw-one enable     # 配置开机自启（systemd user）
claw-one disable    # 取消开机自启
```

### CLI 命令说明

安装后，`claw-one` 支持以下命令：

```bash
# 服务管理
claw-one run           # 前台运行（默认）
claw-one start         # 后台启动（优先使用 systemd）
claw-one start -d      # 后台启动（守护进程模式）
claw-one stop          # 停止服务
claw-one restart       # 重启服务
claw-one status        # 查看运行状态

# 开机自启管理
claw-one enable        # 配置开机自启
claw-one disable       # 取消开机自启

# 配置查看
claw-one config        # 显示当前配置
claw-one -c /path/to/config.toml run  # 指定配置文件
```

### 分发包安装 (推荐)

```bash
# 在编译机器上生成分发包 (使用 Docker，musl 静态链接，兼容所有 Linux 发行版)
make dist

# 复制到目标机器
scp dist/claw-one-VERSION-x86_64-install.sh user@target-host:/tmp/

# 在目标机器上执行安装
bash /tmp/claw-one-VERSION-x86_64-install.sh

# 编辑配置
nano ~/claw-one/config/claw-one.toml

# 启动服务
claw-one run      # 前台运行
# 或
claw-one start    # 后台启动
```

## 配置文件

Claw One 使用 TOML 格式的配置文件，默认路径：`~/claw-one/config/claw-one.toml`

```toml
[server]
host = "0.0.0.0"
port = 8080
log_level = "info"

[openclaw]
openclaw_home = "~/.openclaw"   # OpenClaw 安装根目录
service_name = "openclaw"       # 要管理的 OpenClaw 服务名
health_port = 18790             # OpenClaw 健康检查端口
health_timeout = 30             # 健康检查超时（秒）

[paths]
data_dir = "~/.config/claw-one"  # Claw One 数据目录

[features]
auto_backup = true
safe_mode = true
first_run_wizard = true
```

配置项说明：

| 配置项 | 说明 | 默认值 |
|--------|------|--------|
| `openclaw_home` | OpenClaw 安装根目录，配置文件为 `{openclaw_home}/openclaw.json` | `~/.openclaw` |
| `service_name` | OpenClaw systemd 用户服务名称 | `openclaw` |
| `health_port` | OpenClaw 健康检查端口 | `18790` |
| `config_path` | 覆盖 `openclaw_home` 的配置文件路径（高级） | `""` |

### 多实例管理

通过配置不同的 `openclaw_home` 和 `service_name`，可以管理多个 OpenClaw 实例：

```toml
# 管理测试环境的 OpenClaw
[openclaw]
openclaw_home = "~/.openclaw-test"
service_name = "openclaw-test"
health_port = 18791
```

上述配置将：
- 管理 `~/.openclaw-test/openclaw.json` 配置文件
- 控制 `systemctl --user openclaw-test` 服务

## Makefile 功能说明

### 开发工作流

| 命令 | 说明 |
|------|------|
| `make deps` | 安装依赖 (npm + cargo) |
| `make compile` | 编译项目 (前端 + 后端) |
| `make install` | 安装到 ~/claw-one（用户级）|
| `make uninstall` | 从 ~/claw-one 卸载 |

### 开发辅助

| 命令 | 说明 |
|------|------|
| `make dev` | 开发模式（需要 hull/config.dev.toml）|
| `make check` | 代码检查 |
| `make check-env` | 检测安装环境依赖 |
| `make clean` | 清理构建产物 |

### 分发包安装脚本

分发包中的 `install.sh` 支持以下命令：

```bash
./install.sh check       # 检测环境依赖（git、systemd 等）
./install.sh install     # 安装（默认）
./install.sh uninstall   # 卸载
./install.sh help        # 显示帮助
```

### 分发打包

| 命令 | 输出 | 说明 |
|------|------|------|
| `make dist` | `.tar.gz` + 自解压脚本 | 默认安装到 ~/claw-one |

### 变量

```bash
# 自定义安装路径
make install INSTALL_DIR=~/my-claw-one

# 自定义版本号打包
make dist VERSION=1.0.0
```

## 项目结构

```
~/claw-one/              # 安装目录
├── bin/
│   └── claw-one # 核心程序
├── share/
│   └── static/          # 前端静态文件
├── config/
│   └── claw-one.toml    # 配置文件
└── ...

~/.config/claw-one/      # 数据目录
├── config.git/          # Git 仓库
├── factory-config.json  # 出厂配置备份
└── ...
```

## systemd 用户服务

Claw One 使用 systemd **用户服务**模式（`systemctl --user`），无需 root 权限即可管理开机自启：

```bash
# 查看服务状态
systemctl --user status claw-one

# 启动/停止/重启（使用 --user 标志）
systemctl --user start claw-one
systemctl --user stop claw-one
systemctl --user restart claw-one

# 设置/取消开机自启
systemctl --user enable claw-one
systemctl --user disable claw-one

# 查看日志
journalctl --user -u claw-one -f

# 重新加载 systemd 配置（安装/更新后需要）
systemctl --user daemon-reload
```

**注意**：Claw One 控制 OpenClaw 也使用 `systemctl --user` 命令，确保 OpenClaw 同样以用户服务运行。

## 与 OpenClaw 的关系

```
┌─────────────────────────────────────────────┐
│                 用户空间                      │
│  ┌──────────┐      ┌─────────────────────┐  │
│  │ Claw One │──────│   OpenClaw Service  │  │
│  │  (8080)  │管理  │     (systemd)       │  │
│  └──────────┘      └─────────────────────┘  │
│        │                    │               │
│        │ 读写配置     ┌─────┴─────┐         │
│        ▼              ▼           ▼         │
│  ┌──────────┐    ┌────────┐  ┌────────┐    │
│  │  Git仓库  │    │ 模型API │  │ 渠道   │    │
│  └──────────┘    └────────┘  └────────┘    │
└─────────────────────────────────────────────┘
```

## 升级流程

```bash
# 1. 备份配置
cp -r ~/claw-one/config ~/claw-one-config.backup

# 2. 停止服务
claw-one stop

# 3. 安装新版本
bash claw-one-new-version-install.sh

# 4. 恢复配置（如需要）
cp ~/claw-one-config.backup/claw-one.toml ~/claw-one/config/

# 5. 启动服务
claw-one start

# 6. 验证
curl http://localhost:8080/api/health
```

## 许可证

MIT License
