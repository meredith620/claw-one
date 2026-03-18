# Docker 运行 OpenClaw

使用 Docker 容器运行 OpenClaw，实现配置隔离和快速验证。

## 镜像

```
ghcr.io/openclaw/openclaw:2026.3.13-1
```

## 快速启动

### 1. 准备配置目录

```bash
mkdir -p ./dev-config ./dev-workspace

# 创建最小配置文件
cat > ./dev-config/openclaw.json << 'EOF'
{
  "logging": { "level": "info" },
  "gateway": {
    "port": 18789,
    "bind": "0.0.0.0",
    "controlUi": {
      "dangerouslyAllowHostHeaderOriginFallback": true
    }
  }
}
EOF
```

### 2. 启动 Gateway

```bash
docker run -d \
  --name openclaw-dev \
  --network host \
  -v "$(pwd)/dev-config:/home/node/.openclaw:rw" \
  -v "$(pwd)/dev-workspace:/home/node/.openclaw/workspace:rw" \
  ghcr.io/openclaw/openclaw:2026.3.13-1 \
  openclaw-gateway --allow-unconfigured
```

### 3. 验证状态

```bash
# 检查健康状态
curl http://localhost:18789/healthz

# 查看日志
docker logs -f openclaw-dev
```

### 4. 运行 CLI 命令

```bash
# 使用相同镜像运行 CLI
docker run --rm -it \
  --network host \
  -v "$(pwd)/dev-config:/home/node/.openclaw:rw" \
  -v "$(pwd)/dev-workspace:/home/node/.openclaw/workspace:rw" \
  ghcr.io/openclaw/openclaw:2026.3.13-1 \
  openclaw status
```

## 目录挂载说明

| 宿主机路径 | 容器路径 | 用途 |
|-----------|---------|------|
| `./dev-config` | `/home/node/.openclaw` | 配置文件目录 |
| `./dev-workspace` | `/home/node/.openclaw/workspace` | 工作区目录 |

## 停止和清理

```bash
# 停止容器
docker stop openclaw-dev

# 删除容器
docker rm openclaw-dev

# 完全清理（包括数据）
docker rm -f openclaw-dev
rm -rf ./dev-config ./dev-workspace
```

## 与 claw-one 配合使用

在 claw-one 配置中指定开发环境的 Gateway：

```toml
# dev-config/claw-one.toml
[openclaw]
openclaw_home = "./dev-config"
gateway_url = "http://localhost:18789"
```

## 注意事项

- 使用 `--network host` 模式使容器与宿主机共享网络
- 配置目录需要可写权限（容器内以 `node` 用户运行，UID 1000）
- 生产环境应移除 `dangerouslyAllowHostHeaderOriginFallback` 配置
