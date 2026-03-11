#!/bin/bash
# Claw One 容器测试脚本
# 测试 memory, channel, agent, model 四个模块

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

echo "=========================================="
echo "Claw One 容器测试"
echo "=========================================="

# 创建临时目录
TEST_DIR=$(mktemp -d)
echo "测试目录: $TEST_DIR"

# 复制必要的文件
cp "$PROJECT_ROOT/dist"/*-install.sh "$TEST_DIR/" 2>/dev/null || {
    echo "错误: 未找到安装包，请先运行 make dist"
    exit 1
}

# 获取安装包名称
INSTALLER=$(ls "$TEST_DIR"/*-install.sh | head -1)
echo "安装包: $(basename $INSTALLER)"

# 创建 Dockerfile
cat > "$TEST_DIR/Dockerfile" << 'EOF'
FROM alpine:3.21

# 安装必要的依赖
RUN apk add --no-cache \
    bash \
    curl \
    git \
    nodejs \
    npm \
    ca-certificates \
    libstdc++

# 创建测试用户
RUN adduser -D -s /bin/bash testuser

# 创建 openclaw 目录并复制配置
RUN mkdir -p /home/testuser/.openclaw
COPY demo-openclaw.json /home/testuser/.openclaw/openclaw.json
RUN chown -R testuser:testuser /home/testuser/.openclaw

# 复制安装包
COPY *-install.sh /tmp/
RUN chmod +x /tmp/*-install.sh

# 安装 claw-one
USER testuser
RUN /tmp/*-install.sh -y -t /home/testuser/claw-one || true

# 创建 claw-one 配置
RUN mkdir -p /home/testuser/claw-one/config
RUN echo 'port = 8080' > /home/testuser/claw-one/config/claw-one.toml
RUN echo 'openclaw_home = "/home/testuser/.openclaw"' >> /home/testuser/claw-one/config/claw-one.toml
RUN echo 'log_level = "debug"' >> /home/testuser/claw-one/config/claw-one.toml

EXPOSE 8080

WORKDIR /home/testuser
CMD ["/home/testuser/claw-one/bin/claw-one", "run"]
EOF

# 复制 demo-openclaw.json
cp "$PROJECT_ROOT/../demo-openclaw.json" "$TEST_DIR/" 2>/dev/null || \
cp /home/lvliang/.openclaw/workspace-architecturer/demo-openclaw.json "$TEST_DIR/"

echo ""
echo "构建测试镜像..."
docker build -t claw-one-test "$TEST_DIR"

echo ""
echo "启动测试容器..."
CONTAINER_ID=$(docker run -d -p 18080:8080 --name claw-one-test claw-one-test)
echo "容器 ID: $CONTAINER_ID"

# 等待服务启动
echo ""
echo "等待服务启动..."
sleep 5

# 检查服务是否运行
echo ""
echo "检查服务状态..."
docker logs claw-one-test 2>&1 | tail -30

# 测试 API
echo ""
echo "=========================================="
echo "测试 API"
echo "=========================================="

BASE_URL="http://localhost:18080"

# 1. 测试 Model/Provider API
echo ""
echo "[1/4] 测试 Model/Provider API..."
echo "GET $BASE_URL/api/providers"
PROVIDERS=$(curl -s "$BASE_URL/api/providers" 2>/dev/null || echo '{"error": "failed"}')
echo "响应: $PROVIDERS"

# 2. 测试 Agent API
echo ""
echo "[2/4] 测试 Agent API..."
echo "GET $BASE_URL/api/agents"
AGENTS=$(curl -s "$BASE_URL/api/agents" 2>/dev/null || echo '{"error": "failed"}')
echo "响应: $AGENTS"

# 3. 测试 Channel API
echo ""
echo "[3/4] 测试 Channel API..."
echo "GET $BASE_URL/api/channels"
CHANNELS=$(curl -s "$BASE_URL/api/channels" 2>/dev/null || echo '{"error": "failed"}')
echo "响应: $CHANNELS"

# 4. 测试 Memory API
echo ""
echo "[4/4] 测试 Memory API..."
echo "GET $BASE_URL/api/memory"
MEMORY=$(curl -s "$BASE_URL/api/memory" 2>/dev/null || echo '{"error": "failed"}')
echo "响应: $MEMORY"

# 验证结果
echo ""
echo "=========================================="
echo "验证结果"
echo "=========================================="

# 检查 demo-openclaw.json 中的关键配置
echo ""
echo "Demo 配置中的内容:"
echo "- Providers: redfrog, redfrog2, kimi-coding"
echo "- Agents: main, architecturer, developer"
echo "- Channels: mattermost"
echo "- Memory: provider=ollama, baseUrl=http://10.10.10.86:11434"

echo ""
echo "API 返回的内容:"
echo "- Providers: $(echo $PROVIDERS | grep -o '"id"' | wc -l) 个"
echo "- Agents list: $(echo $AGENTS | grep -o '"id"' | wc -l) 个"
echo "- Channels mattermost enabled: $(echo $CHANNELS | grep -o '"enabled":true' | wc -l)"
echo "- Memory provider: $(echo $MEMORY | grep -o '"provider":"[^"]*"' | head -1)"

# 清理
echo ""
echo "=========================================="
echo "清理"
echo "=========================================="
docker stop claw-one-test 2>/dev/null || true
docker rm claw-one-test 2>/dev/null || true
rm -rf "$TEST_DIR"

echo "测试完成"
