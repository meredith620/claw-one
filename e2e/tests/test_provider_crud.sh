#!/bin/bash
set -e

BASE_URL="http://localhost:28080"

# Reset environment first
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
bash "$SCRIPT_DIR/../scripts/test-env-reset.sh"

echo "1. Creating provider..."
curl -s -X POST "$BASE_URL/api/providers/test-provider" \
    -H "Content-Type: application/json" \
    -d '{
        "api": "openai-responses",
        "apiKey": "sk-test",
        "baseUrl": "https://api.test.com",
        "enabled": true,
        "defaultModel": "gpt-4"
    }' | grep -q '"success":true' || { echo "Create failed"; exit 1; }

echo "2. Verifying openclaw.json..."
docker exec claw-one-test-openclaw cat /root/.openclaw/openclaw.json | grep -q "test-provider" || { echo "Provider not in config"; exit 1; }

echo "3. Getting provider..."
curl -s "$BASE_URL/api/providers/test-provider" | grep -q '"id":"test-provider"' || { echo "Get failed"; exit 1; }

echo "4. Updating provider..."
curl -s -X POST "$BASE_URL/api/providers/test-provider" \
    -H "Content-Type: application/json" \
    -d '{"apiKey":"sk-updated"}' | grep -q '"success":true' || { echo "Update failed"; exit 1; }

echo "5. Listing providers..."
curl -s "$BASE_URL/api/providers" | grep -q '"test-provider"' || { echo "List failed"; exit 1; }

echo "6. Deleting provider..."
curl -s -X DELETE "$BASE_URL/api/providers/test-provider" | grep -q '"success":true' || { echo "Delete failed"; exit 1; }

echo "7. Verifying deletion..."
# Wait for OpenClaw to restart after config change
sleep 5
# Wait for OpenClaw to be healthy again
for i in {1..10}; do
    if docker exec claw-one-test-openclaw curl -sf http://localhost:18789/healthz > /dev/null 2>&1; then
        break
    fi
    echo "  Waiting for OpenClaw restart... ($i/10)"
    sleep 2
done
# Check config - only check in models.providers, not in agents.defaults
if docker exec claw-one-test-openclaw cat /root/.openclaw/openclaw.json 2>/dev/null | python3 -c "import json,sys; d=json.load(sys.stdin); providers=d.get('models',{}).get('providers',{}); sys.exit(0 if 'test-provider' in providers else 1)"; then
    echo "Provider still in config"
    exit 1
fi

echo "✅ Provider CRUD test passed"
exit 0
