#!/bin/bash
# 自解压安装脚本头部
# 将 tar.gz 附加到此脚本后生成自解压安装包

set -e

SELF="$0"
TARGET_DIR=$(mktemp -d)

echo "========================================"
echo "  Claw One 自解压安装程序"
echo "========================================"
echo ""

# 默认安装到 ~/claw-one
INSTALL_DIR="${INSTALL_DIR:-$HOME/claw-one}"

# 解压 payload
echo "正在解压..."
tail -n +20 "$SELF" | tar -xzf - -C "$TARGET_DIR"

# 执行安装
cd "$TARGET_DIR"
if [ -f "install/install.sh" ]; then
    export INSTALL_DIR
    bash install/install.sh
else
    echo "错误: 安装脚本不存在"
    rm -rf "$TARGET_DIR"
    exit 1
fi

# 清理
rm -rf "$TARGET_DIR"

exit 0

# ========================================
# 以下二进制数据为 tar.gz 归档
# ========================================
