#!/bin/bash
#
# Claw One 自解压安装脚本
# 将 tar.gz 包嵌入脚本中，单文件分发
#
# 使用方法:
#   ./claw-one-VERSION-install.sh           # 交互式安装
#   ./claw-one-VERSION-install.sh --help    # 显示帮助
#   ./claw-one-VERSION-install.sh --check   # 仅检查环境
#   ./claw-one-VERSION-install.sh --target /path  # 指定安装路径

set -e

# 脚本元数据（打包时会被替换）
VERSION="__VERSION__"
ARCH="__ARCH__"
BUILD_DATE="__BUILD_DATE__"

# 默认安装路径
INSTALL_DIR="${CLAW_ONE_INSTALL_DIR:-$HOME/claw-one}"

# 颜色输出（仅在TTY时启用）
if [[ -t 1 ]]; then
    RED='\033[0;31m'
    GREEN='\033[0;32m'
    YELLOW='\033[1;33m'
    BLUE='\033[0;34m'
    CYAN='\033[0;36m'
    NC='\033[0m'
else
    RED='' GREEN='' YELLOW='' BLUE='' CYAN='' NC=''
fi

print_info() { echo -e "${BLUE}ℹ${NC} $1"; }
print_ok() { echo -e "${GREEN}✓${NC} $1"; }
print_warn() { echo -e "${YELLOW}⚠${NC} $1"; }
print_error() { echo -e "${RED}✗${NC} $1"; }

# 显示帮助
show_help() {
    cat << 'EOF'
Claw One 自解压安装脚本

使用方法:
  ./install.sh [选项]

选项:
  -h, --help          显示此帮助信息
  -c, --check         仅检查环境，不安装
  -t, --target DIR    指定安装目录 (默认: ~/claw-one)
  -y, --yes           自动确认，无需交互
  -v, --version       显示版本信息

环境变量:
  CLAW_ONE_INSTALL_DIR    设置安装目录

示例:
  ./install.sh                    # 交互式安装
  ./install.sh -y                 # 自动安装到默认目录
  ./install.sh -t /opt/claw-one   # 安装到指定目录
EOF
}

# 显示版本
show_version() {
    echo "Claw One Installer"
    echo "Version: $VERSION"
    echo "Arch: $ARCH"
    echo "Build: $BUILD_DATE"
}

# 检查环境
check_environment() {
    print_info "检查系统环境..."
    
    local errors=0
    
    # 检查操作系统
    if [[ "$OSTYPE" == "linux-gnu"* ]] || [[ "$OSTYPE" == "linux-musl"* ]]; then
        print_ok "操作系统: Linux"
    else
        print_warn "未测试的操作系统: $OSTYPE"
    fi
    
    # 检查必需命令
    for cmd in tar mkdir cp chmod; do
        if command -v "$cmd" &> /dev/null; then
            continue
        else
            print_error "缺少必需命令: $cmd"
            ((errors++))
        fi
    done
    
    # 检查可选命令
    if command -v git &> /dev/null; then
        print_ok "git 已安装"
    else
        print_warn "未安装 git，配置版本控制将不可用"
    fi
    
    # 检查 systemd
    if command -v systemctl &> /dev/null; then
        print_ok "systemd 已安装"
    else
        print_warn "未检测到 systemd，将使用手动启动模式"
    fi
    
    # 检查磁盘空间
    local available=$(df -k "$HOME" | awk 'NR==2 {print $4}')
    if [[ $available -gt 51200 ]]; then  # 50MB
        print_ok "磁盘空间充足"
    else
        print_warn "磁盘空间不足 50MB"
    fi
    
    if [[ $errors -eq 0 ]]; then
        print_ok "环境检查通过"
        return 0
    else
        print_error "环境检查失败，请修复上述问题"
        return 1
    fi
}

