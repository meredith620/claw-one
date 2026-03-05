#!/bin/bash
# Claw One 卸载脚本
# 用法: ./uninstall.sh [安装路径前缀，默认 /usr/local]

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

PREFIX=${1:-/usr/local}
BINDIR="$PREFIX/bin"
DATADIR="$PREFIX/share/claw-one"
CONFIGDIR="/etc/claw-one"
SYSTEMD_DIR="/etc/systemd/system"

if [ "$EUID" -ne 0 ]; then 
    echo -e "${RED}错误: 请使用 sudo 运行卸载脚本${NC}"
    exit 1
fi

echo "========================================"
echo "  Claw One 卸载程序"
echo "========================================"
echo ""

# 确认卸载
read -p "确定要卸载 Claw One 吗? 数据将被保留。[y/N] " confirm
if [[ ! "$confirm" =~ ^[Yy]$ ]]; then
    echo "已取消卸载"
    exit 0
fi

# 停止并禁用服务
echo "[1/4] 停止服务..."
if systemctl is-active --quiet claw-one 2>/dev/null; then
    systemctl stop claw-one
    echo "  服务已停止"
fi
if systemctl is-enabled --quiet claw-one 2>/dev/null; then
    systemctl disable claw-one
    echo "  服务已禁用"
fi

# 删除文件
echo "[2/4] 删除程序文件..."
rm -f "$BINDIR/claw-one-backend"
rm -rf "$DATADIR"
rm -f "$SYSTEMD_DIR/claw-one.service"

# 询问是否删除配置
echo "[3/4] 处理配置文件..."
if [ -d "$CONFIGDIR" ]; then
    read -p "是否删除配置文件 ($CONFIGDIR)? [y/N] " remove_config
    if [[ "$remove_config" =~ ^[Yy]$ ]]; then
        rm -rf "$CONFIGDIR"
        echo "  配置文件已删除"
    else
        echo "  保留配置: $CONFIGDIR"
    fi
fi

# 询问是否删除用户数据
echo "[4/4] 处理用户数据..."
USER_DATA_DIR="$HOME/.config/claw-one"
if [ -d "$USER_DATA_DIR" ]; then
    read -p "是否删除用户数据 ($USER_DATA_DIR)? [y/N] " remove_data
    if [[ "$remove_data" =~ ^[Yy]$ ]]; then
        rm -rf "$USER_DATA_DIR"
        echo "  用户数据已删除"
    else
        echo "  保留数据: $USER_DATA_DIR"
    fi
fi

systemctl daemon-reload 2>/dev/null || true

echo ""
echo "========================================"
echo -e "${GREEN}  卸载完成!${NC}"
echo "========================================"
