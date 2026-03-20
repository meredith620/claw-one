#!/bin/bash
# 验证 save_module_handler deep merge 修复
set -e

TEST_DIR="/tmp/merge-test-$$"
OPENCLAW_HOME="$TEST_DIR/.openclaw"
mkdir -p "$OPENCLAW_HOME" "$TEST_DIR/config" "$TEST_DIR/logs"

# 初始配置：tools 下已有 elevated 和 exec
cat > "$OPENCLAW_HOME/openclaw.json" << 'EOF'
{
  "agents": {
    "defaults": {
      "model": { "primary": "redfrog2/kimi-for-coding" },
      "models": { "redfrog2/kimi-for-coding": { "alias": "Kimi" } }
    }
  },
  "tools": {
    "elevated": { "enabled": true, "allowFrom": { "mattermost": ["user1"] } },
    "exec": { "sandbox": "off" },
    "web": { "enabled": true }
  },
  "models": {
    "providers": {
      "redfrog2": {
        "api": "anthropic-messages",
        "apiKey": "test-key",
        "baseUrl": "https://llm.redfrog.cloud",
        "models": [{ "id": "kimi-for-coding", "contextWindow": 262144, "maxTokens": 32768, "name": "Kimi", "reasoning": true, "input": ["text"] }]
      }
    }
  },
  "gateway": { "port": 18790 }
}
EOF

cat > "$TEST_DIR/config/claw-one.toml" << EOF
[server]
host = "0.0.0.0"
port = 18080
log_level = "info"

[openclaw]
openclaw_home = "$OPENCLAW_HOME"
service_name = "openclaw"
health_port = 18790
health_timeout = 30
EOF

CLAW_ONE_BIN="/workspace/hull/target/release/claw-one"
export CLAW_ONE_CONFIG="$TEST_DIR/config/claw-one.toml"
export CLAW_ONE_LOG_DIR="$TEST_DIR/logs"

$CLAW_ONE_BIN run &
SERVER_PID=$!
sleep 2

echo "=== 初始 tools 配置 ==="
curl -s http://localhost:18080/api/config/tools | python3 -m json.tool

echo ""
echo "=== 保存 A2A 配置（只传 agentToAgent + sessions）==="
curl -s -X POST -H "Content-Type: application/json" \
  -d '{"agentToAgent":{"enabled":true,"allow":["a1","a2"]},"sessions":{"visibility":"all"}}' \
  http://localhost:18080/api/config/tools

echo ""
echo ""
echo "=== 保存后 tools 配置 ==="
curl -s http://localhost:18080/api/config/tools | python3 -m json.tool

echo ""
echo "=== 验证 ==="
python3 -c "
import json, urllib.request
data = json.load(urllib.request.urlopen('http://localhost:18080/api/config/tools'))
checks = {
    'elevated 保留': 'elevated' in data,
    'exec 保留': 'exec' in data,
    'web 保留': 'web' in data,
    'agentToAgent 写入': data.get('agentToAgent', {}).get('enabled') == True,
    'sessions 写入': data.get('sessions', {}).get('visibility') == 'all',
}
all_pass = True
for name, ok in checks.items():
    status = '✅' if ok else '❌'
    if not ok: all_pass = False
    print(f'  {status} {name}')
print()
print('🎉 全部通过！' if all_pass else '💥 有失败项！')
"

kill $SERVER_PID 2>/dev/null
rm -rf "$TEST_DIR"
