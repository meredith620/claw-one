#!/bin/bash
# Claw One 卸载脚本 (简化版)
# 建议使用: ./install.sh uninstall
# 此脚本用于独立分发时卸载

set -e

INSTALL_DIR="${INSTALL_DIR:-$HOME/claw-one}"

echo "========================================"
echo "  Claw One 卸载程序"
echo "========================================"
echo ""
echo "提示: 推荐使用 './install.sh uninstall' 命令"
echo "安装路径: $INSTALL_DIR"
echo ""

# 调用 install.sh 的卸载功能，或直接执行卸载
if [ -f "install/install.sh" ]; then
    bash install/install.sh uninstall
else
    echo "未找到完整安装脚本，执行简单卸载..."
    
    # 停止服务
    systemctl --user stop claw-one 2>/dev/null || true
    systemctl --user disable claw-one 2>/dev/null || true
    
    # 删除文件
    rm -rf "$INSTALL_DIR/bin"
    rm -rf "$INSTALL_DIR/share"
    rm -f "$HOME/.config/systemd/user/claw-one.service"
    
    # 保留配置，提示用户手动删除
    if [ -d "$INSTALL_DIR/config" ]; then
        echo ""
        echo "配置文件保留在: $INSTALL_DIR/config"
        echo "如需完全删除，请手动执行: rm -rf $INSTALL_DIR"
    fi
    
    systemctl --user daemon-reload 2>/dev/null || true
    
    echo ""
    echo "========================================"
    echo "  卸载完成"
    echo "========================================"
fi
