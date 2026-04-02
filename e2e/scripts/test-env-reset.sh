#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR/.."

echo "=== Resetting test environment ==="

# Reset openclaw config to baseline
docker cp fixtures/baseline-openclaw.json claw-one-test-openclaw:/root/.openclaw/openclaw.json

# Wait for OpenClaw to restart and become healthy
echo "Waiting for OpenClaw to restart..."
sleep 3
for i in {1..10}; do
    if docker exec claw-one-test-openclaw curl -sf http://localhost:18789/healthz > /dev/null 2>&1; then
        echo "OpenClaw is healthy"
        break
    fi
    echo "  Waiting for OpenClaw... ($i/10)"
    sleep 2
done

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

# Wait for claw-one to be ready
echo "Waiting for claw-one to be ready..."
for i in {1..10}; do
    if curl -sf http://localhost:28080/api/health > /dev/null 2>&1; then
        echo "claw-one is ready"
        break
    fi
    echo "  Waiting for claw-one... ($i/10)"
    sleep 2
done

echo "=== Reset complete ==="
