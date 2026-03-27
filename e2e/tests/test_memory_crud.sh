#!/bin/bash
set -e

BASE_URL="http://localhost:28080"

# Reset environment first
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
bash "$SCRIPT_DIR/../scripts/test-env-reset.sh"

echo "1. Getting initial memory config (should be null or empty)..."
INITIAL_CONFIG=$(curl -s "$BASE_URL/api/memory")
echo "Initial config: $INITIAL_CONFIG"

echo "2. Creating memory config..."
# Memory API uses module-level save
# Structure: { enabled, provider, remote, model, ... }
curl -s -X POST "$BASE_URL/api/memory" \
    -H "Content-Type: application/json" \
    -d '{
        "enabled": true,
        "provider": "ollama",
        "remote": {
            "baseUrl": "http://localhost:11434"
        },
        "model": "test-memory-model",
        "fallback": "none",
        "sources": ["memory", "sessions"]
    }' | grep -q '"success":true' || { echo "Create failed"; exit 1; }

echo "3. Verifying memory config was saved..."
SAVED_CONFIG=$(curl -s "$BASE_URL/api/memory")
echo "$SAVED_CONFIG" | grep -q '"enabled":true' || { echo "Memory enabled not found"; exit 1; }
echo "$SAVED_CONFIG" | grep -q '"provider":"ollama"' || { echo "Memory provider not found"; exit 1; }
echo "$SAVED_CONFIG" | grep -q '"model":"test-memory-model"' || { echo "Memory model not found"; exit 1; }

echo "4. Verifying openclaw.json..."
docker exec claw-one-test-openclaw cat /home/node/.openclaw/openclaw.json | grep -q "test-memory-model" || { echo "Memory not in config"; exit 1; }

echo "5. Updating memory config..."
curl -s -X POST "$BASE_URL/api/memory" \
    -H "Content-Type: application/json" \
    -d '{
        "enabled": true,
        "provider": "ollama",
        "remote": {
            "baseUrl": "http://localhost:11434"
        },
        "model": "updated-memory-model",
        "fallback": "none",
        "sources": ["memory", "sessions"]
    }' | grep -q '"success":true' || { echo "Update failed"; exit 1; }

echo "6. Verifying update..."
UPDATED_CONFIG=$(curl -s "$BASE_URL/api/memory")
echo "$UPDATED_CONFIG" | grep -q '"model":"updated-memory-model"' || { echo "Updated model not found"; exit 1; }

echo "7. Deleting memory config (saving null)..."
curl -s -X POST "$BASE_URL/api/memory" \
    -H "Content-Type: application/json" \
    -d 'null' | grep -q '"success":true' || { echo "Delete failed"; exit 1; }

echo "8. Verifying deletion..."
FINAL_CONFIG=$(curl -s "$BASE_URL/api/memory")
# After saving null, get_memory returns null
[ "$FINAL_CONFIG" = "null" ] || { echo "Memory config should be null after deletion"; exit 1; }

echo "✅ Memory CRUD test passed"
exit 0
