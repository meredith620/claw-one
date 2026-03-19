#!/bin/bash
#
# Claw One musl 静态构建脚本
# 两阶段构建：
#   1. 构建编译环境镜像（如果本地不存在）
#   2. 使用编译环境镜像构建应用

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
VERSION=${VERSION:-$(cd "$PROJECT_ROOT" && git describe --tags --always --dirty 2>/dev/null || echo "0.1.0")}
ARCH="x86_64"
DIST_NAME="claw-one-${VERSION}-${ARCH}"
BUILDER_IMAGE="claw-one-builder:${VERSION}"
BUILDER_IMAGE_LATEST="claw-one-builder:latest"

# 缓存目录
CACHE_DIR="$PROJECT_ROOT/.cache"
CARGO_REGISTRY_CACHE="$CACHE_DIR/cargo-registry"
CARGO_GIT_CACHE="$CACHE_DIR/cargo-git"
CARGO_TARGET_CACHE="$CACHE_DIR/cargo-target"
NPM_CACHE="$CACHE_DIR/npm"

# 当前用户ID（用于容器内权限映射）
USER_ID=$(id -u)
GROUP_ID=$(id -g)

echo "========================================"
echo "Claw One musl 静态构建"
echo "Version: $VERSION"
echo "Builder Image: $BUILDER_IMAGE"
echo "========================================"
echo ""

# 检查 Docker
if ! command -v docker &> /dev/null; then
    echo "❌ Docker 未安装"
    exit 1
fi

# 阶段1: 构建/检查编译环境镜像
build_builder_image() {
    echo "📦 阶段1: 准备编译环境镜像..."

    # 检查本地是否已存在对应版本的镜像
    if docker image inspect "$BUILDER_IMAGE" >/dev/null 2>&1; then
        echo "✅ 编译环境镜像已存在: $BUILDER_IMAGE"
        return 0
    fi

    # 检查是否存在 latest 标签的镜像（可重用基础层）
    if docker image inspect "$BUILDER_IMAGE_LATEST" >/dev/null 2>&1; then
        echo "📝 发现旧版本镜像，将重新构建新版本..."
    fi

    echo "🔨 构建编译环境镜像..."
    echo "   Dockerfile: scripts/Dockerfile.builder"
    echo "   镜像名称: $BUILDER_IMAGE"
    echo ""

    docker build \
        -f "$SCRIPT_DIR/Dockerfile.builder" \
        -t "$BUILDER_IMAGE" \
        -t "$BUILDER_IMAGE_LATEST" \
        "$SCRIPT_DIR"

    echo ""
    echo "✅ 编译环境镜像构建完成"
    echo "   镜像: $BUILDER_IMAGE"
    echo ""
}

