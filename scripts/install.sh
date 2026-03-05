#!/bin/bash
# Claw One 用户级安装脚本
# 用法: ./install.sh [check|install|uninstall]

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 安装路径（可由环境变量覆盖）
INSTALL_DIR="${INSTALL_DIR:-$HOME/claw-one}"

# 显示帮助
show_help() {
    cat << EOF
Claw One 安装脚本

用法:
    $0 [命令]

命令:
    check       检测环境依赖
    install     安装 Claw One (默认)
    uninstall   卸载 Claw One
    help        显示帮助信息

环境变量:
    INSTALL_DIR    自定义安装路径 (默认: ~/claw-one)

示例:
    $0 check                    # 检查环境
    $0 install                  # 安装到 ~/claw-one
    INSTALL_DIR=/opt/claw $0    # 安装到自定义路径
EOF
}

# 检测环境
check_env() {
    echo "========================================"
    echo "  环境检测"
    echo "========================================"
    echo ""

    local has_error=0

    # 检查 Git
    echo -n "[1/5] 检查 Git ... "
    if command -v git &> /dev/null; then
        GIT_VERSION=$(git --version | awk '{print $3}')
        echo -e "${GREEN}✓${NC} (版本: $GIT_VERSION)"
    else
        echo -e "${RED}✗ 未找到${NC}"
        echo "      请安装 Git: sudo apt install git"
        has_error=1
    fi

    # 检查 systemd
    echo -n "[2/5] 检查 systemd ... "
    if command -v systemctl &> /dev/null; then
        echo -e "${GREEN}✓${NC}"
    else
        echo -e "${RED}✗ 未找到${NC}"
        echo "      需要 systemd 来管理服务"
        has_error=1
    fi

    # 检查 systemd user 模式
    echo -n "[3/5] 检查 systemd user 模式 ... "
    if systemctl --user status &> /dev/null; then
        echo -e "${GREEN}✓${NC}"
    else
        echo -e "${YELLOW}⚠ 可能不可用${NC}"
        echo "      尝试运行: systemctl --user status"
        echo "      如果失败，可能需要启用 linger: loginctl enable-linger $USER"
    fi

    # 检查必要的目录权限
    echo -n "[4/5] 检查安装目录权限 ... "
    if mkdir -p "$INSTALL_DIR" 2>/dev/null; then
        echo -e "${GREEN}✓${NC} ($INSTALL_DIR)"
    else
        echo -e "${RED}✗ 无法创建目录${NC}"
        has_error=1
    fi

    # 检查 systemd user 目录
    echo -n "[5/5] 检查 systemd 用户目录 ... "
    local systemd_dir="$HOME/.config/systemd/user"
    if mkdir -p "$systemd_dir" 2>/dev/null; then
        echo -e "${GREEN}✓${NC}"
    else
        echo -e "${RED}✗ 无法创建目录${NC}"
        has_error=1
    fi

    echo ""
    if [ $has_error -eq 0 ]; then
        echo -e "${GREEN}✓ 环境检测通过，可以安装${NC}"
        return 0
    else
        echo -e "${RED}✗ 环境检测失败，请修复上述问题后再安装${NC}"
        return 1
    fi
}

