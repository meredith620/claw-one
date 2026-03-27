#!/bin/bash
set -e

BASE_URL="http://localhost:28080"

# Reset environment first
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
bash "$SCRIPT_DIR/../scripts/test-env-reset.sh"

echo "1. Getting initial agents config..."
INITIAL_CONFIG=$(curl -s "$BASE_URL/api/agents")
echo "Initial config: $INITIAL_CONFIG"

# Verify we have the expected structure
echo "$INITIAL_CONFIG" | grep -q '"defaults"' || { echo "Missing defaults field"; exit 1; }
echo "$INITIAL_CONFIG" | grep -q '"list"' || { echo "Missing list field"; exit 1; }

echo "2. Creating agent by saving complete config..."
# Agents API uses module-level save, not per-entity
# Structure: { defaults: {...}, list: [...] }
curl -s -X POST "$BASE_URL/api/agents" \
    -H "Content-Type: application/json" \
    -d '{
        "defaults": {
            "workspace": "~/.openclaw/workspace",
            "compaction": {"mode": "safeguard"}
        },
        "list": [
            {
                "id": "test-agent",
                "name": "Test Agent",
                "model": "gpt-4",
                "provider": "test-provider",
                "systemPrompt": "You are a test agent"
            }
        ]
    }' | grep -q '"success":true' || { echo "Create failed"; exit 1; }

echo "3. Verifying agent was saved..."
SAVED_CONFIG=$(curl -s "$BASE_URL/api/agents")
echo "$SAVED_CONFIG" | grep -q '"id":"test-agent"' || { echo "Agent not found in saved config"; exit 1; }
echo "$SAVED_CONFIG" | grep -q '"name":"Test Agent"' || { echo "Agent name not found"; exit 1; }

echo "4. Verifying openclaw.json..."
docker exec claw-one-test-openclaw cat /home/node/.openclaw/openclaw.json | grep -q "test-agent" || { echo "Agent not in config"; exit 1; }

echo "5. Updating agent..."
# Update by saving complete config with modified agent
curl -s -X POST "$BASE_URL/api/agents" \
    -H "Content-Type: application/json" \
    -d '{
        "defaults": {
            "workspace": "~/.openclaw/workspace",
            "compaction": {"mode": "safeguard"}
        },
        "list": [
            {
                "id": "test-agent",
                "name": "Updated Agent",
                "model": "gpt-4",
                "provider": "test-provider",
                "systemPrompt": "You are an updated test agent"
            }
        ]
    }' | grep -q '"success":true' || { echo "Update failed"; exit 1; }

echo "6. Verifying update..."
UPDATED_CONFIG=$(curl -s "$BASE_URL/api/agents")
echo "$UPDATED_CONFIG" | grep -q '"name":"Updated Agent"' || { echo "Updated name not found"; exit 1; }

echo "7. Deleting agent by saving empty list..."
curl -s -X POST "$BASE_URL/api/agents" \
    -H "Content-Type: application/json" \
    -d '{
        "defaults": {
            "workspace": "~/.openclaw/workspace",
            "compaction": {"mode": "safeguard"}
        },
        "list": []
    }' | grep -q '"success":true' || { echo "Delete failed"; exit 1; }

echo "8. Verifying deletion..."
FINAL_CONFIG=$(curl -s "$BASE_URL/api/agents")
! echo "$FINAL_CONFIG" | grep -q '"id":"test-agent"' || { echo "Agent still exists"; exit 1; }
! docker exec claw-one-test-openclaw cat /home/node/.openclaw/openclaw.json | grep -q "test-agent" || { echo "Agent still in config"; exit 1; }

echo "✅ Agent CRUD test passed"
exit 0