# 阶段2: 使用编译环境镜像构建应用
build_application() {
    echo "📦 阶段2: 构建应用程序..."
    echo ""

    # 创建临时目录用于输出
    OUTPUT_DIR=$(mktemp -d)
    # 设置输出目录权限，确保容器内用户可以写入
    chmod 777 "$OUTPUT_DIR"
    mkdir -p "$OUTPUT_DIR/static"
    chmod 777 "$OUTPUT_DIR/static"
    # 清理时先修复权限再删除（容器内创建的文件需要修改权限）
    trap "docker run --rm -v $OUTPUT_DIR:/out alpine:latest sh -c 'chmod -R 777 /out' 2>/dev/null; rm -rf $OUTPUT_DIR" EXIT

    echo "🔨 构建 bridge (前端)..."
    docker run --rm \
        --user "$USER_ID:$GROUP_ID" \
        -v "$PROJECT_ROOT/bridge:/build/bridge:ro" \
        -v "$OUTPUT_DIR/static:/output/static" \
        -v "$NPM_CACHE:/home/builder/.npm" \
        -e NPM_CONFIG_CACHE=/home/builder/.npm \
        -w /home/builder \
        "$BUILDER_IMAGE" \
        sh -c "cp -r /build/bridge/* /home/builder/ && npm install && npx vite build --outDir /output/static"

    echo "✅ 前端构建完成"
    echo ""

    echo "🔨 构建 hull (musl 静态)..."
    docker run --rm \
        --user "$USER_ID:$GROUP_ID" \
        -v "$PROJECT_ROOT/hull:/build/hull:ro" \
        -v "$OUTPUT_DIR:/output" \
        -v "$CARGO_REGISTRY_CACHE:/usr/local/cargo/registry" \
        -v "$CARGO_GIT_CACHE:/usr/local/cargo/git" \
        -v "$CARGO_TARGET_CACHE:/home/builder/target" \
        -e CARGO_HOME=/usr/local/cargo \
        -e CARGO_TARGET_DIR=/home/builder/target \
        -e GIT_COMMIT_HASH="$(cd "$PROJECT_ROOT" && git rev-parse --short HEAD)" \
        -w /home/builder \
        "$BUILDER_IMAGE" \
        sh -c "cp -r /build/hull/* /home/builder/ && cp /build/hull/Cargo.lock /home/builder/ && \
               cargo build --release --target x86_64-unknown-linux-musl && \
               cp /home/builder/target/x86_64-unknown-linux-musl/release/claw-one /output/"

    echo "✅ 后端构建完成"
    echo ""

    # 收集输出
    echo "📦 打包分发包..."
    mkdir -p "$PROJECT_ROOT/dist"

    # 创建分发目录结构
    DIST_TMP=$(mktemp -d)
    mkdir -p "$DIST_TMP/$DIST_NAME/bin"
    mkdir -p "$DIST_TMP/$DIST_NAME/share/static"
    mkdir -p "$DIST_TMP/$DIST_NAME/share/config"
    mkdir -p "$DIST_TMP/$DIST_NAME/scripts"

    # 复制构建产物
    cp "$OUTPUT_DIR/claw-one" "$DIST_TMP/$DIST_NAME/bin/"
    chmod +x "$DIST_TMP/$DIST_NAME/bin/claw-one"
    cp -r "$OUTPUT_DIR/static/"* "$DIST_TMP/$DIST_NAME/share/static/"

    # 复制脚本和文档
    cp "$PROJECT_ROOT/scripts/install.sh" \
       "$PROJECT_ROOT/scripts/uninstall.sh" \
       "$PROJECT_ROOT/scripts/check-env.sh" \
       "$DIST_TMP/$DIST_NAME/scripts/"
    chmod +x "$DIST_TMP/$DIST_NAME/scripts/"*.sh

    cp "$PROJECT_ROOT/scripts/setup-config.sh" "$DIST_TMP/$DIST_NAME/bin/"
    chmod +x "$DIST_TMP/$DIST_NAME/bin/setup-config.sh"

    cp "$PROJECT_ROOT/scripts/README.md" "$DIST_TMP/$DIST_NAME/"

    # 生成配置模板
    cat > "$DIST_TMP/$DIST_NAME/share/config/claw-one.toml.template" << EOF
[server]
host = "0.0.0.0"
port = 8080
log_level = "info"

[openclaw]
openclaw_home = "~/.openclaw"
service_name = "openclaw"
health_port = 18790
health_timeout = 30

[paths]
data_dir = "~/claw-one/data"
static_dir = "~/claw-one/share/static"

[features]
auto_backup = true
safe_mode = true
EOF

    # 创建 tar.gz
    tar czf "$PROJECT_ROOT/dist/$DIST_NAME.tar.gz" -C "$DIST_TMP" "$DIST_NAME"

    # 清理临时目录
    rm -rf "$DIST_TMP"

    echo "✅ 分发包已创建: dist/$DIST_NAME.tar.gz"
    echo ""
}

