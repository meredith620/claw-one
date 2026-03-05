#!/bin/bash
# Claw One 用户级卸载脚本

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# 安装路径（可由环境变量覆盖）
INSTALL_DIR="${INSTALL_DIR:-$HOME/claw-one}"
BINDIR="$INSTALL_DIR/bin"
DATADIR="$INSTALL_DIR/share"
CONFIGDIR="$INSTALL_DIR/config"
SYSTEMD_USER_DIR="$HOME/.config/systemd/user"

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
echo "[1/3] 停止服务..."
if systemctl --user is-active --quiet claw-one 2>/dev/null; then
    systemctl --user stop claw-one
    echo "  服务已停止"
fi
if systemctl --user is-enabled --quiet claw-one 2>/dev/null; then
    systemctl --user disable claw-one
    echo "  服务已禁用"
fi

# 删除程序文件
echo "[2/3] 删除程序文件..."
rm -rf "$BINDIR"
rm -rf "$DATADIR"
rm -f "$SYSTEMD_USER_DIR/claw-one.service"

# 询问是否删除配置
echo "[3/3] 处理配置文件..."
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

systemctl --user daemon-reload 2>/dev/null || true

echo ""
echo "========================================"
echo -e "${GREEN}  卸载完成!${NC}"
echo "========================================"
