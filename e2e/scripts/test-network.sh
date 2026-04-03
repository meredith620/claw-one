# 快速测试 Playwright 容器是否能访问 claw-one 服务
#!/bin/bash

# 获取 claw-one 容器 IP
CLAW_ONE_IP=$(docker inspect -f '{{range .NetworkSettings.Networks}}{{.IPAddress}}{{end}}' claw-one-test-app 2>/dev/null || echo "")

if [ -z "$CLAW_ONE_IP" ]; then
    echo "claw-one 容器未运行，请先启动:"
    echo "  docker compose -f e2e/docker-compose.test.yml up -d claw-one"
    exit 1
fi

echo "claw-one 容器 IP: $CLAW_ONE_IP"
echo ""
echo "测试连接..."
docker run --rm --network claw-one-test-net mcr.microsoft.com/playwright:v1.51.1-noble \
    curl -s http://claw-one-test-app:8080/health || echo "连接失败"
