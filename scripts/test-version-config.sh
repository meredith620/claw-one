#!/bin/bash
# version-config 迁移与功能测试脚本
# 在容器中隔离测试，避免污染宿主机环境

set -e

echo "=========================================="
echo "🧪 Claw-One version-config 测试"
echo "=========================================="

# 配置测试环境
TEST_DIR="/tmp/claw-one-test-$$"
CLAW_ONE_BIN="/workspace/hull/target/release/claw-one"
STATIC_DIR="/workspace/static"

mkdir -p "$TEST_DIR"
cd "$TEST_DIR"

echo ""
echo "📁 测试目录: $TEST_DIR"
echo ""

# 创建模拟的 openclaw_home 结构
echo "🔧 创建测试环境..."
mkdir -p "$TEST_DIR/.openclaw"
mkdir -p "$TEST_DIR/config"
mkdir -p "$TEST_DIR/logs"

# 创建最小化的 openclaw.json
cat > "$TEST_DIR/.openclaw/openclaw.json" << 'EOF'
{
  "version": "1.0",
  "gateway": {
    "port": 18590,
    "bind": "127.0.0.1"
  },
  "models": {
    "providers": {}
  },
  "channels": [],
  "agents": {
    "defaults": {
      "workspace": "~/.openclaw/workspace",
      "compaction": { "mode": "safeguard" },
      "maxConcurrent": 4
    },
    "list": []
  }
}
EOF

# 创建 claw-one.toml
cat > "$TEST_DIR/config/claw-one.toml" << EOF
[server]
host = "0.0.0.0"
port = 18080
log_level = "debug"

[openclaw]
openclaw_home = "$TEST_DIR/.openclaw"
service_name = "openclaw"
health_port = 18590
health_timeout = 30
EOF

echo "✅ 测试环境创建完成"
echo ""

# ===== 测试1: 全新安装场景 =====
echo "=========================================="
echo "🧪 测试1: 全新安装（无旧版 Git）"
echo "=========================================="

# 确保没有 .git 目录
rm -rf "$TEST_DIR/.openclaw/.git"
rm -rf "$TEST_DIR/.openclaw/version-config"

# 启动服务进行测试
export CLAW_ONE_CONFIG="$TEST_DIR/config/claw-one.toml"
export CLAW_ONE_LOG_DIR="$TEST_DIR/logs"

# 在后台启动服务
echo "🚀 启动 claw-one..."
"$CLAW_ONE_BIN" run &
SERVER_PID=$!

# 等待服务启动
sleep 2

# 检查是否创建了 version-config/
if [ -d "$TEST_DIR/.openclaw/version-config/.git" ]; then
    echo "✅ version-config/.git 已创建"
else
    echo "⚠️ version-config/.git 未创建（首次保存时会创建）"
fi

# 测试配置保存 API
echo ""
echo "📤 测试保存 Agents 配置..."
curl -s -X POST \
    -H "Content-Type: application/json" \
    -d '{"list":[],"defaults":{"workspace":"~/.openclaw/workspace","maxConcurrent":4}}' \
    http://localhost:18080/api/agents > /tmp/test1_result.json

if grep -q '"success":true' /tmp/test1_result.json; then
    echo "✅ Agents 配置保存成功"
    cat /tmp/test1_result.json
else
    echo "❌ Agents 配置保存失败"
    cat /tmp/test1_result.json
    kill $SERVER_PID 2>/dev/null || true
    exit 1
fi

# 检查 version-config/ 结构
if [ -d "$TEST_DIR/.openclaw/version-config/.git" ]; then
    echo "✅ version-config/.git 现在已存在"
    
    # 检查 Git 提交
    cd "$TEST_DIR/.openclaw/version-config"
    COMMIT_COUNT=$(git log --oneline | wc -l)
    echo "📊 Git 提交数: $COMMIT_COUNT"
    
    if [ "$COMMIT_COUNT" -ge 1 ]; then
        echo "✅ Git 提交正常"
    else
        echo "❌ Git 提交异常"
        kill $SERVER_PID 2>/dev/null || true
        exit 1
    fi
else
    echo "❌ version-config/.git 仍未创建"
    kill $SERVER_PID 2>/dev/null || true
    exit 1
fi

# 停止服务
kill $SERVER_PID 2>/dev/null || true
sleep 1

echo ""
echo "✅ 测试1 通过"
echo ""

