#!/bin/bash
# Claw One 用户级安装脚本
# 默认安装到 ~/claw-one

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 安装路径（可由环境变量覆盖）
INSTALL_DIR="${INSTALL_DIR:-$HOME/claw-one}"
BINDIR="$INSTALL_DIR/bin"
DATADIR="$INSTALL_DIR/share"
CONFIGDIR="$INSTALL_DIR/config"
SYSTEMD_USER_DIR="$HOME/.config/systemd/user"

echo "========================================"
echo "  Claw One 用户级安装程序"
echo "========================================"
echo ""
echo "安装路径: $INSTALL_DIR"
echo ""

# 检查依赖
echo "[1/5] 检查依赖..."
if ! command -v systemctl &> /dev/null; then
    echo -e "${YELLOW}警告: 未检测到 systemd，开机自启功能将不可用${NC}"
fi

# 创建目录
echo "[2/5] 创建目录结构..."
mkdir -p "$BINDIR"
mkdir -p "$DATADIR/static"
mkdir -p "$CONFIGDIR"
mkdir -p "$SYSTEMD_USER_DIR"

# 安装二进制文件
echo "[3/5] 安装后端程序..."
if [ -f "bin/claw-one-backend" ]; then
    cp bin/claw-one-backend "$BINDIR/"
    chmod +x "$BINDIR/claw-one-backend"
else
    echo -e "${RED}错误: 未找到 claw-one-backend 二进制文件${NC}"
    exit 1
fi

# 安装静态文件
echo "[4/5] 安装前端资源..."
if [ -d "share/static" ]; then
    cp -r share/static/* "$DATADIR/static/"
else
    echo -e "${YELLOW}警告: 未找到前端静态文件${NC}"
fi

# 安装配置文件
echo "[5/5] 安装配置文件..."
if [ -f "config/claw-one.toml.example" ]; then
    if [ ! -f "$CONFIGDIR/claw-one.toml" ]; then
        cp config/claw-one.toml.example "$CONFIGDIR/claw-one.toml"
        echo -e "${GREEN}✅ 已创建默认配置: $CONFIGDIR/claw-one.toml${NC}"
    else
        echo -e "${YELLOW}ℹ️  配置文件已存在，保留现有配置${NC}"
    fi
else
    echo -e "${YELLOW}警告: 未找到配置模板${NC}"
fi

# 安装 systemd 用户服务
if [ -f "scripts/claw-one.user.service" ]; then
    sed -e "s|{INSTALL_DIR}|$INSTALL_DIR|g" \
        -e "s|{HOME}|$HOME|g" \
        scripts/claw-one.user.service > "$SYSTEMD_USER_DIR/claw-one.service"
    echo "  已安装用户服务: $SYSTEMD_USER_DIR/claw-one.service"
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
echo -e "${YELLOW}⚠️  重要: 请先编辑配置文件${NC}"
echo "  nano $CONFIGDIR/claw-one.toml"
echo ""
echo "配置说明:"
echo "  - 设置要管理的 OpenClaw 服务名称"
echo "  - 配置 OpenClaw 健康检查端口"
echo "  - 指定 OpenClaw 配置文件路径"
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
