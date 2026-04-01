#!/bin/bash
# HARNESS METADATA
# type: script
# part-of: harness-architecture
# scope: git-workflow
# managed-by: harness-system
# do-not-modify-manually: 如需修改请遵循 harness-evolution.spec.md
# version: 1.0
# created: 2026-04-01
#
# 原子提交辅助脚本
# 帮助开发者规划和执行原子提交

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🎯 原子提交辅助工具${NC}"
echo "===================="
echo ""

# 1. 显示当前变更
echo -e "${YELLOW}📋 当前变更文件:${NC}"
git status --short
echo ""

# 2. 按目录分组
echo -e "${YELLOW}📂 按领域分组:${NC}"

# 定义领域映射
declare -A DOMAINS
declare -A FILES

DOMAINS=(
    ["harness"]="Harness架构"
    ["source"]="源代码"
    ["tests"]="测试代码"
    ["docs"]="文档"
    ["config"]="配置"
    ["scripts"]="脚本"
    ["other"]="其他"
)

# 分类文件
while IFS= read -r line; do
    [[ -z "$line" ]] && continue
    
    file=$(echo "$line" | sed 's/^.. //')
    
    if [[ "$file" =~ ^\.harness/ ]] || [[ "$file" == "AGENTS.md" ]] || [[ "$file" == "harness.yaml" ]]; then
        FILES["harness"]+="$file\n"
    elif [[ "$file" =~ ^hull/src/ ]] || [[ "$file" =~ ^bridge/src/ ]]; then
        FILES["source"]+="$file\n"
    elif [[ "$file" =~ ^hull/tests/ ]] || [[ "$file" =~ ^e2e/ ]]; then
        FILES["tests"]+="$file\n"
    elif [[ "$file" =~ \.md$ ]] || [[ "$file" =~ ^docs/ ]]; then
        FILES["docs"]+="$file\n"
    elif [[ "$file" =~ \.toml$ ]] || [[ "$file" =~ \.yaml$ ]] || [[ "$file" =~ ^config/ ]]; then
        FILES["config"]+="$file\n"
    elif [[ "$file" =~ ^scripts/ ]]; then
        FILES["scripts"]+="$file\n"
    else
        FILES["other"]+="$file\n"
    fi
done < <(git status --porcelain)

# 显示分组
for key in harness source tests docs config scripts other; do
    if [[ -n "${FILES[$key]}" ]]; then
        echo -e "\n${GREEN}[${DOMAINS[$key]}]${NC}"
        echo -e "${FILES[$key]}" | sed '/^$/d' | sed 's/^/  /'
    fi
done

# 3. 原子提交建议
echo ""
echo -e "${YELLOW}💡 建议的原子提交拆分:${NC}"
echo ""

COMMIT_COUNT=0

if [[ -n "${FILES["harness"]}" ]]; then
    COMMIT_COUNT=$((COMMIT_COUNT + 1))
    echo -e "${BLUE}提交 $COMMIT_COUNT: Harness 架构${NC}"
    echo "  git add .harness/ AGENTS.md harness.yaml"
    echo "  git commit -m \"harness: add/update architecture guidelines\""
    echo ""
fi

if [[ -n "${FILES["source"]}" ]]; then
    COMMIT_COUNT=$((COMMIT_COUNT + 1))
    echo -e "${BLUE}提交 $COMMIT_COUNT: 源代码变更${NC}"
    
    # 尝试推断变更类型
    if git diff --cached --name-only | grep -q "test"; then
        echo "  git add hull/src/ bridge/src/"
        echo "  git commit -m \"feat: implement feature X\""
    elif git diff --cached --name-only | grep -q "refactor"; then
        echo "  git add hull/src/ bridge/src/"
        echo "  git commit -m \"refactor: improve code structure\""
    else
        echo "  git add hull/src/ bridge/src/"
        echo "  git commit -m \"feat/fix/refactor: description\""
    fi
    echo ""
fi

if [[ -n "${FILES["tests"]}" ]]; then
    COMMIT_COUNT=$((COMMIT_COUNT + 1))
    echo -e "${BLUE}提交 $COMMIT_COUNT: 测试代码${NC}"
    echo "  git add hull/tests/ e2e/"
    echo "  git commit -m \"test: add tests for feature X\""
    echo ""
fi

if [[ -n "${FILES["docs"]}" ]]; then
    COMMIT_COUNT=$((COMMIT_COUNT + 1))
    echo -e "${BLUE}提交 $COMMIT_COUNT: 文档更新${NC}"
    echo "  git add *.md docs/"
    echo "  git commit -m \"docs: update documentation\""
    echo ""
fi

if [[ -n "${FILES["config"]}" ]]; then
    COMMIT_COUNT=$((COMMIT_COUNT + 1))
    echo -e "${BLUE}提交 $COMMIT_COUNT: 配置变更${NC}"
    echo "  git add *.toml *.yaml config/"
    echo "  git commit -m \"config: update configuration\""
    echo ""
fi

# 4. 交互式选择
echo ""
echo -e "${YELLOW}🚀 下一步操作:${NC}"
echo ""
echo "1) 按建议拆分提交 (推荐)"
echo "2) 查看当前变更详情"
echo "3) 直接提交所有 (不推荐)"
echo "4) 取消"
echo ""
read -p "选择 [1-4]: " choice

case $choice in
    1)
        echo ""
        echo -e "${GREEN}按建议执行:${NC}"
        echo "请手动执行上述建议的 git add 和 git commit 命令"
        echo "或使用 git add -p 进行补丁级别的精选提交"
        ;;
    2)
        echo ""
        git diff --cached --stat
        echo ""
        echo "详细变更:"
        git diff --cached
        ;;
    3)
        echo ""
        echo -e "${YELLOW}⚠️  警告: 您选择了非原子提交${NC}"
        read -p "请说明为什么需要合并提交: " reason
        git commit -m "WIP: $reason"
        echo -e "${GREEN}已提交，但建议未来遵循原子提交原则${NC}"
        ;;
    4)
        echo "取消操作"
        exit 0
        ;;
    *)
        echo "无效选择"
        exit 1
        ;;
esac
