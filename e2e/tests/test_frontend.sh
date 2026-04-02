#!/bin/bash
set -e

BASE_URL="http://localhost:28080"

echo "Testing frontend/API endpoints..."

echo "1. Testing API health endpoint..."
RESPONSE=$(curl -s "$BASE_URL/api/health")
HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" "$BASE_URL/api/health")
if [ "$HTTP_CODE" = "200" ]; then
    if echo "$RESPONSE" | grep -q '"status".*"ok"'; then
        echo "✅ API health endpoint returns correct JSON"
    else
        echo "❌ API health endpoint returned 200 but unexpected content"
        echo "Response: $RESPONSE"
        exit 1
    fi
else
    echo "❌ API health endpoint returned HTTP $HTTP_CODE"
    exit 1
fi

echo "2. Testing index page (optional for API-only service)..."
HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" "$BASE_URL/" || echo "000")
if [ "$HTTP_CODE" = "200" ]; then
    echo "✅ Index page accessible"
elif [ "$HTTP_CODE" = "404" ]; then
    echo "ℹ️  Index page returns 404 (API-only service, this is OK)"
else
    echo "⚠️  Index page returned HTTP $HTTP_CODE"
fi

echo "✅ Frontend/API test passed"
exit 0