# 构建自解压脚本
build_self_extract() {
    echo "📦 创建自解压安装脚本..."

    # 创建临时目录
    TMP_DIR=$(mktemp -d)

    # 解压 tar.gz
    tar xzf "$PROJECT_ROOT/dist/$DIST_NAME.tar.gz" -C "$TMP_DIR"

    # 重新打包为 tar.gz 并 base64 编码
    tar czf - -C "$TMP_DIR" "$DIST_NAME" | base64 -w0 > "$TMP_DIR/archive.b64"

    # 创建自解压脚本头部
    cat > "$PROJECT_ROOT/dist/$DIST_NAME-install.sh" << 'SCRIPT_HEADER'
#!/bin/bash
#
# Claw One 自解压安装脚本
# 版本: SCRIPT_VERSION
# 架构: SCRIPT_ARCH

set -e

VERSION="SCRIPT_VERSION"
ARCH="SCRIPT_ARCH"

# 颜色输出
if [[ -t 1 ]]; then
    RED='\033[0;31m'
    GREEN='\033[0;32m'
    YELLOW='\033[1;33m'
    BLUE='\033[0;34m'
    NC='\033[0m'
else
    RED='' GREEN='' YELLOW='' BLUE='' NC=''
fi

print_info() { echo -e "${BLUE}ℹ${NC} $1"; }
print_ok() { echo -e "${GREEN}✓${NC} $1"; }
print_warn() { echo -e "${YELLOW}⚠${NC} $1"; }
print_error() { echo -e "${RED}✗${NC} $1"; }

show_help() {
    cat << EOF
Claw One 自解压安装脚本

用法: ./install.sh [选项]

选项:
  -h, --help      显示帮助
  -c, --check     仅检查环境
  -t DIR          指定安装目录 (默认: ~/claw-one)
  -y              自动确认

EOF
}

# 解析参数
INSTALL_DIR="${CLAW_ONE_INSTALL_DIR:-$HOME/claw-one}"
CHECK_ONLY="no"
AUTO_CONFIRM="no"

while [[ $# -gt 0 ]]; do
    case "$1" in
        -h|--help) show_help; exit 0 ;;
        -c|--check) CHECK_ONLY="yes"; shift ;;
        -t|--target) INSTALL_DIR="$2"; shift 2 ;;
        -y|--yes) AUTO_CONFIRM="yes"; shift ;;
        *) print_error "未知选项: $1"; exit 1 ;;
    esac
done

echo "========================================"
echo "  Claw One 自解压安装脚本"
echo "  Version: $VERSION"
echo "  Arch: $ARCH"
echo "========================================"
echo ""

# 环境检查
print_info "检查系统环境..."

# 检查基本命令
for cmd in tar mkdir cp chmod; do
    if ! command -v "$cmd" &> /dev/null; then
        print_error "缺少必需命令: $cmd"
        exit 1
    fi
done

# 检查可选命令
if command -v git &> /dev/null; then
    print_ok "git 已安装"
else
    print_warn "未安装 git"
fi

if command -v systemctl &> /dev/null; then
    print_ok "systemd 已安装"
else
    print_warn "未检测到 systemd"
fi

print_ok "环境检查通过"

if [[ "$CHECK_ONLY" == "yes" ]]; then
    exit 0
fi

# 确认安装
if [[ "$AUTO_CONFIRM" != "yes" ]]; then
    echo ""
    read -p "安装到 $INSTALL_DIR? [Y/n]: " confirm
    if [[ "$confirm" =~ ^[Nn]$ ]]; then
        read -p "请输入安装路径: " INSTALL_DIR
    fi
fi

# 处理已存在安装
if [[ -d "$INSTALL_DIR" ]]; then
    print_warn "检测到已有安装: $INSTALL_DIR"
    if [[ "$AUTO_CONFIRM" != "yes" ]]; then
        read -p "是否升级安装? [Y/n]: " upgrade
        if [[ "$upgrade" =~ ^[Nn]$ ]]; then
            print_info "取消安装"
            exit 0
        fi
    fi
    # 升级安装：只删除程序文件，保留配置、数据和日志
    print_info "升级安装，保留配置和数据..."
    rm -rf "$INSTALL_DIR/bin" 2>/dev/null || true
    rm -rf "$INSTALL_DIR/share" 2>/dev/null || true
    rm -rf "$INSTALL_DIR/scripts" 2>/dev/null || true
    rm -f "$INSTALL_DIR/uninstall.sh" 2>/dev/null || true
    rm -f "$INSTALL_DIR/README.md" 2>/dev/null || true
    print_ok "旧版本文件已清理"
fi

# 解压安装
print_info "解压安装包..."

