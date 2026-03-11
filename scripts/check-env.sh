#!/bin/bash
#
# Claw One 环境检查脚本
# 检查安装前的依赖和环境

set -e

CLAW_ONE_VERSION="0.1.0"
INSTALL_DIR="$HOME/claw-one"

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# 检查结果
CHECK_PASSED=0
CHECK_WARNING=0
CHECK_FAILED=0

print_header() {
    echo "========================================"
    echo "  Claw One 环境检查"
    echo "  Version: $CLAW_ONE_VERSION"
    echo "========================================"
    echo ""
}

print_ok() {
    echo -e "${GREEN}✓${NC} $1"
    CHECK_PASSED=$((CHECK_PASSED + 1))
}

print_warn() {
    echo -e "${YELLOW}⚠${NC} $1"
    CHECK_WARNING=$((CHECK_WARNING + 1))
}

print_error() {
    echo -e "${RED}✗${NC} $1"
    CHECK_FAILED=$((CHECK_FAILED + 1))
}

# 检查操作系统
check_os() {
    echo "[1/6] 检查操作系统..."
    
    OS=$(uname -s)
    case "$OS" in
        Linux)
            print_ok "操作系统: Linux"
            # 检查 systemd
            if command -v systemctl &> /dev/null; then
                print_ok "systemd 已安装"
            else
                print_warn "systemd 未安装（将使用手动启动模式）"
            fi
            ;;
        Darwin)
            print_warn "操作系统: macOS（实验性支持）"
            print_warn "推荐使用 Linux 系统"
            ;;
        *)
            print_error "操作系统: $OS（不支持）"
            print_error "仅支持 Linux 和 macOS"
            ;;
    esac
    echo ""
}

# 检查必需依赖
check_required_deps() {
    echo "[2/6] 检查必需依赖..."
    
    # git（必需，用于配置版本控制）
    if command -v git &> /dev/null; then
        GIT_VERSION=$(git --version | awk '{print $3}')
        print_ok "git 已安装 (版本: $GIT_VERSION)"
    else
        print_error "git 未安装（必需）"
        echo ""
        echo "请安装 git:"
        echo "  Ubuntu/Debian: sudo apt install git"
        echo "  CentOS/RHEL:   sudo yum install git"
        echo "  macOS:         brew install git"
        echo ""
    fi
    
    echo ""
}

# 检查可选依赖
check_optional_deps() {
    echo "[3/6] 检查可选依赖..."
    
    # curl 或 wget（用于健康检查）
    if command -v curl &> /dev/null; then
        print_ok "curl 已安装"
    elif command -v wget &> /dev/null; then
        print_ok "wget 已安装"
    else
        print_warn "curl/wget 未安装（健康检查功能受限）"
    fi
    
    # lsof 或 ss（用于端口检查）
    if command -v lsof &> /dev/null || command -v ss &> /dev/null; then
        print_ok "端口检查工具已安装"
    else
        print_warn "lsof/ss 未安装（端口检查功能受限）"
    fi
    
    echo ""
}

# 检查端口占用
check_port() {
    echo "[4/6] 检查端口占用..."
    
    DEFAULT_PORT=8080
    PORT_IN_USE=false
    
    if command -v lsof &> /dev/null; then
        if lsof -i :$DEFAULT_PORT &> /dev/null; then
            PORT_IN_USE=true
        fi
    elif command -v ss &> /dev/null; then
        if ss -tlnp | grep -q ":$DEFAULT_PORT "; then
            PORT_IN_USE=true
        fi
    elif command -v netstat &> /dev/null; then
        if netstat -tlnp 2>/dev/null | grep -q ":$DEFAULT_PORT "; then
            PORT_IN_USE=true
        fi
    fi
    
    if [ "$PORT_IN_USE" = true ]; then
        print_warn "端口 $DEFAULT_PORT 已被占用"
        echo "    安装后可在 ~/claw-one/config/claw-one.toml 中修改端口"
    else
        print_ok "端口 $DEFAULT_PORT 可用"
    fi
    
    echo ""
}

# 检查磁盘空间
check_disk_space() {
    echo "[5/6] 检查磁盘空间..."
    
    # 获取家目录可用空间（MB）
    if [ "$(uname -s)" = "Darwin" ]; then
        # macOS
        AVAILABLE=$(df -m "$HOME" | awk 'NR==2{print $4}')
    else
        # Linux
        AVAILABLE=$(df -m "$HOME" | awk 'NR==2{print $4}')
    fi
    
    REQUIRED=200  # 需要200MB
    
    if [ -z "$AVAILABLE" ]; then
        print_warn "无法检测磁盘空间"
    elif [ "$AVAILABLE" -lt "$REQUIRED" ]; then
        print_error "磁盘空间不足: ${AVAILABLE}MB（需要 ${REQUIRED}MB）"
    else
        print_ok "磁盘空间充足: ${AVAILABLE}MB"
    fi
    
    echo ""
}

# 检查安装目录
check_install_dir() {
    echo "[6/6] 检查安装目录..."
    
    if [ -d "$INSTALL_DIR" ]; then
        print_warn "安装目录已存在: $INSTALL_DIR"
        echo "    如需重新安装，请先卸载或备份现有配置"
    else
        print_ok "安装目录可用: $INSTALL_DIR"
    fi
    
    # 检查家目录可写
    if [ -w "$HOME" ]; then
        print_ok "家目录可写"
    else
        print_error "家目录不可写: $HOME"
    fi
    
    echo ""
}

# 打印检查摘要
print_summary() {
    echo "========================================"
    echo "           检查摘要"
    echo "========================================"
    echo -e "${GREEN}通过: $CHECK_PASSED${NC}"
    echo -e "${YELLOW}警告: $CHECK_WARNING${NC}"
    echo -e "${RED}失败: $CHECK_FAILED${NC}"
    echo ""
    
    if [ $CHECK_FAILED -gt 0 ]; then
        echo -e "${RED}✗ 环境检查未通过${NC}"
        echo "请解决上述错误后重新运行安装"
        exit 1
    elif [ $CHECK_WARNING -gt 0 ]; then
        echo -e "${YELLOW}⚠ 环境检查通过，但有警告${NC}"
        echo "可以继续安装，但部分功能可能受限"
        echo ""
        echo "如需完整功能，请先解决警告项"
        exit 0
    else
        echo -e "${GREEN}✓ 环境检查通过${NC}"
        echo "可以开始安装"
        exit 0
    fi
}

# 主流程
main() {
    print_header
    check_os
    check_required_deps
    check_optional_deps
    check_port
    check_disk_space
    check_install_dir
    print_summary
}

main "$@"
