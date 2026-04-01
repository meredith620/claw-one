#!/bin/bash
# HARNESS METADATA
# type: script
# part-of: harness-architecture
# scope: discovery
# managed-by: harness-system
# do-not-modify-manually: 如需修改请遵循 harness-evolution.spec.md
# version: 1.0
# created: 2026-04-01
#
# Harness 发现脚本
# 列出所有可用的 specs 和 scripts

echo "📋 Claw One Harness 文档清单"
echo "=============================="
echo ""

echo "📖 规范文档 (.harness/specs/):"
if [ -d ".harness/specs" ]; then
    for file in .harness/specs/*.md; do
        [ -f "$file" ] || continue
        name=$(basename "$file" .md)
        
        # 提取第一行标题
        title=$(head -1 "$file" | sed 's/^# //' 2>/dev/null || echo "$name")
        
        # 提取适用范围
        scope=$(grep -m1 "适用于:" "$file" 2>/dev/null | sed 's/.*适用于: *//' || echo "通用")
        
        printf "  • %-25s %s\n" "$name" "[$scope]"
    done
else
    echo "  (specs/ 目录不存在)"
fi

echo ""
echo "🔧 脚本工具 (.harness/scripts/):"
if [ -d ".harness/scripts" ]; then
    for script in .harness/scripts/*.sh; do
        [ -f "$script" ] || continue
        name=$(basename "$script")
        
        # 提取描述
        # 跳过 shebang、HARNESS METADATA 头部的 key-value 行，找到第一个短描述行
        desc=$(awk '
            BEGIN { in_meta=0 }
            NR==1 && /^#!/ { next }  # 跳过 shebang
            /^# HARNESS METADATA/ { in_meta=1; next }
            in_meta && /^# [a-z-]+:/ { next }  # 跳过 key-value 行
            in_meta && /^#$/ { in_meta=0; next }  # 空行结束 metadata
            in_meta && /^# --/ { in_meta=0; next }  # 分隔行结束 metadata
            in_meta { next }  # 跳过其他 metadata 行
            /^# [^#]/ { sub(/^# /, ""); print; exit }  # 第一行描述
            /^#$/ { next }  # 跳过空注释行
        ' "$script" 2>/dev/null || echo "$name")
        
        printf "  • %-25s %s\n" "$name" "$desc"
    done
else
    echo "  (scripts/harness/ 目录不存在)"
fi

echo ""
echo "🛡️ 熵防护规则 (.harness/guards/):"
if [ -d ".harness/guards" ]; then
    for rule in .harness/guards/*.rule; do
        [ -f "$rule" ] || continue
        name=$(basename "$rule")
        printf "  • %s\n" "$name"
    done
else
    echo "  (.entropy-guards/ 目录不存在)"
fi

echo ""
echo "💡 使用说明:"
echo "  - 根据任务类型选择对应 spec"
echo "  - 运行 .harness/scripts/pre-commit.sh 提交前检查"
echo "  - 配置变更前运行 .harness/scripts/pre-config-change.sh"
