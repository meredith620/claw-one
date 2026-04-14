#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR/.."

echo "=== Resetting test environment ==="

# Check if OpenClaw container exists
if ! docker ps -a --filter "name=claw-one-test-openclaw" --format "{{.Names}}" | grep -q "claw-one-test-openclaw"; then
    echo "OpenClaw container not found, starting..."
    docker compose -f docker-compose.test.yml up -d openclaw
    sleep 5
fi

# Get current container ID to detect if it changes
OPENCLAW_CONTAINER_ID=$(docker ps --filter "name=claw-one-test-openclaw" --format "{{.ID}}" 2>/dev/null || echo "")

# Copy baseline config to OpenClaw container
echo "Copying baseline config..."
docker cp fixtures/baseline-openclaw.json claw-one-test-openclaw:/root/.openclaw/openclaw.json

# Wait for OpenClaw to detect config change and restart (if running)
if [ -n "$OPENCLAW_CONTAINER_ID" ]; then
    echo "Waiting for OpenClaw to process config change..."
    sleep 8
    
    # Check if container is still running
    NEW_CONTAINER_ID=$(docker ps --filter "name=claw-one-test-openclaw" --format "{{.ID}}" 2>/dev/null || echo "")
    if [ -z "$NEW_CONTAINER_ID" ]; then
        echo "OpenClaw container stopped, waiting for restart..."
        # With restart: always, container should restart automatically
        for i in {1..20}; do
            sleep 2
            NEW_CONTAINER_ID=$(docker ps --filter "name=claw-one-test-openclaw" --format "{{.ID}}" 2>/dev/null || echo "")
            if [ -n "$NEW_CONTAINER_ID" ]; then
                echo "OpenClaw restarted successfully"
                break
            fi
            echo "  Waiting for OpenClaw restart... ($i/20)"
        done
    fi
fi

# Wait for OpenClaw to be healthy
echo "Waiting for OpenClaw to be healthy..."
for i in {1..20}; do
    if docker exec claw-one-test-openclaw curl -sf http://localhost:18790/ > /dev/null 2>&1; then
        echo "OpenClaw is healthy"
        break
    fi
    echo "  Waiting for OpenClaw... ($i/20)"
    sleep 2
done

# Reset claw-one git repo
echo "Resetting claw-one git repo..."
docker exec claw-one-test-app bash -c '
    rm -rf /app/data/version-config
    mkdir -p /app/data/version-config
    cd /app/data/version-config
    git init
    git config user.name "Test"
    git config user.email "test@test.com"
' 2>/dev/null || {
    echo "claw-one container not ready, waiting..."
    sleep 5
    docker exec claw-one-test-app bash -c '
        rm -rf /app/data/version-config
        mkdir -p /app/data/version-config
        cd /app/data/version-config
        git init
        git config user.name "Test"
        git config user.email "test@test.com"
    '
}

# Restart claw-one container to refresh ConfigManager cache
echo "Restarting claw-one container..."
docker restart claw-one-test-app 2>/dev/null || docker compose -f docker-compose.test.yml restart claw-one

# Wait for claw-one to be ready
echo "Waiting for claw-one to be ready..."
for i in {1..15}; do
    if curl -sf http://localhost:28080/api/health > /dev/null 2>&1; then
        echo "claw-one is ready"
        break
    fi
    echo "  Waiting for claw-one... ($i/15)"
    sleep 2
done

echo "=== Reset complete ==="