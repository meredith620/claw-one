#!/bin/bash
set -e

BASE_URL="http://localhost:28080"

echo "Testing health endpoint..."
RESPONSE=$(curl -s "$BASE_URL/api/health")
if echo "$RESPONSE" | grep -q "ok"; then
    echo "✅ Health check passed"
    exit 0
else
    echo "❌ Health check failed: $RESPONSE"
    exit 1
fi
