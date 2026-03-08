# Claw One 用户级安装包

## 快速开始

### 1. 检查环境

```bash
./scripts/check-env.sh
```

检查系统依赖、端口占用、磁盘空间等。

### 2. 安装

```bash
./scripts/install.sh
```

一键安装到 `~/claw-one/`，无需 root 权限。

### 3. 配置（交互式）

```bash
~/claw-one/bin/setup-config.sh
```

配置端口、OpenClaw 路径、systemd 服务等。

### 4. 访问

```
http://localhost:8080
```

## 目录结构

```
~/claw-one/
├── bin/
│   ├── claw-one            # 主程序
│   └── setup-config.sh     # 配置向导
├── config/                 # 配置文件
│   ├── claw-one.toml       # 主配置
│   └── openclaw.json       # OpenClaw 配置
├── data/                   # 数据（Git 版本控制）
├── logs/                   # 日志文件
└── share/
    └── static/             # 前端静态文件
```

## 管理命令

### 手动启动

```bash
~/claw-one/bin/claw-one run
```

### systemd 用户服务

```bash
# 启动
systemctl --user start claw-one

# 停止
systemctl --user stop claw-one

# 查看状态
systemctl --user status claw-one

# 设置开机自启
systemctl --user enable claw-one

# 查看日志
journalctl --user -u claw-one -f
```

### 重新配置

```bash
~/claw-one/bin/setup-config.sh
```

## 卸载

```bash
~/claw-one/uninstall.sh
```

卸载时会询问是否保留：
- 配置文件
- 数据目录（Git 历史）
- 日志文件

## 环境要求

**必需依赖：**
- `git` - 用于配置版本控制

**推荐依赖：**
- `systemd` - 用于服务管理（可选，可手动启动）
- `curl` 或 `wget` - 用于健康检查
- `lsof` 或 `ss` - 用于端口检查

**系统要求：**
- Linux 系统（推荐 Ubuntu/Debian/CentOS）
- macOS 实验性支持
- 磁盘空间 > 200MB
- 端口 8080 可用（可配置）

## 配置说明

### 修改端口

编辑 `~/claw-one/config/claw-one.toml`：

```toml
[server]
port = 8080  # 修改为你想要的端口
```

### 修改 OpenClaw 路径

编辑 `~/claw-one/config/claw-one.toml`：

```toml
[openclaw]
openclaw_home = "/path/to/openclaw"
```

## 故障排除

### 端口被占用

```bash
# 检查端口占用
lsof -i :8080

# 修改端口后重新配置
~/claw-one/bin/setup-config.sh
```

### 服务无法启动

```bash
# 查看日志
cat ~/claw-one/logs/*.log

# 或使用 systemd
journalctl --user -u claw-one
```

### 重置配置

```bash
# 备份当前配置
cp ~/claw-one/config ~/claw-one/config.backup

# 恢复默认配置
cp ~/claw-one/share/config/*.template ~/claw-one/config/

# 重新配置
~/claw-one/bin/setup-config.sh
```

## 更多信息

- 项目文档: https://github.com/meredith620/claw-one
- 问题反馈: https://github.com/meredith620/claw-one/issues