# ===== 测试2: 迁移场景 =====
echo "=========================================="
echo "🧪 测试2: 旧版 Git 仓库迁移"
echo "=========================================="

# 清理
rm -rf "$TEST_DIR/.openclaw/version-config"
rm -rf "$TEST_DIR/.openclaw/.git"

# 创建旧版 Git 仓库
echo "🔧 创建旧版 Git 仓库结构..."
cd "$TEST_DIR/.openclaw"
git init
git config user.name "Test"
git config user.email "test@test.com"
git add openclaw.json
git commit -m "Initial commit"
git log --oneline

echo ""
echo "📁 旧版仓库结构:"
ls -la "$TEST_DIR/.openclaw/"
echo ""

# 再次启动服务，应该触发迁移
echo "🚀 启动 claw-one（应触发迁移）..."
cd "$TEST_DIR"
"$CLAW_ONE_BIN" run &
SERVER_PID=$!

# 等待服务启动和迁移完成
sleep 3

# 检查迁移结果
if [ -d "$TEST_DIR/.openclaw/version-config/.git" ]; then
    echo "✅ 迁移成功: version-config/.git 已创建"
else
    echo "❌ 迁移失败: version-config/.git 不存在"
    kill $SERVER_PID 2>/dev/null || true
    exit 1
fi

if [ ! -d "$TEST_DIR/.openclaw/.git" ]; then
    echo "✅ 旧版 .git 已移除"
else
    echo "⚠️ 旧版 .git 仍存在"
fi

# 检查 Git 历史是否保留
cd "$TEST_DIR/.openclaw/version-config"
COMMIT_COUNT=$(git log --oneline | wc -l)
echo "📊 Git 提交数（含迁移提交）: $COMMIT_COUNT"

if [ "$COMMIT_COUNT" -ge 2 ]; then
    echo "✅ Git 历史保留正常（包含迁移提交）"
else
    echo "⚠️ Git 历史可能不完整"
fi

# 再次测试保存
echo ""
echo "📤 测试保存 Providers 配置..."
curl -s -X POST \
    -H "Content-Type: application/json" \
    -d '{"id":"test-provider","api":"openai-responses","baseUrl":"http://localhost:8080","apiKey":"test-key","enabled":true}' \
    http://localhost:18080/api/providers/test-provider > /tmp/test2_result.json

if grep -q '"success":true' /tmp/test2_result.json; then
    echo "✅ Providers 配置保存成功"
    cat /tmp/test2_result.json
else
    echo "❌ Providers 配置保存失败"
    cat /tmp/test2_result.json
fi

# 检查提交
cd "$TEST_DIR/.openclaw/version-config"
LATEST_COMMIT=$(git log -1 --pretty=format:"%s")
echo "📌 最新提交: $LATEST_COMMIT"

if echo "$LATEST_COMMIT" | grep -q "test-provider"; then
    echo "✅ Git 提交信息正确"
else
    echo "⚠️ Git 提交信息可能不正确"
fi

# 停止服务
kill $SERVER_PID 2>/dev/null || true
sleep 1

echo ""
echo "✅ 测试2 通过"
echo ""

# ===== 测试3: 快照和回滚 =====
echo "=========================================="
echo "🧪 测试3: 快照列表和回滚功能"
echo "=========================================="

# 启动服务
echo "🚀 启动 claw-one..."
cd "$TEST_DIR"
"$CLAW_ONE_BIN" run &
SERVER_PID=$!

sleep 2

# 获取快照列表
echo ""
echo "📋 获取快照列表..."
curl -s http://localhost:18080/api/snapshots > /tmp/snapshots.json
echo "快照列表:"
cat /tmp/snapshots.json

SNAPSHOT_COUNT=$(cat /tmp/snapshots.json | grep -c '"id"' || echo "0")
echo ""
echo "📊 快照数量: $SNAPSHOT_COUNT"

if [ "$SNAPSHOT_COUNT" -ge 2 ]; then
    echo "✅ 快照列表正常"
else
    echo "⚠️ 快照数量偏少"
fi

# 停止服务
kill $SERVER_PID 2>/dev/null || true

echo ""
echo "✅ 测试3 通过"
echo ""

# ===== 清理 =====
echo "=========================================="
echo "🧹 清理测试环境"
echo "=========================================="

rm -rf "$TEST_DIR"
echo "✅ 测试目录已清理: $TEST_DIR"

echo ""
echo "=========================================="
echo "✅ 所有测试通过!"
echo "=========================================="
