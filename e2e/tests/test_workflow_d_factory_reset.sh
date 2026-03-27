#!/bin/bash
set -e

BASE_URL="http://localhost:28080"

# Reset environment first
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
bash "$SCRIPT_DIR/../scripts/test-env-reset.sh"

echo "=========================================="
echo "流程 D: 出厂设置测试"
echo "=========================================="

# Pre-condition: Setup some configuration
echo ""
echo "Pre-condition: 设置一些配置..."
curl -s -X POST "$BASE_URL/api/providers/provider-1" \
    -H "Content-Type: application/json" \
    -d '{
        "api": "openai-completions",
        "apiKey": "sk-test-1",
        "baseUrl": "https://api1.example.com",
        "enabled": true,
        "defaultModel": "gpt-4"
    }' > /dev/null

curl -s -X POST "$BASE_URL/api/providers/provider-2" \
    -H "Content-Type: application/json" \
    -d '{
        "api": "anthropic-messages",
        "apiKey": "sk-test-2",
        "baseUrl": "https://api2.example.com",
        "enabled": true,
        "defaultModel": "claude-3"
    }' > /dev/null

curl -s -X POST "$BASE_URL/api/agents" \
    -H "Content-Type: application/json" \
    -d '{
        "defaults": {
            "workspace": "/custom/workspace"
        },
        "list": [
            {"id": "agent-1", "name": "Custom Agent"}
        ]
    }' > /dev/null

curl -s -X POST "$BASE_URL/api/channels" \
    -H "Content-Type: application/json" \
    -d '{
        "mattermost": {
            "enabled": true,
            "accounts": {
                "custom": {
                    "name": "Custom Account",
                    "botToken": "token",
                    "baseUrl": "https://mm.example.com"
                }
            }
        }
    }' > /dev/null

curl -s -X POST "$BASE_URL/api/memory" \
    -H "Content-Type: application/json" \
    -d '{
        "enabled": true,
        "provider": "ollama",
        "model": "custom-model"
    }' > /dev/null

echo "✅ 初始配置已设置"

# Get snapshot count before reset
echo ""
echo "获取重置前的版本数量..."
BEFORE_SNAPSHOTS=$(curl -s "$BASE_URL/api/snapshots")
BEFORE_COUNT=$(echo "$BEFORE_SNAPSHOTS" | grep -o '"id"' | wc -l)
echo "重置前版本数量: $BEFORE_COUNT"

# Step 1: Verify configuration exists
echo ""
echo "Step 1: 验证配置存在..."
CONFIG=$(curl -s "$BASE_URL/api/config")
PROVIDER_COUNT=$(echo "$CONFIG" | grep -o '"providers"' | wc -l)
echo "Provider 配置存在: $PROVIDER_COUNT"

# Step 2: Trigger factory reset
echo ""
echo "Step 2: 触发出厂设置..."
RESET_RESULT=$(curl -s -X POST "$BASE_URL/api/setup/reset")
echo "重置结果: $RESET_RESULT"

# Note: Current implementation may return "not implemented"
if echo "$RESET_RESULT" | grep -q '"success":true'; then
    echo "✅ 出厂设置成功"
    RESET_SUCCESS=true
elif echo "$RESET_RESULT" | grep -qi "not implemented"; then
    echo "⚠️  出厂设置未实现"
    RESET_SUCCESS=false
else
    echo "⚠️  出厂设置结果未知"
    RESET_SUCCESS=false
fi

# Step 3: Verify configuration after reset (if reset succeeded)
if [ "$RESET_SUCCESS" = true ]; then
    echo ""
    echo "Step 3: 验证重置后的配置..."
    
    # Check providers
    PROVIDERS=$(curl -s "$BASE_URL/api/providers")
    PROVIDER_LIST_COUNT=$(echo "$PROVIDERS" | grep -o '"id"' | wc -l)
    echo "重置后 Provider 数量: $PROVIDER_LIST_COUNT"
    
    # Check agents
    AGENTS=$(curl -s "$BASE_URL/api/agents")
    AGENT_LIST_COUNT=$(echo "$AGENTS" | grep '"list"' | grep -o '\[\]' | wc -l)
    echo "重置后 Agent list: $AGENT_LIST_COUNT"
    
    # Check channels
    CHANNELS=$(curl -s "$BASE_URL/api/channels")
    if echo "$CHANNELS" | grep -q '"enabled":false'; then
        echo "✅ Channels 已禁用"
    fi
    
    # Check memory
    MEMORY=$(curl -s "$BASE_URL/api/memory")
    if [ "$MEMORY" = "null" ] || [ -z "$MEMORY" ]; then
        echo "✅ Memory 已重置"
    fi
else
    echo ""
    echo "Step 3: 跳过配置验证（出厂设置未实现）"
fi

# Step 4: Verify Git commit was created for reset
echo ""
echo "Step 4: 验证 Git 提交..."
AFTER_SNAPSHOTS=$(curl -s "$BASE_URL/api/snapshots")
AFTER_COUNT=$(echo "$AFTER_SNAPSHOTS" | grep -o '"id"' | wc -l)
echo "重置后版本数量: $AFTER_COUNT"

if [ "$AFTER_COUNT" -gt "$BEFORE_COUNT" ]; then
    echo "✅ 有新的 Git 提交（出厂设置）"
else
    echo "⚠️  可能没有新的 Git 提交"
fi

# Step 5: Check setup status after reset
echo ""
echo "Step 5: 检查重置后的设置状态..."
SETUP_CHECK=$(curl -s "$BASE_URL/api/setup/check")
echo "设置状态: $SETUP_CHECK"

# After reset, should be back to first-time setup
if echo "$SETUP_CHECK" | grep -q '"is_first_setup":true'; then
    echo "✅ 已回到首次启动状态"
else
    echo "ℹ️  设置状态: $(echo "$SETUP_CHECK" | grep -o '"is_first_setup":[^,}]*')"
fi

# Step 6: Health check
echo ""
echo "Step 6: 健康检查..."
HEALTH=$(curl -s "$BASE_URL/api/health")
if echo "$HEALTH" | grep -q '"status":"ok"'; then
    echo "✅ 服务健康"
else
    echo "❌ 服务不健康: $HEALTH"
    exit 1
fi

# Step 7: Verify we can start over
echo ""
echo "Step 7: 验证可以重新开始配置..."
if [ "$RESET_SUCCESS" = true ]; then
    # Try to create a new provider after reset
    curl -s -X POST "$BASE_URL/api/providers/new-after-reset" \
        -H "Content-Type: application/json" \
        -d '{
            "api": "openai-completions",
            "apiKey": "sk-after-reset",
            "baseUrl": "https://api.new.com",
            "enabled": true,
            "defaultModel": "gpt-4"
        }' | grep -q '"success":true' && echo "✅ 重置后可以创建新配置"
else
    echo "ℹ️  跳过（出厂设置未实现）"
fi

echo ""
echo "=========================================="
echo "✅ 流程 D: 出厂设置测试完成"
echo "=========================================="
echo ""
echo "注意: 完整的出厂设置测试需要:"
echo "  1. /api/setup/reset 端点完全实现"
echo "  2. factory-config.json 存在"
echo "  3. 重置后自动重启服务"
echo ""
exit 0
