#!/bin/bash
# 容器内测试脚本

set -e

CLAW_ONE_DIR="$HOME/claw-one"

echo "========================================"
echo "Claw One 安装后功能测试"
echo "========================================"
echo ""

# 1. 检查安装目录
echo "[1/8] 检查安装目录..."
if [ -d "$CLAW_ONE_DIR" ]; then
    echo "✅ 安装目录存在: $CLAW_ONE_DIR"
    ls -la "$CLAW_ONE_DIR"
else
    echo "❌ 安装目录不存在"
    exit 1
fi
echo ""

# 2. 检查二进制文件
echo "[2/8] 检查二进制文件..."
if [ -f "$CLAW_ONE_DIR/bin/claw-one" ]; then
    echo "✅ 主程序存在"
    "$CLAW_ONE_DIR/bin/claw-one" --version 2>/dev/null || echo "⚠️  无法获取版本"
else
    echo "❌ 主程序不存在"
    exit 1
fi
echo ""

# 3. 检查配置文件
echo "[3/8] 检查配置文件..."
if [ -d "$CLAW_ONE_DIR/config" ]; then
    echo "✅ 配置目录存在"
    ls -la "$CLAW_ONE_DIR/config"
else
    echo "⚠️  配置目录不存在"
fi
echo ""

# 4. 检查静态文件
echo "[4/8] 检查静态文件..."
if [ -d "$CLAW_ONE_DIR/share/static" ]; then
    echo "✅ 静态文件目录存在"
    if [ -f "$CLAW_ONE_DIR/share/static/index.html" ]; then
        echo "✅ index.html 存在"
    fi
else
    echo "⚠️  静态文件目录不存在"
fi
echo ""

# 5. 检查数据目录和 Git 仓库
echo "[5/8] 检查数据目录..."
if [ -d "$CLAW_ONE_DIR/data/.git" ]; then
    echo "✅ Git 仓库已初始化"
    cd "$CLAW_ONE_DIR/data" && git log --oneline -1
else
    echo "⚠️  Git 仓库未初始化"
fi
echo ""

# 6. 测试 CLI 配置查看
echo "[6/8] 测试 CLI 配置查看..."
if [ -f "$CLAW_ONE_DIR/config/claw-one.toml" ]; then
    export CLAW_ONE_CONFIG="$CLAW_ONE_DIR/config/claw-one.toml"
    "$CLAW_ONE_DIR/bin/claw-one" config 2>/dev/null || echo "⚠️  config 命令执行失败"
else
    echo "⚠️  配置文件不存在，跳过配置测试"
fi
echo ""

# 7. 检查卸载脚本
echo "[7/8] 检查卸载脚本..."
if [ -f "$CLAW_ONE_DIR/uninstall.sh" ]; then
    echo "✅ 卸载脚本存在"
else
    echo "⚠️  卸载脚本不存在"
fi
echo ""

# 8. 检查命令行快捷方式
echo "[8/8] 检查命令行快捷方式..."
if [ -L "$HOME/.local/bin/claw-one" ]; then
    echo "✅ 快捷方式存在: ~/.local/bin/claw-one"
    readlink "$HOME/.local/bin/claw-one"
else
    echo "⚠️  快捷方式不存在"
fi
echo ""

echo "========================================"
echo "测试完成"
echo "========================================"
