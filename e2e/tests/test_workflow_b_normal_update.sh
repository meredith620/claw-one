#!/bin/bash
set -e

BASE_URL="http://localhost:28080"

# Reset environment first
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
bash "$SCRIPT_DIR/../scripts/test-env-reset.sh"

echo "=========================================="
echo "流程 B: 正常配置更新测试"
echo "=========================================="

# Pre-condition: Setup initial configuration
echo ""
echo "Pre-condition: 设置初始配置..."
curl -s -X POST "$BASE_URL/api/providers/initial" \
    -H "Content-Type: application/json" \
    -d '{
        "api": "openai-completions",
        "apiKey": "sk-initial",
        "baseUrl": "https://api.initial.com",
        "enabled": true,
        "defaultModel": "gpt-4"
    }' > /dev/null

curl -s -X POST "$BASE_URL/api/setup/complete" > /dev/null

# Get initial snapshot count
echo ""
echo "获取初始快照数量..."
INITIAL_SNAPSHOTS=$(curl -s "$BASE_URL/api/snapshots")
INITIAL_COUNT=$(echo "$INITIAL_SNAPSHOTS" | grep -o '"id"' | wc -l)
echo "初始快照数量: $INITIAL_COUNT"

# Step 1: Add a new provider
echo ""
echo "Step 1: 添加新 Provider..."
curl -s -X POST "$BASE_URL/api/providers/new-provider" \
    -H "Content-Type: application/json" \
    -d '{
        "api": "anthropic-messages",
        "apiKey": "sk-new-provider",
        "baseUrl": "https://api.anthropic.com",
        "enabled": true,
        "defaultModel": "claude-3-opus"
    }' | grep -q '"success":true' || { echo "❌ 添加 Provider 失败"; exit 1; }
echo "✅ 新 Provider 添加成功"

# Step 2: Update model priority
echo ""
echo "Step 2: 更新模型优先级..."
curl -s -X POST "$BASE_URL/api/model-priority" \
    -H "Content-Type: application/json" \
    -d '{
        "primary": "new-provider/claude-3-opus",
        "fallbacks": ["initial/gpt-4"]
    }' | grep -q '"success":true' || { echo "❌ 更新模型优先级失败"; exit 1; }
echo "✅ 模型优先级更新成功"

# Step 3: Verify configuration changes
echo ""
echo "Step 3: 验证配置变更..."
CONFIG=$(curl -s "$BASE_URL/api/config")

if echo "$CONFIG" | grep -q "new-provider"; then
    echo "✅ 新 Provider 在配置中"
else
    echo "❌ 新 Provider 不在配置中"
    exit 1
fi

# Step 4: Check state (should be Normal or ApplyingConfig)
echo ""
echo "Step 4: 检查服务状态..."
STATE=$(curl -s "$BASE_URL/api/state")
echo "当前状态: $STATE"

# Step 5: Verify Git commit was created
echo ""
echo "Step 5: 验证 Git 提交..."
UPDATED_SNAPSHOTS=$(curl -s "$BASE_URL/api/snapshots")
UPDATED_COUNT=$(echo "$UPDATED_SNAPSHOTS" | grep -o '"id"' | wc -l)
echo "更新后快照数量: $UPDATED_COUNT"

if [ "$UPDATED_COUNT" -gt "$INITIAL_COUNT" ]; then
    echo "✅ 有新的 Git 提交"
else
    echo "⚠️  可能没有新的 Git 提交（如果配置无变更）"
fi

# Step 6: Verify providers list
echo ""
echo "Step 6: 验证 Provider 列表..."
PROVIDERS=$(curl -s "$BASE_URL/api/providers")
PROVIDER_COUNT=$(echo "$PROVIDERS" | grep -o '"id"' | wc -l)
echo "Provider 数量: $PROVIDER_COUNT"

if [ "$PROVIDER_COUNT" -ge 2 ]; then
    echo "✅ 有多个 Provider"
else
    echo "⚠️  Provider 数量较少: $PROVIDER_COUNT"
fi

# Step 7: Update agent configuration
echo ""
echo "Step 7: 更新 Agent 配置..."
curl -s -X POST "$BASE_URL/api/agents" \
    -H "Content-Type: application/json" \
    -d '{
        "defaults": {
            "workspace": "/updated/workspace",
            "compaction": {"mode": "safeguard"}
        },
        "list": [
            {
                "id": "main-agent",
                "name": "Main Agent",
                "workspace": "/workspace/main"
            }
        ]
    }' | grep -q '"success":true' || { echo "❌ 更新 Agent 配置失败"; exit 1; }
echo "✅ Agent 配置更新成功"

# Step 8: Verify agent changes
echo ""
echo "Step 8: 验证 Agent 变更..."
AGENTS=$(curl -s "$BASE_URL/api/agents")
if echo "$AGENTS" | grep -q "/updated/workspace"; then
    echo "✅ Agent workspace 已更新"
else
    echo "❌ Agent workspace 未更新"
    exit 1
fi

if echo "$AGENTS" | grep -q "main-agent"; then
    echo "✅ Agent list 已更新"
else
    echo "❌ Agent list 未更新"
    exit 1
fi

# Step 9: Test memory configuration
echo ""
echo "Step 9: 配置 Memory..."
curl -s -X POST "$BASE_URL/api/memory" \
    -H "Content-Type: application/json" \
    -d '{
        "enabled": true,
        "provider": "ollama",
        "model": "qwen3-embedding:0.6b",
        "remote": {
            "baseUrl": "http://localhost:11434"
        }
    }' | grep -q '"success":true' || { echo "❌ Memory 配置失败"; exit 1; }
echo "✅ Memory 配置成功"

# Step 10: Verify memory configuration
echo ""
echo "Step 10: 验证 Memory 配置..."
MEMORY=$(curl -s "$BASE_URL/api/memory")
if echo "$MEMORY" | grep -q "qwen3-embedding"; then
    echo "✅ Memory 配置已保存"
else
    echo "❌ Memory 配置未保存"
    exit 1
fi

# Step 11: Final state check
echo ""
echo "Step 11: 最终状态检查..."
FINAL_STATE=$(curl -s "$BASE_URL/api/state")
echo "最终状态: $FINAL_STATE"

# Verify health
echo ""
echo "健康检查..."
HEALTH=$(curl -s "$BASE_URL/api/health")
if echo "$HEALTH" | grep -q '"status":"ok"'; then
    echo "✅ 服务健康"
else
    echo "❌ 服务不健康: $HEALTH"
    exit 1
fi

echo ""
echo "=========================================="
echo "✅ 流程 B: 正常配置更新测试通过"
echo "=========================================="
exit 0
