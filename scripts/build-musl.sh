#!/bin/bash
#
# Claw One musl 静态构建脚本
# 使用 Docker 构建完全静态链接的二进制

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
VERSION=${VERSION:-$(cd "$PROJECT_ROOT" && git describe --tags --always --dirty 2>/dev/null || echo "0.1.0")}
ARCH="x86_64"
DIST_NAME="claw-one-${VERSION}-${ARCH}-musl"

echo "========================================"
echo "Claw One musl 静态构建"
echo "Version: $VERSION"
echo "========================================"
echo ""

# 检查 Docker
if ! command -v docker &> /dev/null; then
    echo "❌ Docker 未安装"
    exit 1
fi

# 创建临时构建目录
BUILD_DIR=$(mktemp -d)
trap "rm -rf $BUILD_DIR" EXIT

echo "📦 步骤 1: 准备构建环境..."
mkdir -p "$BUILD_DIR/hull/src"
mkdir -p "$BUILD_DIR/bridge"
mkdir -p "$BUILD_DIR/static"
mkdir -p "$BUILD_DIR/scripts"
mkdir -p "$BUILD_DIR/config"

# 复制源代码
cp -r "$PROJECT_ROOT/hull/"*.toml "$BUILD_DIR/hull/"
cp -r "$PROJECT_ROOT/hull/src/"* "$BUILD_DIR/hull/src/"
cp -r "$PROJECT_ROOT/bridge/"* "$BUILD_DIR/bridge/"
cp -r "$PROJECT_ROOT/static/"* "$BUILD_DIR/static/"
cp -r "$PROJECT_ROOT/scripts/"*.sh "$BUILD_DIR/scripts/"
cp -r "$PROJECT_ROOT/scripts/README.md" "$BUILD_DIR/scripts/"
cp -r "$PROJECT_ROOT/config/"* "$BUILD_DIR/config/" 2>/dev/null || true

echo "✅ 源代码已复制"
echo ""

# 创建 Dockerfile
cat > "$BUILD_DIR/Dockerfile" << 'DOCKERFILE_EOF'
FROM rust:1.75-alpine3.19 AS builder

# 安装构建依赖
RUN apk add --no-cache \
    musl-dev \
    openssl-dev \
    openssl-libs-static \
    zlib-dev \
    zlib-static \
    git \
    nodejs \
    npm \
    pkgconfig

WORKDIR /build

# 复制 hull (Rust)
COPY hull ./hull

# 复制 bridge (Node.js)
COPY bridge ./bridge

# 复制静态文件目录
COPY static ./static

# 复制脚本和配置
COPY scripts ./scripts
COPY config ./config

# 构建 bridge (前端)
RUN cd bridge && npm install && npx vite build

# 构建 hull (musl 静态)
RUN cd hull && cargo build --release --target x86_64-unknown-linux-musl

# 收集输出
RUN mkdir -p /output/dist && \
    cp /build/hull/target/x86_64-unknown-linux-musl/release/claw-one /output/ && \
    cp -r /build/static/dist /output/static && \
    cp /build/scripts/*.sh /output/ && \
    cp /build/scripts/README.md /output/

# 第二阶段：创建分发包
FROM alpine:3.19

RUN apk add --no-cache bash

COPY --from=builder /output/ /tmp/build/

WORKDIR /tmp

RUN mkdir -p dist/${DIST_NAME}/bin && \
    mkdir -p dist/${DIST_NAME}/share/static && \
    mkdir -p dist/${DIST_NAME}/share/config && \
    mkdir -p dist/${DIST_NAME}/scripts && \
    cp /tmp/build/claw-one dist/${DIST_NAME}/bin/ && \
    chmod +x dist/${DIST_NAME}/bin/claw-one && \
    cp -r /tmp/build/static/* dist/${DIST_NAME}/share/static/ && \
    cp /tmp/build/*.sh dist/${DIST_NAME}/scripts/ && \
    cp /tmp/build/README.md dist/${DIST_NAME}/ && \
    printf '%s\n' '[server]' 'host = "0.0.0.0"' 'port = 8080' 'log_level = "info"' '' '[openclaw]' 'openclaw_home = "~/.openclaw"' 'service_name = "openclaw"' 'health_port = 18790' 'health_timeout = 30' > dist/${DIST_NAME}/share/config/claw-one.toml.template && \
    tar czf dist/${DIST_NAME}.tar.gz -C dist ${DIST_NAME}/

# 创建自解压脚本
COPY self-extract-header.sh /tmp/
RUN sed \
    -e "s|__VERSION__|${VERSION}|g" \
    -e "s|__ARCH__|x86_64-musl|g" \
    -e "s|__BUILD_DATE__|$(date -Iseconds)|g" \
    /tmp/self-extract-header.sh > /tmp/dist/${DIST_NAME}-install.sh && \
    echo "" >> /tmp/dist/${DIST_NAME}-install.sh && \
    echo "# === 数据开始 ===" >> /tmp/dist/${DIST_NAME}-install.sh && \
    base64 /tmp/dist/${DIST_NAME}.tar.gz >> /tmp/dist/${DIST_NAME}-install.sh && \
    chmod +x /tmp/dist/${DIST_NAME}-install.sh

CMD ["cp", "/tmp/dist/claw-one-${VERSION}-x86_64-musl-install.sh", "/output/"]
DOCKERFILE_EOF

# 复制自解压头部模板
cp "$PROJECT_ROOT/scripts/self-extract-header.sh" "$BUILD_DIR/"

# 替换 Dockerfile 中的变量
sed -i "s/\${DIST_NAME}/$DIST_NAME/g" "$BUILD_DIR/Dockerfile"
sed -i "s/\${VERSION}/$VERSION/g" "$BUILD_DIR/Dockerfile"

echo "📦 步骤 2: 构建 Docker 镜像..."
docker build -t claw-one-musl-builder:latest "$BUILD_DIR"

echo "✅ 镜像构建完成"
echo ""

# 提取构建产物
echo "📦 步骤 3: 提取构建产物..."
mkdir -p "$PROJECT_ROOT/dist"

# 运行容器提取文件
docker run --rm \
    -v "$PROJECT_ROOT/dist:/output" \
    claw-one-musl-builder:latest \
    sh -c "cp /tmp/dist/* /output/"

echo "✅ 构建产物已提取到 dist/"
echo ""

# 验证
echo "📦 步骤 4: 验证构建产物..."
ls -lh "$PROJECT_ROOT/dist/"*-musl-*

echo ""
echo "========================================"
echo "musl 静态构建完成！"
echo "========================================"
echo ""
echo "输出文件:"
ls -1 "$PROJECT_ROOT/dist/"*-musl-* 2>/dev/null | while read f; do
    echo "  - $(basename "$f")"
done
echo ""
echo "测试命令:"
echo "  ./dist/${DIST_NAME}-install.sh --check"
echo "  ./dist/${DIST_NAME}-install.sh -y"
