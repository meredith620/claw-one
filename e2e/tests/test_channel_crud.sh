#!/bin/bash
set -e

BASE_URL="http://localhost:28080"

# Reset environment first
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
bash "$SCRIPT_DIR/../scripts/test-env-reset.sh"

echo "1. Getting initial channels config..."
INITIAL_CONFIG=$(curl -s "$BASE_URL/api/channels")
echo "Initial config: $INITIAL_CONFIG"

echo "2. Creating channel by saving complete config..."
# Channels API uses module-level save
# Structure: { mattermost: { enabled, dmPolicy, accounts: {...} }, feishu: {...} }
curl -s -X POST "$BASE_URL/api/channels" \
    -H "Content-Type: application/json" \
    -d '{
        "mattermost": {
            "enabled": true,
            "dmPolicy": "allow",
            "groupPolicy": "allowlist",
            "accounts": {
                "test-channel": {
                    "name": "Test Channel",
                    "botToken": "test-token",
                    "baseUrl": "https://mattermost.test.com"
                }
            }
        }
    }' | grep -q '"success":true' || { echo "Create failed"; exit 1; }

echo "3. Verifying channel was saved..."
SAVED_CONFIG=$(curl -s "$BASE_URL/api/channels")
echo "$SAVED_CONFIG" | grep -q '"test-channel"' || { echo "Channel not found in saved config"; exit 1; }
echo "$SAVED_CONFIG" | grep -q '"name":"Test Channel"' || { echo "Channel name not found"; exit 1; }

echo "4. Verifying openclaw.json..."
docker exec claw-one-test-openclaw cat /root/.openclaw/openclaw.json | grep -q "test-channel" || { echo "Channel not in config"; exit 1; }

echo "5. Updating channel..."
# Update by saving complete config with modified channel
curl -s -X POST "$BASE_URL/api/channels" \
    -H "Content-Type: application/json" \
    -d '{
        "mattermost": {
            "enabled": true,
            "dmPolicy": "allow",
            "groupPolicy": "allowlist",
            "accounts": {
                "test-channel": {
                    "name": "Updated Channel",
                    "botToken": "updated-token",
                    "baseUrl": "https://mattermost.updated.com"
                }
            }
        }
    }' | grep -q '"success":true' || { echo "Update failed"; exit 1; }

echo "6. Verifying update..."
UPDATED_CONFIG=$(curl -s "$BASE_URL/api/channels")
echo "$UPDATED_CONFIG" | grep -q '"name":"Updated Channel"' || { echo "Updated name not found"; exit 1; }
echo "$UPDATED_CONFIG" | grep -q '"botToken":"updated-token"' || { echo "Updated token not found"; exit 1; }

echo "7. Deleting channel by saving empty accounts..."
curl -s -X POST "$BASE_URL/api/channels" \
    -H "Content-Type: application/json" \
    -d '{
        "mattermost": {
            "enabled": false,
            "dmPolicy": "pairing",
            "groupPolicy": "allowlist",
            "accounts": {}
        }
    }' | grep -q '"success":true' || { echo "Delete failed"; exit 1; }

echo "8. Verifying deletion..."
FINAL_CONFIG=$(curl -s "$BASE_URL/api/channels")
! echo "$FINAL_CONFIG" | grep -q '"test-channel"' || { echo "Channel still exists"; exit 1; }
! docker exec claw-one-test-openclaw cat /root/.openclaw/openclaw.json | grep -q "test-channel" || { echo "Channel still in config"; exit 1; }

echo "✅ Channel CRUD test passed"
exit 0
