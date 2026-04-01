#!/bin/bash
# HARNESS METADATA
# type: script
# part-of: harness-architecture
# scope: guard-execution
# managed-by: harness-system
# do-not-modify-manually: 如需修改请遵循 harness-evolution.spec.md
# version: 1.0
# created: 2026-04-01
#
# 熵防护规则执行引擎
# 解析并执行 .harness/guards/*.rule 文件中定义的规则

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}🛡️  熵防护规则执行引擎${NC}"
echo "========================"
echo ""

GUARDS_DIR=".harness/guards"
TOTAL_VIOLATIONS=0
TOTAL_WARNINGS=0

# 检查 guards 目录是否存在
if [ ! -d "$GUARDS_DIR" ]; then
    echo -e "${RED}❌ 错误: $GUARDS_DIR 目录不存在${NC}"
    exit 1
fi

# 获取所有 .rule 文件
RULE_FILES=$(find "$GUARDS_DIR" -name "*.rule" 2>/dev/null)

if [ -z "$RULE_FILES" ]; then
    echo -e "${YELLOW}⚠️  未找到任何 .rule 文件${NC}"
    exit 0
fi

echo -e "${YELLOW}📋 检查 $(( $(echo "$RULE_FILES" | wc -l) )) 个熵防护规则...${NC}"
echo ""

