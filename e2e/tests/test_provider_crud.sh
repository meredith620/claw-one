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
docker exec claw-one-test-openclaw cat /home/node/.openclaw/openclaw.json | grep -q "test-provider" || { echo "Provider not in config"; exit 1; }

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
! docker exec claw-one-test-openclaw cat /home/node/.openclaw/openclaw.json | grep -q "test-provider" || { echo "Provider still in config"; exit 1; }

echo "✅ Provider CRUD test passed"
exit 0
