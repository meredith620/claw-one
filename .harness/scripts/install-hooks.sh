#!/bin/bash
# HARNESS METADATA
# type: script
# part-of: harness-architecture
# scope: git-integration
# managed-by: harness-system
# do-not-modify-manually: 如需修改请遵循 harness-evolution.spec.md
# version: 1.0
# created: 2026-04-01
#
# Git Hook 安装脚本
# 将 Harness 脚本链接为 git hooks

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}🪝  Git Hook 安装脚本${NC}"
echo "======================"
echo ""

# 检查是否在 git 仓库中
if [ ! -d ".git" ]; then
    echo -e "${RED}❌ 错误: 当前目录不是 Git 仓库${NC}"
    exit 1
fi

# 确保 hooks 目录存在
mkdir -p .git/hooks

# 需要链接的 hooks
declare -A HOOKS
HOOKS["pre-commit"]=".harness/scripts/pre-commit.sh"
HOOKS["prepare-commit-msg"]=".harness/scripts/prepare-commit-msg.sh"

# 检查脚本是否存在
for hook_script in "${HOOKS[@]}"; do
    if [ ! -f "$hook_script" ]; then
        echo -e "${RED}❌ 错误: $hook_script 不存在${NC}"
        exit 1
    fi
done

echo -e "${YELLOW}安装 Git Hooks...${NC}"
echo ""

# 链接 pre-commit hook
if [ -f ".git/hooks/pre-commit" ]; then
    if grep -q "harness" .git/hooks/pre-commit 2>/dev/null; then
        echo -e "${GREEN}✓ pre-commit hook 已存在且为 Harness hook${NC}"
    else
        echo -e "${YELLOW}⚠️  pre-commit hook 已存在，备份为 pre-commit.orig${NC}"
        mv .git/hooks/pre-commit .git/hooks/pre-commit.orig
    fi
fi

echo -e "链接 .harness/scripts/pre-commit.sh → .git/hooks/pre-commit"
ln -sf ../../.harness/scripts/pre-commit.sh .git/hooks/pre-commit
chmod +x .git/hooks/pre-commit

# 可选：prepare-commit-msg hook（如果存在）
if [ -f ".harness/scripts/prepare-commit-msg.sh" ]; then
    echo -e "链接 .harness/scripts/prepare-commit-msg.sh → .git/hooks/prepare-commit-msg"
    ln -sf ../../.harness/scripts/prepare-commit-msg.sh .git/hooks/prepare-commit-msg
    chmod +x .git/hooks/prepare-commit-msg
fi

echo ""
echo -e "${GREEN}✅ Git Hooks 安装完成！${NC}"
echo ""

# 验证安装
echo "验证安装..."
if [ -L .git/hooks/pre-commit ]; then
    target=$(readlink .git/hooks/pre-commit)
    echo -e "  ${GREEN}✓${NC} pre-commit → $target"
else
    echo -e "  ${RED}✗${NC} pre-commit 链接失败"
fi

echo ""
echo -e "${BLUE}💡 提示:${NC}"
echo "  - 提交时会自动运行 pre-commit.sh 中的检查"
echo "  - 如需卸载，运行: rm .git/hooks/pre-commit"
echo "  - 如需临时跳过 hook，运行: git commit --no-verify"
echo ""

# 询问是否运行验证
read -p "是否现在运行架构验证? (y/n): " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    ./.harness/scripts/validate-arch.sh
fi
