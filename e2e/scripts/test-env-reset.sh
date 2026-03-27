#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR/.."

echo "=== Resetting test environment ==="

# Reset openclaw config to baseline
docker cp fixtures/baseline-openclaw.json claw-one-test-openclaw:/home/node/.openclaw/openclaw.json

# Reset claw-one git repo
docker exec claw-one-test-app bash -c '
    rm -rf /app/data/version-config
    mkdir -p /app/data/version-config
    cd /app/data/version-config
    git init
    git config user.name "Test"
    git config user.email "test@test.com"
'

# Restart claw-one container to refresh ConfigManager cache
echo "Restarting claw-one container..."
docker restart claw-one-test-app

echo "=== Reset complete ==="
