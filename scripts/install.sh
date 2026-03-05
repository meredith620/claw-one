#!/bin/bash
# Claw One 安装脚本
# 用法: ./install.sh [安装路径前缀，默认 /usr/local]

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# 安装路径
PREFIX=${1:-/usr/local}
BINDIR="$PREFIX/bin"
DATADIR="$PREFIX/share/claw-one"
CONFIGDIR="/etc/claw-one"
SYSTEMD_DIR="/etc/systemd/system"

# 检查 root 权限
if [ "$EUID" -ne 0 ]; then 
    echo -e "${RED}错误: 请使用 sudo 运行安装脚本${NC}"
    exit 1
fi

echo "========================================"
echo "  Claw One 安装程序"
echo "========================================"
echo ""

# 检查依赖
echo "[1/6] 检查依赖..."
if ! command -v systemctl &> /dev/null; then
    echo -e "${YELLOW}警告: 未检测到 systemd，服务管理功能将不可用${NC}"
fi

# 创建目录
echo "[2/6] 创建目录..."
mkdir -p "$BINDIR"
mkdir -p "$DATADIR"
mkdir -p "$CONFIGDIR"
mkdir -p "$SYSTEMD_DIR"

# 安装二进制文件
echo "[3/6] 安装后端程序..."
if [ -f "bin/claw-one-backend" ]; then
    install -Dm755 bin/claw-one-backend "$BINDIR/claw-one-backend"
else
    echo -e "${RED}错误: 未找到 claw-one-backend 二进制文件${NC}"
    echo "请先运行 'make compile' 编译项目"
    exit 1
fi

# 安装静态文件
echo "[4/6] 安装前端资源..."
if [ -d "share/static" ]; then
    cp -r share/static/* "$DATADIR/"
else
    echo -e "${YELLOW}警告: 未找到前端静态文件${NC}"
fi

# 安装配置文件
echo "[5/6] 安装配置文件..."
if [ -f "config/openclaw.json.example" ]; then
    if [ ! -f "$CONFIGDIR/openclaw.json" ]; then
        cp config/openclaw.json.example "$CONFIGDIR/openclaw.json"
        echo "  已创建默认配置: $CONFIGDIR/openclaw.json"
    else
        echo "  配置已存在，保留现有配置"
    fi
fi

# 安装 systemd 服务
echo "[6/6] 安装 systemd 服务..."
if [ -f "scripts/claw-one.service" ]; then
    cp scripts/claw-one.service "$SYSTEMD_DIR/"
    systemctl daemon-reload
    echo "  服务已安装，可用命令:"
    echo "    systemctl start claw-one    # 启动服务"
    echo "    systemctl enable claw-one   # 开机自启"
else
    echo -e "${YELLOW}警告: 未找到服务文件${NC}"
fi

echo ""
echo "========================================"
echo -e "${GREEN}  安装完成!${NC}"
echo "========================================"
echo ""
echo "文件位置:"
echo "  二进制:   $BINDIR/claw-one-backend"
echo "  静态文件: $DATADIR/"
echo "  配置:     $CONFIGDIR/openclaw.json"
echo "  服务:     $SYSTEMD_DIR/claw-one.service"
echo ""
echo "首次使用:"
echo "  1. 编辑配置文件: sudo nano $CONFIGDIR/openclaw.json"
echo "  2. 启动服务:     sudo systemctl start claw-one"
echo "  3. 查看状态:     sudo systemctl status claw-one"
echo "  4. 访问界面:     http://localhost:8080"
echo ""
echo "数据目录: ~/.config/claw-one/"
echo ""