# 遍历每个规则文件
for rule_file in $RULE_FILES; do
    rule_name=$(basename "$rule_file" .rule)
    echo -e "${BLUE}检查: $rule_name${NC}"
    
    # 读取规则文件并执行检查
    # 这里实现了各个规则类型的检查逻辑
    
    case "$rule_name" in
        "protect-harness-files")
            # 检查是否修改了受保护的 Harness 文件
            protected_files=(
                "AGENTS.md"
                "harness.yaml"
                ".harness/specs/"
                ".harness/scripts/"
                ".harness/guards/"
            )
            
            # 获取当前变更的文件
            changed_files=$(git diff --cached --name-only 2>/dev/null || git diff --name-only 2>/dev/null || echo "")
            
            for protected in "${protected_files[@]}"; do
                if echo "$changed_files" | grep -q "$protected"; then
                    echo -e "  ${YELLOW}⚠️  警告: 检测到受保护文件变更: $protected${NC}"
                    echo -e "  请确认这是有意的 Harness 修改，而非意外变更"
                    TOTAL_WARNINGS=$((TOTAL_WARNINGS + 1))
                fi
            done
            ;;
            
        "enforce-layer-separation")
            # 检查 API 层是否直接调用系统命令
            api_files=$(find hull/src/api -name "*.rs" 2>/dev/null || echo "")
            
            for api_file in $api_files; do
                # 检查是否包含禁止的模式
                if grep -q "Command::new" "$api_file" 2>/dev/null; then
                    if grep -q "tokio::process::Command\|std::process::Command" "$api_file" 2>/dev/null; then
                        echo -e "  ${RED}❌ 违规: $api_file 包含直接系统调用${NC}"
                        echo -e "     API 层应通过 RuntimeManager 执行系统命令"
                        TOTAL_VIOLATIONS=$((TOTAL_VIOLATIONS + 1))
                    fi
                fi
                
                if grep -q "git2::" "$api_file" 2>/dev/null; then
                    echo -e "  ${RED}❌ 违规: $api_file 直接调用 Git 操作${NC}"
                    echo -e "     API 层应通过 ConfigManager 执行 Git 操作"
                    TOTAL_VIOLATIONS=$((TOTAL_VIOLATIONS + 1))
                fi
                
                if grep -q "fs::write\|fs::read\|File::open" "$api_file" 2>/dev/null; then
                    # 检查是否是配置相关的文件操作（某些文件操作是允许的）
                    if ! grep -q "tokio::fs::File" "$api_file" 2>/dev/null; then
                        echo -e "  ${YELLOW}⚠️  注意: $api_file 可能包含直接文件操作${NC}"
                    fi
                fi
            done
            ;;
            
        "no-direct-config-modification")
            # 检查是否有直接修改 openclaw.json 的模式
            suspicious_patterns=(
                'echo.*\.openclaw/openclaw\.json'
                'cat.*>.*\.openclaw/openclaw\.json'
                'tee.*\.openclaw/openclaw\.json'
            )
            
            changed_files=$(git diff --cached --name-only 2>/dev/null || git diff --name-only 2>/dev/null || echo "")
            
            for pattern in "${suspicious_patterns[@]}"; do
                # 在 shell 脚本中检查
                shell_files=$(find . -name "*.sh" -type f 2>/dev/null | grep -v ".harness/scripts/pre-" || echo "")
                for sh_file in $shell_files; do
                    if grep -qE "$pattern" "$sh_file" 2>/dev/null; then
                        echo -e "  ${RED}❌ 违规: $sh_file 包含直接配置修改模式${NC}"
                        echo -e "     请通过 claw-one API 修改配置"
                        TOTAL_VIOLATIONS=$((TOTAL_VIOLATIONS + 1))
                    fi
                done
            done
            ;;
            
        "enforce-atomic-commits")
            # 检查提交消息是否符合规范
            if git rev-parse --verify HEAD >/dev/null 2>&1; then
                last_msg=$(git log -1 --format="%s" HEAD 2>/dev/null || echo "")
                
                # 检查是否包含 type(scope): 格式
                if ! echo "$last_msg" | grep -qE "^[a-z]+(\([a-z-]+\))?: "; then
                    echo -e "  ${YELLOW}⚠️  建议: 提交消息格式不符合 conventional commits${NC}"
                    echo -e "     建议格式: <type>(<scope>): <subject>"
                    TOTAL_WARNINGS=$((TOTAL_WARNINGS + 1))
                fi
            fi
            ;;
            
        "require-test-coverage")
            # 检查关键模块变更是否有对应测试
            critical_files=(
                "hull/src/config.rs"
                "hull/src/state.rs"
                "hull/src/validation.rs"
            )
            
            changed_files=$(git diff --cached --name-only 2>/dev/null || git diff --name-only 2>/dev/null || echo "")
            
            for critical in "${critical_files[@]}"; do
                if echo "$changed_files" | grep -q "$critical"; then
                    # 检查是否有对应的测试文件变更
                    test_file=$(echo "$critical" | sed 's|src/|tests/api_|g' | sed 's|\.rs$|_test.rs|g')
                    if ! echo "$changed_files" | grep -q "$test_file" && ! echo "$changed_files" | grep -q "_test\.rs"; then
                        echo -e "  ${YELLOW}⚠️  注意: 关键模块 $critical 变更但无对应测试更新${NC}"
                        TOTAL_WARNINGS=$((TOTAL_WARNINGS + 1))
                    fi
                fi
            done
            ;;
            
        *)
            echo -e "  ${YELLOW}⚠️  未知规则类型，跳过${NC}"
            ;;
    esac
    
    echo ""
done

# 汇总结果
echo "========================"
echo -e "${BLUE}📊 检查结果汇总${NC}"
echo ""

if [ $TOTAL_VIOLATIONS -gt 0 ]; then
    echo -e "${RED}❌ 发现 $TOTAL_VIOLATIONS 个违规${NC}"
    echo -e "${RED}   请修复上述问题后再提交${NC}"
    exit 1
elif [ $TOTAL_WARNINGS -gt 0 ]; then
    echo -e "${YELLOW}⚠️  发现 $TOTAL_WARNINGS 个警告${NC}"
    echo -e "${YELLOW}   建议检查，但不影响提交${NC}"
    exit 0
else
    echo -e "${GREEN}✅ 所有熵防护规则检查通过${NC}"
    exit 0
fi
