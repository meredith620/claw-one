#!/bin/bash
set -e

BASE_URL="http://localhost:28080"

# Reset environment to simulate first-time setup
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
bash "$SCRIPT_DIR/../scripts/test-env-reset.sh"

echo "=========================================="
echo "流程 A: 首次启动配置测试"
echo "=========================================="

# Step 1: Check first-time setup status
echo ""
echo "Step 1: 检查首次启动状态..."
SETUP_CHECK=$(curl -s "$BASE_URL/api/setup/check")
echo "Setup check response: $SETUP_CHECK"

if echo "$SETUP_CHECK" | grep -q '"is_first_setup":true'; then
    echo "✅ 检测到首次启动状态"
else
    echo "⚠️  不是首次启动状态，但继续测试..."
fi

# Step 2: Configure providers (Model configuration step)
echo ""
echo "Step 2: 配置模型 Provider..."
curl -s -X POST "$BASE_URL/api/providers/moonshot-main" \
    -H "Content-Type: application/json" \
    -d '{
        "api": "openai-completions",
        "apiKey": "sk-moonshot-test",
        "baseUrl": "https://api.moonshot.cn/v1",
        "enabled": true,
        "defaultModel": "kimi-k2.5"
    }' | grep -q '"success":true' || { echo "❌ Provider 配置失败"; exit 1; }
echo "✅ Moonshot Provider 配置成功"

# Step 3: Set model priority
echo ""
echo "Step 3: 设置模型优先级..."
curl -s -X POST "$BASE_URL/api/model-priority" \
    -H "Content-Type: application/json" \
    -d '{
        "primary": "moonshot-main/kimi-k2.5",
        "fallbacks": []
    }' | grep -q '"success":true' || { echo "❌ 模型优先级设置失败"; exit 1; }
echo "✅ 模型优先级设置成功"

# Step 4: Configure channels
echo ""
echo "Step 4: 配置渠道..."
curl -s -X POST "$BASE_URL/api/channels" \
    -H "Content-Type: application/json" \
    -d '{
        "mattermost": {
            "enabled": true,
            "dmPolicy": "pairing",
            "groupPolicy": "allowlist",
            "accounts": {
                "default": {
                    "name": "Main Bot",
                    "botToken": "mm-test-token",
                    "baseUrl": "https://mattermost.example.com"
                }
            }
        }
    }' | grep -q '"success":true' || { echo "❌ 渠道配置失败"; exit 1; }
echo "✅ Mattermost 渠道配置成功"

# Step 5: Complete setup
echo ""
echo "Step 5: 完成初始化..."
curl -s -X POST "$BASE_URL/api/setup/complete" | grep -q '"success":true' || { echo "❌ 完成初始化失败"; exit 1; }
echo "✅ 初始化完成"

# Step 6: Verify configuration is saved to openclaw.json
echo ""
echo "Step 6: 验证配置已保存..."
CONFIG_CHECK=$(docker exec claw-one-test-openclaw cat /root/.openclaw/openclaw.json)

if echo "$CONFIG_CHECK" | grep -q "moonshot-main"; then
    echo "✅ Provider 配置已保存到 openclaw.json"
else
    echo "❌ Provider 配置未找到"
    exit 1
fi

if echo "$CONFIG_CHECK" | grep -q "mattermost"; then
    echo "✅ 渠道配置已保存到 openclaw.json"
else
    echo "❌ 渠道配置未找到"
    exit 1
fi

# Step 7: Verify Git commit was created
echo ""
echo "Step 7: 验证 Git 有初始提交..."
SNAPSHOTS=$(curl -s "$BASE_URL/api/snapshots")
SNAPSHOT_COUNT=$(echo "$SNAPSHOTS" | grep -o '"id"' | wc -l)

if [ "$SNAPSHOT_COUNT" -ge 1 ]; then
    echo "✅ Git 有 $SNAPSHOT_COUNT 个提交"
else
    echo "⚠️  Git 提交数量异常: $SNAPSHOT_COUNT"
fi

# Step 8: Verify setup is marked as complete
echo ""
echo "Step 8: 验证初始化标记..."
SETUP_CHECK=$(curl -s "$BASE_URL/api/setup/check")
if echo "$SETUP_CHECK" | grep -q '"is_first_setup":false'; then
    echo "✅ 初始化标记已设置"
else
    echo "⚠️  初始化标记可能未设置: $SETUP_CHECK"
fi

# Step 9: Verify we can access main interface
echo ""
echo "Step 9: 验证主界面可访问..."
HEALTH=$(curl -s "$BASE_URL/api/health")
if echo "$HEALTH" | grep -q '"status":"ok"'; then
    echo "✅ 服务健康检查通过"
else
    echo "❌ 服务健康检查失败: $HEALTH"
    exit 1
fi

echo ""
echo "=========================================="
echo "✅ 流程 A: 首次启动配置测试通过"
echo "=========================================="
exit 0
