#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR/.."

echo "=== Stopping E2E Test Environment ==="
docker compose -f docker-compose.test.yml down -v

echo "=== Cleanup complete ==="
