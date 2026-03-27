#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR/.."

echo "=== Starting E2E Test Environment ==="

# Check if already running
if docker ps | grep -q "claw-one-test"; then
    echo "Test environment already running"
    exit 0
fi

# Start services
docker-compose -f docker-compose.test.yml up -d --build

echo "=== Waiting for services to be ready ==="
sleep 5

# Wait for claw-one health
echo "Checking claw-one health..."
for i in {1..30}; do
    if curl -s http://localhost:28080/api/health | grep -q "ok"; then
        echo "claw-one is ready!"
        break
    fi
    echo "Waiting... ($i/30)"
    sleep 2
done

echo "=== Test environment ready ==="
echo "claw-one: http://localhost:28080"
echo "openclaw: http://localhost:28789"
