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
                "model": "gpt-4"
            }
        ]
    }' | grep -q '"success":true' || { echo "Create failed"; exit 1; }

echo "3. Verifying agent was saved..."
sleep 3
SAVED_CONFIG=$(curl -s "$BASE_URL/api/agents")
echo "$SAVED_CONFIG" | grep -q '"id":"test-agent"' || { echo "Agent not found in saved config"; exit 1; }
echo "$SAVED_CONFIG" | grep -q '"name":"Test Agent"' || { echo "Agent name not found"; exit 1; }

echo "4. Verifying openclaw.json..."
if ! docker ps --filter "name=claw-one-test-openclaw" --format "{{.Names}}" | grep -q "claw-one-test-openclaw"; then
    echo "OpenClaw container stopped during test"
    exit 1
fi
docker exec claw-one-test-openclaw cat /root/.openclaw/openclaw.json | grep -q "test-agent" || { echo "Agent not in config"; exit 1; }

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
                "model": "gpt-4"
            }
        ]
    }' | grep -q '"success":true' || { echo "Update failed"; exit 1; }

echo "6. Verifying update..."
sleep 3
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
sleep 5
# Check container is running before exec
if ! docker ps --filter "name=claw-one-test-openclaw" --format "{{.Names}}" | grep -q "claw-one-test-openclaw"; then
    echo "OpenClaw container stopped after delete, restarting..."
    docker start claw-one-test-openclaw 2>/dev/null || true
    sleep 5
fi
FINAL_CONFIG=$(curl -s "$BASE_URL/api/agents")
! echo "$FINAL_CONFIG" | grep -q '"id":"test-agent"' || { echo "Agent still exists"; exit 1; }
! docker exec claw-one-test-openclaw cat /root/.openclaw/openclaw.json 2>/dev/null | grep -q "test-agent" || { echo "Agent still in config"; exit 1; }

echo "✅ Agent CRUD test passed"
exit 0