# 从脚本中提取数据（在 __ARCHIVE_MARKER__ 之后）
SCRIPT_PATH="${BASH_SOURCE[0]}"
MARKER_LINE=$(grep -n "__ARCHIVE_MARKER__" "$SCRIPT_PATH" | tail -1 | cut -d: -f1)
ARCHIVE_START=$((MARKER_LINE + 1))

mkdir -p "$INSTALL_DIR"
tail -n +$ARCHIVE_START "$SCRIPT_PATH" | base64 -d | tar xzf - -C "$INSTALL_DIR" --strip-components=1 --skip-old-files 2>/dev/null || \
    tail -n +$ARCHIVE_START "$SCRIPT_PATH" | base64 -d | tar xzf - -C "$INSTALL_DIR" --strip-components=1

print_ok "文件已解压到: $INSTALL_DIR"

# 初始化
cd "$INSTALL_DIR"

# 创建数据目录
mkdir -p "$INSTALL_DIR/data"
mkdir -p "$INSTALL_DIR/logs"

# 初始化 git（仅在未初始化时）
if command -v git \&> /dev/null && [[ ! -d "$INSTALL_DIR/data/.git" ]]; then
    cd "$INSTALL_DIR/data"
    git init --quiet 2>/dev/null || true
    git config user.name "Claw One" 2>/dev/null || true
    git config user.email "dev@claw.one" 2>/dev/null || true
    echo "# Claw One Data" > README.md
    git add README.md 2>/dev/null || true
    git commit -m "Initial commit" --quiet 2>/dev/null || true
    print_ok "Git 仓库已初始化"
fi

# 创建快捷方式
if [[ -d "$HOME/.local/bin" ]]; then
    ln -sf "$INSTALL_DIR/bin/claw-one" "$HOME/.local/bin/claw-one"
    print_ok "快捷方式已创建: ~/.local/bin/claw-one"
fi

# 打印完成信息
echo ""
echo "========================================"
echo -e "${GREEN}    安装完成！${NC}"
echo "========================================"
echo ""
echo "安装位置: $INSTALL_DIR"
echo "配置文件: $INSTALL_DIR/config/claw-one.toml"
echo "数据目录: $INSTALL_DIR/data"
echo "日志目录: $INSTALL_DIR/logs"
echo ""
echo "启动命令:"
echo "  前台运行: $INSTALL_DIR/bin/claw-one run"
echo "  后台启动: $INSTALL_DIR/bin/claw-one start"
echo ""
echo "访问: http://localhost:8080"
echo ""

exit 0

__ARCHIVE_MARKER__
SCRIPT_HEADER

    # 替换版本变量
    sed -i "s/SCRIPT_VERSION/$VERSION/g" "$PROJECT_ROOT/dist/$DIST_NAME-install.sh"
    sed -i "s/SCRIPT_ARCH/$ARCH/g" "$PROJECT_ROOT/dist/$DIST_NAME-install.sh"

    # 追加 base64 编码的归档数据
    cat "$TMP_DIR/archive.b64" >> "$PROJECT_ROOT/dist/$DIST_NAME-install.sh"

    # 设置可执行权限
    chmod +x "$PROJECT_ROOT/dist/$DIST_NAME-install.sh"

    # 清理
    rm -rf "$TMP_DIR"

    echo "✅ 自解压脚本已创建: dist/$DIST_NAME-install.sh"
    echo ""
}

# 主流程
main() {
    # 创建缓存目录
    mkdir -p "$CARGO_REGISTRY_CACHE"
    mkdir -p "$CARGO_GIT_CACHE"
    mkdir -p "$CARGO_TARGET_CACHE"
    mkdir -p "$NPM_CACHE"

    # 阶段1: 构建编译环境
    build_builder_image

    # 阶段2: 构建应用程序
    build_application

    # 可选: 构建自解压脚本
    build_self_extract

    echo "========================================"
    echo "构建完成！"
    echo "========================================"
    echo ""
    echo "输出文件:"
    ls -lh "$PROJECT_ROOT/dist/"*.tar.gz "$PROJECT_ROOT/dist/"*-install.sh 2>/dev/null || true
    echo ""
    echo "测试命令:"
    echo "  ./dist/$DIST_NAME-install.sh --check"
    echo "  ./dist/$DIST_NAME-install.sh -y"
}

main "$@"
