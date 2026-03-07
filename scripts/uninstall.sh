#!/bin/bash
#
# Claw One 卸载脚本
# 完全清理用户级安装

set -e

INSTALL_DIR="$HOME/claw-one"
SYSTEMD_USER_DIR="$HOME/.config/systemd/user"

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

print_header() {
    echo "========================================"
    echo "  Claw One 卸载脚本"
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

# 停止服务
stop_service() {
    print_info "检查并停止服务..."
    
    # 停止 systemd 服务
    if [ -f "$SYSTEMD_USER_DIR/claw-one.service" ]; then
        if systemctl --user is-active --quiet claw-one 2>/dev/null; then
            systemctl --user stop claw-one
            print_ok "systemd 服务已停止"
        fi
        
        # 禁用自启
        if systemctl --user is-enabled --quiet claw-one 2>/dev/null; then
            systemctl --user disable claw-one
            print_ok "systemd 服务已禁用"
        fi
    fi
    
    # 查找并终止手动启动的进程
    PID=$(pgrep -f "claw-one" || true)
    if [ -n "$PID" ]; then
        print_warn "发现运行中的进程 (PID: $PID)"
        read -p "是否终止该进程? [Y/n]: " confirm
        if [[ ! "$confirm" =~ ^[Nn]$ ]]; then
            kill $PID 2>/dev/null || true
            sleep 1
            # 强制终止如果还在运行
            if kill -0 $PID 2>/dev/null; then
                kill -9 $PID 2>/dev/null || true
            fi
            print_ok "进程已终止"
        fi
    fi
}

# 删除 systemd 服务文件
remove_systemd_service() {
    if [ -f "$SYSTEMD_USER_DIR/claw-one.service" ]; then
        rm -f "$SYSTEMD_USER_DIR/claw-one.service"
        systemctl --user daemon-reload 2>/dev/null || true
        print_ok "systemd 服务文件已删除"
    fi
}

# 删除快捷方式
remove_symlink() {
    LOCAL_BIN="$HOME/.local/bin/claw-one"
    if [ -L "$LOCAL_BIN" ]; then
        rm -f "$LOCAL_BIN"
        print_ok "命令行快捷方式已删除"
    fi
}

# 询问保留数据
ask_keep_data() {
    echo ""
    echo "是否保留以下数据？"
    echo ""
    
    # 配置
    if [ -d "$INSTALL_DIR/config" ]; then
        read -p "保留配置文件? [y/N]: " keep_config
        if [[ "$keep_config" =~ ^[Yy]$ ]]; then
            BACKUP_DIR="$HOME/.claw-one-backup.$(date +%Y%m%d%H%M%S)"
            mkdir -p "$BACKUP_DIR"
            cp -r "$INSTALL_DIR/config" "$BACKUP_DIR/"
            print_ok "配置已备份到: $BACKUP_DIR/config"
        fi
    fi
    
    # 数据（Git 历史）
    if [ -d "$INSTALL_DIR/data/.git" ]; then
        echo ""
        read -p "保留数据目录（包含 Git 版本历史）? [y/N]: " keep_data
        if [[ "$keep_data" =~ ^[Yy]$ ]]; then
            BACKUP_DIR="${BACKUP_DIR:-$HOME/.claw-one-backup.$(date +%Y%m%d%H%M%S)}"
            mkdir -p "$BACKUP_DIR"
            cp -r "$INSTALL_DIR/data" "$BACKUP_DIR/"
            print_ok "数据已备份到: $BACKUP_DIR/data"
        fi
    fi
    
    # 日志
    if [ -d "$INSTALL_DIR/logs" ] && [ "$(ls -A $INSTALL_DIR/logs 2>/dev/null)" ]; then
        echo ""
        read -p "保留日志文件? [y/N]: " keep_logs
        if [[ "$keep_logs" =~ ^[Yy]$ ]]; then
            BACKUP_DIR="${BACKUP_DIR:-$HOME/.claw-one-backup.$(date +%Y%m%d%H%M%S)}"
            mkdir -p "$BACKUP_DIR"
            cp -r "$INSTALL_DIR/logs" "$BACKUP_DIR/"
            print_ok "日志已备份到: $BACKUP_DIR/logs"
        fi
    fi
}

# 删除安装目录
remove_installation() {
    print_info "删除安装目录..."
    
    if [ -d "$INSTALL_DIR" ]; then
        rm -rf "$INSTALL_DIR"
        print_ok "安装目录已删除: $INSTALL_DIR"
    else
        print_warn "未找到安装目录"
    fi
}

# 打印完成信息
print_completion() {
    echo ""
    echo "========================================"
    echo -e "${GREEN}    卸载完成${NC}"
    echo "========================================"
    echo ""
    
    if [ -n "$BACKUP_DIR" ]; then
        echo "备份位置: $BACKUP_DIR"
        echo ""
    fi
    
    echo "如需重新安装，请运行:"
    echo "  ./install.sh"
    echo ""
}

# 确认卸载
confirm_uninstall() {
    echo ""
    echo -e "${RED}⚠️  警告: 此操作将删除 Claw One${NC}"
    echo ""
    echo "将删除以下内容:"
    echo "  - 程序文件: $INSTALL_DIR"
    echo "  - systemd 服务"
    echo "  - 命令行快捷方式"
    echo ""
    
    read -p "确认卸载? [y/N]: " confirm
    
    if [[ ! "$confirm" =~ ^[Yy]$ ]]; then
        echo "取消卸载"
        exit 0
    fi
}

# 主流程
main() {
    print_header
    
    if [ ! -d "$INSTALL_DIR" ]; then
        print_error "未检测到 Claw One 安装"
        exit 1
    fi
    
    confirm_uninstall
    stop_service
    remove_systemd_service
    ask_keep_data
    remove_symlink
    remove_installation
    print_completion
}

main "$@"