# 提取并安装
extract_and_install() {
    local target_dir="$1"
    local auto_confirm="$2"
    
    print_info "准备安装 Claw One..."
    
    # 确认安装路径
    if [[ "$auto_confirm" != "yes" ]]; then
        echo ""
        read -p "安装到 $target_dir? [Y/n]: " confirm
        if [[ "$confirm" =~ ^[Nn]$ ]]; then
            read -p "请输入安装路径: " target_dir
        fi
    fi
    
    # 检查是否已存在
    if [[ -d "$target_dir" ]]; then
        if [[ "$auto_confirm" != "yes" ]]; then
            print_warn "目录已存在: $target_dir"
            read -p "是否覆盖安装? [y/N]: " overwrite
            if [[ ! "$overwrite" =~ ^[Yy]$ ]]; then
                print_info "取消安装"
                exit 0
            fi
        fi
        # 备份配置
        if [[ -d "$target_dir/config" ]]; then
            local backup="$target_dir.config.backup.$(date +%Y%m%d%H%M%S)"
            cp -r "$target_dir/config" "$backup"
            print_ok "配置已备份到: $backup"
        fi
        rm -rf "$target_dir"
    fi
    
    # 创建临时目录
    local tmpdir=$(mktemp -d)
    trap "rm -rf $tmpdir" EXIT
    
    # 提取数据
    print_info "解压安装包..."
    
    # 从脚本中提取 tar.gz 数据（在 __ARCHIVE_MARKER__ 之后）
    local script_path="${BASH_SOURCE[0]}"
    local marker_line=$(grep -n "__ARCHIVE_MARKER__" "$script_path" | tail -1 | cut -d: -f1)
    local archive_start=$((marker_line + 1))
    
    tail -n +$archive_start "$script_path" | base64 -d | tar xzf - -C "$tmpdir"
    
    # 移动安装
    mkdir -p "$(dirname "$target_dir")"
    mv "$tmpdir"/claw-one-* "$target_dir"
    
    print_ok "文件已解压到: $target_dir"
    
    # 运行安装脚本
    if [[ -f "$target_dir/scripts/install.sh" ]]; then
        print_info "运行安装配置..."
        cd "$target_dir"
        bash scripts/install.sh
    else
        # 直接初始化
        print_info "初始化安装..."
        
        # 创建必要目录
        mkdir -p "$target_dir/config"
        mkdir -p "$target_dir/data"
        mkdir -p "$target_dir/logs"
        
        # 生成默认配置
        cat > "$target_dir/config/claw-one.toml" << EOF
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
        
        # 初始化 git
        if command -v git &> /dev/null; then
            cd "$target_dir/data"
            git init --quiet 2>/dev/null || true
            git config user.name "Claw One" 2>/dev/null || true
            git config user.email "dev@claw.one" 2>/dev/null || true
            echo "# Claw One Data" > README.md
            git add README.md 2>/dev/null || true
            git commit -m "Initial commit" --quiet 2>/dev/null || true
        fi
        
        # 创建快捷方式
        local local_bin="$HOME/.local/bin"
        if [[ -d "$local_bin" ]]; then
            ln -sf "$target_dir/bin/claw-one" "$local_bin/claw-one"
            print_ok "快捷方式已创建: ~/.local/bin/claw-one"
        fi
    fi
    
    print_ok "安装完成！"
    echo ""
    echo "安装位置: $target_dir"
    echo "启动命令: $target_dir/bin/claw-one run"
    echo "配置文件: $target_dir/config/claw-one.toml"
}

# 主函数
main() {
    local check_only="no"
    local auto_confirm="no"
    local target_dir="$INSTALL_DIR"
    
    # 解析参数
    while [[ $# -gt 0 ]]; do
        case "$1" in
            -h|--help)
                show_help
                exit 0
                ;;
            -v|--version)
                show_version
                exit 0
                ;;
            -c|--check)
                check_only="yes"
                shift
                ;;
            -y|--yes)
                auto_confirm="yes"
                shift
                ;;
            -t|--target)
                target_dir="$2"
                shift 2
                ;;
            *)
                print_error "未知选项: $1"
                show_help
                exit 1
                ;;
        esac
    done
    
    # 显示标题
    echo "========================================"
    echo "  Claw One 自解压安装脚本"
    echo "  Version: $VERSION"
    echo "  Arch: $ARCH"
    echo "========================================"
    echo ""
    
    # 检查环境
    if ! check_environment; then
        exit 1
    fi
    
    # 仅检查模式
    if [[ "$check_only" == "yes" ]]; then
        print_ok "环境检查完成"
        exit 0
    fi
    
    # 执行安装
    extract_and_install "$target_dir" "$auto_confirm"
}

# 运行主函数
main "$@"

exit 0

# 数据标记 - 之后的所有内容都是 base64 编码的 tar.gz
__ARCHIVE_MARKER__
