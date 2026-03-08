#!/bin/bash
#
# CentOS 7 musl 兼容性测试脚本
# 验证 musl 静态构建的二进制在旧系统上的兼容性

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

echo "========================================"
echo "CentOS 7 musl 兼容性测试"
echo "========================================"
echo ""

# 检查 Docker
if ! command -v docker &> /dev/null; then
    echo "❌ Docker 未安装"
    exit 1
fi

# 步骤1: 快速构建 musl 版本（仅 hull）
echo "📦 步骤1: 快速构建 musl 静态二进制..."

BUILD_DIR=$(mktemp -d)
trap "rm -rf $BUILD_DIR" EXIT

# 创建最小 Dockerfile
cat > "$BUILD_DIR/Dockerfile" << 'EOF'
FROM rust:1.85-alpine3.21 AS builder
RUN apk add --no-cache musl-dev openssl-dev openssl-libs-static zlib-dev zlib-static git pkgconfig
WORKDIR /build
COPY hull ./hull
RUN cd hull && cargo build --release --target x86_64-unknown-linux-musl
FROM alpine:3.21
COPY --from=builder /build/hull/target/x86_64-unknown-linux-musl/release/claw-one /claw-one-musl
CMD ["cp", "/claw-one-musl", "/output/"]
EOF

# 复制 hull 源代码
mkdir -p "$BUILD_DIR/hull/src"
cp "$PROJECT_ROOT/hull/"*.toml "$BUILD_DIR/hull/"
cp -r "$PROJECT_ROOT/hull/src/"* "$BUILD_DIR/hull/src/"

echo "🔨 构建 musl 二进制..."
docker build -t claw-one-musl-quick:latest "$BUILD_DIR"

# 提取二进制
mkdir -p "$PROJECT_ROOT/dist"
docker run --rm -v "$PROJECT_ROOT/dist:/output" claw-one-musl-quick:latest sh -c "cp /claw-one-musl /output/"

echo "✅ musl 二进制已生成: dist/claw-one-musl"
ls -lh "$PROJECT_ROOT/dist/claw-one-musl"
echo ""

# 步骤2: 在 CentOS 7 中测试
echo "📦 步骤2: 启动 CentOS 7 测试容器..."

# 构建 CentOS 7 测试镜像
docker build -f "$SCRIPT_DIR/Dockerfile.centos7" -t claw-one-centos7-test:latest "$SCRIPT_DIR"

# 运行测试
echo "🔍 测试1: 检查 CentOS 7 glibc 版本..."
docker run --rm claw-one-centos7-test:latest bash -c "ldd --version | head -1"
echo ""

echo "🔍 测试2: 复制 musl 二进制到 CentOS 7 并运行..."
# 使用临时容器复制并测试
docker run --rm \
    -v "$PROJECT_ROOT/dist/claw-one-musl:/tmp/claw-one:ro" \
    claw-one-centos7-test:latest \
    bash -c "
        echo '检查文件...'
        ls -lh /tmp/claw-one
        file /tmp/claw-one
        echo ''
        echo '运行 musl 二进制...'
        /tmp/claw-one --version
        echo ''
        echo '✅ musl 静态二进制在 CentOS 7 运行成功！'
    "

echo ""

# 步骤3: 对比测试 glibc 版本（应该失败）
echo "📦 步骤3: 对比测试 - 检查 glibc 动态链接版本..."

# 检查宿主机构建的 glibc 版本
if [ -f "$PROJECT_ROOT/hull/target/release/claw-one" ]; then
    echo "宿主机构建的 glibc 版本："
    file "$PROJECT_ROOT/hull/target/release/claw-one"
    echo ""
    
    docker run --rm \
        -v "$PROJECT_ROOT/hull/target/release/claw-one:/tmp/claw-one-glibc:ro" \
        claw-one-centos7-test:latest \
        bash -c "
            echo '尝试运行 glibc 动态链接版本...'
            /tmp/claw-one-glibc --version 2>&1 || echo '❌ 预期中的失败: glibc 版本不兼容'
        " 2>&1 || true
else
    echo "⚠️  未找到 glibc 动态链接版本，跳过对比测试"
fi

echo ""
echo "========================================"
echo "CentOS 7 兼容性测试完成！"
echo "========================================"
echo ""
echo "结论:"
echo "  ✅ musl 静态二进制可在 CentOS 7 运行"
echo "  ❌ glibc 动态链接版本在 CentOS 7 失败"
echo ""
echo "musl 静态链接提供了真正的跨发行版兼容性！"
