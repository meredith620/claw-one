#!/bin/bash
# run-playwright-tests.sh - 在容器中运行 Playwright Browser E2E 测试

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR/.."

echo "=========================================="
echo "Playwright Browser E2E 测试"
echo "=========================================="

# 确保报告目录存在
mkdir -p e2e/playwright-report e2e/test-results

# 构建并运行 Playwright 容器
echo ""
echo "正在构建 Playwright 测试容器..."
docker compose -f e2e/docker-compose.test.yml build playwright

echo ""
echo "运行 Browser E2E 测试..."
docker compose -f e2e/docker-compose.test.yml run --rm playwright

# 检查测试结果
EXIT_CODE=$?

if [ $EXIT_CODE -eq 0 ]; then
    echo ""
    echo "=========================================="
    echo "✅ Browser E2E 测试全部通过"
    echo "=========================================="
else
    echo ""
    echo "=========================================="
    echo "❌ Browser E2E 测试有失败"
    echo "=========================================="
    echo ""
    echo "测试报告位置:"
    echo "  - HTML 报告: e2e/playwright-report/index.html"
    echo "  - 测试结果: e2e/test-results/"
fi

exit $EXIT_CODE
