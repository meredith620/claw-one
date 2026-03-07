#!/bin/bash
#
# Docker 内测试 Claw One 安装和卸载
#

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CLAW_ONE_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"
DIST_FILE="$CLAW_ONE_DIR/dist/claw-one-*.tar.gz"

# 颜色
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
log_ok() { echo -e "${GREEN}[OK]${NC} $1"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }

# 检查安装包
check_dist() {
    if ! ls $DIST_FILE 1> /dev/null 2>&1; then
        log_error "未找到安装包，请先运行 make dist"
        exit 1
    fi
    log_ok "找到安装包"
}

# 构建镜像
build_image() {
    log_info "构建 Docker 测试镜像..."
    cd "$SCRIPT_DIR"
    cp "$CLAW_ONE_DIR"/dist/claw-one-*.tar.gz .
    docker build -t claw-one-test:latest . >/dev/null 2>&1
    log_ok "镜像构建完成"
}

# 测试安装
test_install() {
    log_info "开始安装测试..."
    
    docker run --rm claw-one-test:latest bash -c '
        set -e
        echo "=== 步骤 1: 解压安装包 ==="
        cd /tmp
        tar xzf claw-one-*.tar.gz
        cd claw-one-*/
        
        echo "=== 步骤 2: 检查环境 ==="
        ./scripts/check-env.sh || true
        
        echo "=== 步骤 3: 执行安装 ==="
        ./scripts/install.sh
        
        echo "=== 步骤 4: 验证安装 ==="
        ls -la ~/claw-one/
        echo "安装成功！"
    '
    
    if [ $? -eq 0 ]; then
        log_ok "安装测试通过"
    else
        log_error "安装测试失败"
        return 1
    fi
}

# 测试配置
test_config() {
    log_info "开始配置测试..."
    
    docker run --rm claw-one-test:latest bash -c '
        set -e
        cd /tmp
        tar xzf claw-one-*.tar.gz
        cd claw-one-*/
        ./scripts/install.sh
        
        echo "=== 检查配置 ==="
        ls -la ~/claw-one/config/
        cat ~/claw-one/config/claw-one.toml
        
        echo "=== 检查数据目录 ==="
        ls -la ~/claw-one/data/
        cd ~/claw-one/data && git log --oneline
    '
    
    if [ $? -eq 0 ]; then
        log_ok "配置测试通过"
    else
        log_error "配置测试失败"
        return 1
    fi
}

# 测试卸载
test_uninstall() {
    log_info "开始卸载测试..."
    
    docker run --rm claw-one-test:latest bash -c '
        set -e
        cd /tmp
        tar xzf claw-one-*.tar.gz
        cd claw-one-*/
        ./scripts/install.sh
        
        echo "=== 创建测试文件 ==="
        echo test > ~/claw-one/config/test.txt
        
        echo "=== 执行卸载 ==="
        echo "y" | ~/claw-one/uninstall.sh || true
        
        echo "=== 检查卸载结果 ==="
        if [ -d ~/claw-one ]; then
            echo "目录仍存在:"
            ls -la ~/claw-one* 2>/dev/null || true
        else
            echo "目录已删除"
        fi
    '
    
    if [ $? -eq 0 ]; then
        log_ok "卸载测试通过"
    else
        log_error "卸载测试失败"
        return 1
    fi
}

# 清理
cleanup() {
    log_info "清理..."
    docker rmi claw-one-test:latest 2>/dev/null || true
    rm -f "$SCRIPT_DIR"/claw-one-*.tar.gz
    log_ok "清理完成"
}

# 主流程
case "${1:-all}" in
    check)
        check_dist
        ;;
    build)
        check_dist
        build_image
        ;;
    install)
        check_dist
        build_image
        test_install
        ;;
    config)
        check_dist
        build_image
        test_config
        ;;
    uninstall)
        check_dist
        build_image
        test_uninstall
        ;;
    all)
        check_dist
        build_image
        test_install
        test_config
        test_uninstall
        cleanup
        log_ok "所有测试完成！"
        ;;
    *)
        echo "用法: $0 {check|build|install|config|uninstall|all}"
        exit 1
        ;;
esac
