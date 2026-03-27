#!/bin/bash
set -e

BASE_URL="http://localhost:28080"

echo "Testing frontend endpoints..."

echo "1. Testing index page..."
RESPONSE=$(curl -s "$BASE_URL/")
HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" "$BASE_URL/")
if [ "$HTTP_CODE" = "200" ]; then
    # Check for actual HTML content, not just any response
    if echo "$RESPONSE" | grep -qi "<!doctype html\|<html\|<body"; then
        echo "✅ Index page accessible and returns valid HTML"
    else
        echo "❌ Index page returned 200 but not valid HTML"
        echo "Response preview: $(echo "$RESPONSE" | head -c 200)"
        exit 1
    fi
else
    echo "❌ Index page returned HTTP $HTTP_CODE"
    exit 1
fi

echo "2. Testing API health endpoint..."
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

echo "3. Testing static assets directory..."
HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" "$BASE_URL/assets/")
if [ "$HTTP_CODE" = "200" ] || [ "$HTTP_CODE" = "404" ]; then
    echo "✅ Static assets endpoint responds (HTTP $HTTP_CODE)"
else
    echo "⚠️  Static assets endpoint returned unexpected HTTP $HTTP_CODE"
fi

echo "4. Testing JavaScript files..."
# Try to find a JS file in the page and test it
JS_URL=$(echo "$RESPONSE" | grep -oE 'src="[^"]*\.js"' | head -1 | sed 's/src="//;s/"$//')
if [ -n "$JS_URL" ]; then
    # Handle relative URLs
    if [[ "$JS_URL" == /* ]]; then
        FULL_JS_URL="${BASE_URL}${JS_URL}"
    else
        FULL_JS_URL="${BASE_URL}/${JS_URL}"
    fi
    JS_HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" "$FULL_JS_URL" || echo "000")
    if [ "$JS_HTTP_CODE" = "200" ]; then
        echo "✅ JavaScript files accessible"
    else
        echo "⚠️  JavaScript file returned HTTP $JS_HTTP_CODE (may be bundled)"
    fi
else
    echo "ℹ️  No external JS files found (likely bundled)"
fi

echo "✅ Frontend test passed"
exit 0
