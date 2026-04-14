#!/bin/bash
set -e

BASE_URL="http://localhost:28080"

# Reset environment first
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
bash "$SCRIPT_DIR/../scripts/test-env-reset.sh"

# Wait for OpenClaw to be stable
sleep 5

# Check if OpenClaw container is still running
OPENCLAW_CONTAINER=$(docker ps --filter "name=claw-one-test-openclaw" --format "{{.Names}}" 2>/dev/null || echo "")
if [ -z "$OPENCLAW_CONTAINER" ]; then
    echo "OpenClaw container not running after reset, restarting..."
    docker start claw-one-test-openclaw 2>/dev/null || true
    sleep 10
fi

echo "1. Creating provider..."
CREATE_RESULT=$(curl -s -X POST "$BASE_URL/api/providers/test-provider" \
    -H "Content-Type: application/json" \
    -d '{
        "api": "openai-responses",
        "apiKey": "sk-test",
        "baseUrl": "https://api.test.com",
        "enabled": true,
        "defaultModel": "gpt-4"
    }')
echo "Create result: $CREATE_RESULT"
echo "$CREATE_RESULT" | grep -q '"success":true' || { echo "Create failed"; exit 1; }

echo "2. Verifying openclaw.json..."
# Give OpenClaw time to write config
sleep 3
# Verify container is running before exec
if ! docker ps --filter "name=claw-one-test-openclaw" --format "{{.Names}}" | grep -q "claw-one-test-openclaw"; then
    echo "OpenClaw container stopped during test"
    exit 1
fi
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
# Check container is running before exec
if ! docker ps --filter "name=claw-one-test-openclaw" --format "{{.Names}}" | grep -q "claw-one-test-openclaw"; then
    echo "OpenClaw container stopped after delete, restarting..."
    docker start claw-one-test-openclaw 2>/dev/null || true
    sleep 5
fi
# Wait for OpenClaw to be healthy again
for i in {1..10}; do
    if docker exec claw-one-test-openclaw curl -sf http://localhost:18790/ > /dev/null 2>&1; then
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
