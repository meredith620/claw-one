#!/bin/bash
#
# Claw One 用户级安装脚本
# 安装到 ~/claw-one/，无需 root 权限

set -e

CLAW_ONE_VERSION="0.1.0"
INSTALL_DIR="$HOME/claw-one"
BIN_DIR="$INSTALL_DIR/bin"
CONFIG_DIR="$INSTALL_DIR/config"
DATA_DIR="$INSTALL_DIR/data"
LOGS_DIR="$INSTALL_DIR/logs"
SHARE_DIR="$INSTALL_DIR/share"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

print_header() {
    echo "========================================"
    echo "  Claw One 安装脚本"
    echo "  Version: $CLAW_ONE_VERSION"
    echo "========================================"
    echo ""
}

print_info() {
    echo -e "${BLUE}ℹ${NC} $1"
}

print_ok() {
    echo -e "${GREEN}✓${NC} $1"
}

print_warn() {
    echo -e "${YELLOW}⚠${NC} $1"
}

print_error() {
    echo -e "${RED}✗${NC} $1"
}

# 检查是否已安装
check_existing() {
    if [ -d "$INSTALL_DIR" ]; then
        print_warn "检测到已有安装: $INSTALL_DIR"
        echo ""
        echo "选项:"
        echo "  1) 覆盖安装（保留数据）"
        echo "  2) 备份并重新安装"
        echo "  3) 取消安装"
        echo ""
        read -p "请选择 [1/2/3]: " choice
        
        case "$choice" in
            1)
                print_info "覆盖安装..."
                BACKUP_DIR="$INSTALL_DIR.backup.$(date +%Y%m%d%H%M%S)"
                mv "$INSTALL_DIR/config" "$BACKUP_DIR/config" 2>/dev/null || true
                print_ok "配置已备份到: $BACKUP_DIR/config"
                rm -rf "$INSTALL_DIR"
                ;;
            2)
                BACKUP_DIR="$INSTALL_DIR.backup.$(date +%Y%m%d%H%M%S)"
                mv "$INSTALL_DIR" "$BACKUP_DIR"
                print_ok "已备份到: $BACKUP_DIR"
                ;;
            3)
                print_info "取消安装"
                exit 0
                ;;
            *)
                print_error "无效选择"
                exit 1
                ;;
        esac
    fi
}

# 创建目录结构
create_directories() {
    print_info "创建目录结构..."
    
    mkdir -p "$BIN_DIR"
    mkdir -p "$CONFIG_DIR"
    mkdir -p "$DATA_DIR"
    mkdir -p "$LOGS_DIR"
    mkdir -p "$SHARE_DIR"/{static,config}
    
    print_ok "目录结构创建完成"
}

# 复制文件
copy_files() {
    print_info "复制文件..."
    
    # 复制后端二进制
    if [ -f "$SCRIPT_DIR/../bin/claw-one-backend" ]; then
        cp "$SCRIPT_DIR/../bin/claw-one-backend" "$BIN_DIR/"
        chmod +x "$BIN_DIR/claw-one-backend"
        print_ok "后端程序已复制"
    else
        print_error "未找到后端程序: bin/claw-one-backend"
        exit 1
    fi
    
    # 复制配置向导
    if [ -f "$SCRIPT_DIR/../bin/setup-config.sh" ]; then
        cp "$SCRIPT_DIR/../bin/setup-config.sh" "$BIN_DIR/"
        chmod +x "$BIN_DIR/setup-config.sh"
        print_ok "配置向导已复制"
    fi
    
    # 复制静态文件
    if [ -d "$SCRIPT_DIR/../share/static" ]; then
        cp -r "$SCRIPT_DIR/../share/static/"* "$SHARE_DIR/static/"
        print_ok "静态文件已复制"
    else
        print_warn "未找到静态文件"
    fi
    
    # 复制配置模板
    if [ -d "$SCRIPT_DIR/../share/config" ]; then
        cp "$SCRIPT_DIR/../share/config/"*.template "$SHARE_DIR/config/"
        print_ok "配置模板已复制"
    fi
    
    # 复制卸载脚本
    cp "$SCRIPT_DIR/uninstall.sh" "$INSTALL_DIR/"
    chmod +x "$INSTALL_DIR/uninstall.sh"
    print_ok "卸载脚本已复制"
}

# 初始化默认配置
init_config() {
    print_info "初始化配置..."
    
    # 从模板生成配置
    if [ -f "$SHARE_DIR/config/claw-one.toml.template" ]; then
        # 替换模板变量
        sed "s|{{HOME}}|$HOME|g" "$SHARE_DIR/config/claw-one.toml.template" \
            | sed "s|{{VERSION}}|$CLAW_ONE_VERSION|g" \
            > "$CONFIG_DIR/claw-one.toml"
        print_ok "主配置已生成: ~/claw-one/config/claw-one.toml"
    fi
    
    # OpenClaw 配置
    if [ -f "$SHARE_DIR/config/openclaw.json.template" ]; then
        cp "$SHARE_DIR/config/openclaw.json.template" "$CONFIG_DIR/openclaw.json"
        print_ok "OpenClaw 配置已生成: ~/claw-one/config/openclaw.json"
    fi
}

# 初始化 Git 仓库
init_git_repo() {
    print_info "初始化数据目录..."
    
    cd "$DATA_DIR"
    git init --quiet
    git config user.name "Claw One"
    git config user.email "dev@claw.one"
    
    # 创建初始提交
    echo "# Claw One Data Directory" > README.md
    git add README.md
    git commit -m "Initial commit" --quiet || true
    
    print_ok "Git 仓库已初始化"
}

# 创建快捷方式
create_symlink() {
    print_info "创建命令行快捷方式..."
    
    LOCAL_BIN="$HOME/.local/bin"
    
    if [ -d "$LOCAL_BIN" ]; then
        ln -sf "$BIN_DIR/claw-one-backend" "$LOCAL_BIN/claw-one"
        print_ok "快捷方式已创建: ~/.local/bin/claw-one"
        print_info "请确保 ~/.local/bin 在 PATH 中"
    else
        print_warn "未找到 ~/.local/bin，跳过创建快捷方式"
        print_info "可以手动创建: ln -s $BIN_DIR/claw-one-backend ~/.local/bin/claw-one"
    fi
}

# 打印安装完成信息
print_completion() {
    echo ""
    echo "========================================"
    echo -e "${GREEN}    安装完成！${NC}"
    echo "========================================"
    echo ""
    echo "安装位置: $INSTALL_DIR"
    echo "配置文件: $CONFIG_DIR/claw-one.toml"
    echo "数据目录: $DATA_DIR"
    echo "日志目录: $LOGS_DIR"
    echo ""
    echo "下一步:"
    echo ""
    echo "  1. 运行配置向导:"
    echo "     $BIN_DIR/setup-config.sh"
    echo ""
    echo "  2. 手动启动服务:"
    echo "     $BIN_DIR/claw-one-backend run"
    echo ""
    echo "  3. 或使用 systemd 用户服务:"
    echo "     systemctl --user start claw-one"
    echo ""
    echo "  4. 访问配置界面:"
    echo "     http://localhost:8080"
    echo ""
    echo "卸载命令:"
    echo "  $INSTALL_DIR/uninstall.sh"
    echo ""
}

# 主流程
main() {
    print_header
    check_existing
    create_directories
    copy_files
    init_config
    init_git_repo
    create_symlink
    print_completion
}

main "$@"