# 安装
do_install() {
    local BINDIR="$INSTALL_DIR/bin"
    local DATADIR="$INSTALL_DIR/share"
    local CONFIGDIR="$INSTALL_DIR/config"
    local SYSTEMD_USER_DIR="$HOME/.config/systemd/user"

    echo "========================================"
    echo "  Claw One 用户级安装程序"
    echo "========================================"
    echo ""
    echo "安装路径: $INSTALL_DIR"
    echo ""

    # 先检查环境
    if ! check_env; then
        exit 1
    fi
    echo ""

    # 创建目录
    echo "[安装] 创建目录结构..."
    mkdir -p "$BINDIR"
    mkdir -p "$DATADIR/static"
    mkdir -p "$CONFIGDIR"
    mkdir -p "$SYSTEMD_USER_DIR"

    # 安装二进制文件
    echo "[安装] 安装后端程序..."
    if [ -f "bin/claw-one-backend" ]; then
        cp bin/claw-one-backend "$BINDIR/"
        chmod +x "$BINDIR/claw-one-backend"
    else
        echo -e "${RED}错误: 未找到 claw-one-backend 二进制文件${NC}"
        exit 1
    fi

    # 安装静态文件
    echo "[安装] 安装前端资源..."
    if [ -d "share/static" ]; then
        cp -r share/static/* "$DATADIR/static/"
    else
        echo -e "${YELLOW}警告: 未找到前端静态文件${NC}"
    fi

    # 安装配置文件
    echo "[安装] 安装配置文件..."
    if [ -f "config/claw-one.toml.example" ]; then
        if [ ! -f "$CONFIGDIR/claw-one.toml" ]; then
            cp config/claw-one.toml.example "$CONFIGDIR/claw-one.toml"
            echo -e "${GREEN}✅ 已创建默认配置: $CONFIGDIR/claw-one.toml${NC}"
        else
            echo -e "${YELLOW}ℹ️ 配置文件已存在，保留现有配置${NC}"
        fi
    else
        echo -e "${YELLOW}警告: 未找到配置模板${NC}"
    fi

    # 安装 systemd 用户服务
    if [ -f "scripts/claw-one.user.service" ]; then
        sed -e "s|{INSTALL_DIR}|$INSTALL_DIR|g" \
            -e "s|{HOME}|$HOME|g" \
            scripts/claw-one.user.service > "$SYSTEMD_USER_DIR/claw-one.service"
        echo "  已安装用户服务"
    fi

    echo ""
    echo "========================================"
    echo -e "${GREEN}  安装完成!${NC}"
    echo "========================================"
    echo ""
    echo "文件位置:"
    echo "  二进制:   $BINDIR/claw-one-backend"
    echo "  静态文件: $DATADIR/static/"
    echo "  配置:     $CONFIGDIR/claw-one.toml"
    echo "  服务:     $SYSTEMD_USER_DIR/claw-one.service"
    echo ""
    echo -e "${YELLOW}⚠️ 重要: 请先编辑配置文件${NC}"
    echo "  nano $CONFIGDIR/claw-one.toml"
    echo ""
    echo "配置说明:"
    echo "  - openclaw_home: OpenClaw 安装根目录 (默认: ~/.openclaw)"
    echo "  - service_name:  OpenClaw systemd 服务名 (默认: openclaw)"
    echo ""
    echo "首次启动:"
    echo "  1. 编辑配置: nano $CONFIGDIR/claw-one.toml"
    echo "  2. 重载服务: systemctl --user daemon-reload"
    echo "  3. 设置开机自启: systemctl --user enable claw-one"
    echo "  4. 启动服务: systemctl --user start claw-one"
    echo "  5. 查看状态: systemctl --user status claw-one"
    echo ""
    echo "访问界面: http://localhost:8080"
    echo ""
}

# 卸载
do_uninstall() {
    local BINDIR="$INSTALL_DIR/bin"
    local DATADIR="$INSTALL_DIR/share"
    local CONFIGDIR="$INSTALL_DIR/config"
    local SYSTEMD_USER_DIR="$HOME/.config/systemd/user"

    echo "========================================"
    echo "  Claw One 用户级卸载程序"
    echo "========================================"
    echo ""
    echo "卸载路径: $INSTALL_DIR"
    echo ""

    # 确认卸载
    read -p "确定要卸载 Claw One 吗? [y/N] " confirm
    if [[ ! "$confirm" =~ ^[Yy]$ ]]; then
        echo "已取消卸载"
        exit 0
    fi

    # 停止并禁用服务
    echo "[卸载] 停止服务..."
    if systemctl --user is-active --quiet claw-one 2>/dev/null; then
        systemctl --user stop claw-one
        echo "  服务已停止"
    fi
    if systemctl --user is-enabled --quiet claw-one 2>/dev/null; then
        systemctl --user disable claw-one
        echo "  服务已禁用"
    fi

    # 删除程序文件
    echo "[卸载] 删除程序文件..."
    rm -rf "$BINDIR"
    rm -rf "$DATADIR"
    rm -f "$SYSTEMD_USER_DIR/claw-one.service"

    # 询问是否删除配置
    echo "[卸载] 处理配置文件..."
    if [ -d "$CONFIGDIR" ]; then
        echo -e "${YELLOW}配置文件目录: $CONFIGDIR${NC}"
        read -p "是否删除配置文件? [y/N] " remove_config
        if [[ "$remove_config" =~ ^[Yy]$ ]]; then
            rm -rf "$CONFIGDIR"
            echo "  配置文件已删除"
        else
            echo "  保留配置: $CONFIGDIR"
        fi
    fi

    # 尝试删除安装目录（如果为空）
    if [ -d "$INSTALL_DIR" ] && [ -z "$(ls -A "$INSTALL_DIR" 2>/dev/null)" ]; then
        rmdir "$INSTALL_DIR"
        echo "  已删除空安装目录"
    fi

    systemctl --user daemon-reload 2>/dev/null || true

    echo ""
    echo "========================================"
    echo -e "${GREEN}  卸载完成!${NC}"
    echo "========================================"
}

# 主逻辑
case "${1:-install}" in
    check)
        check_env
        ;;
    install|""|-i)
        do_install
        ;;
    uninstall|-u)
        do_uninstall
        ;;
    help|-h|--help)
        show_help
        ;;
    *)
        echo -e "${RED}错误: 未知命令 '$1'${NC}"
        show_help
        exit 1
        ;;
esac
