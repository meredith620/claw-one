#!/bin/bash
# 自解压安装脚本头部
# 将 tar.gz 附加到此脚本后生成自解压安装包

set -e

# 提取并执行安装
SELF="$0"
TARGET_DIR=$(mktemp -d)

echo "Claw One 自解压安装程序"
echo ""

# 解压 payload
echo "正在解压..."
tail -n +$(awk '/^#__PAYLOAD_BELOW__/ {print NR + 1; exit 0; }' "$SELF") "$SELF" | tar -xzf - -C "$TARGET_DIR"

# 执行安装
cd "$TARGET_DIR"
if [ -f "install/install.sh" ]; then
    sudo bash install/install.sh "$@"
else
    echo "错误: 安装脚本不存在"
    exit 1
fi

# 清理
rm -rf "$TARGET_DIR"

exit 0

#__PAYLOAD_BELOW__
