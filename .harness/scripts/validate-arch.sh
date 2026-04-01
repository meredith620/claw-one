#!/bin/bash
# HARNESS METADATA
# type: script
# part-of: harness-architecture
# scope: validation
# managed-by: harness-system
# do-not-modify-manually: 如需修改请遵循 harness-evolution.spec.md
# version: 1.0
# created: 2026-04-01
#
# Harness 架构验证脚本
# 检查代码是否符合架构约束

set -e

echo "🔍 运行 Harness 架构验证..."
echo ""

ERRORS=0

# 1. 检查禁止的直接调用
echo "  检查 API 层是否直接调用系统命令..."
if grep -r "Command::new" hull/src/api/ --include="*.rs" 2>/dev/null; then
    echo "    ⚠️  警告: API 层包含系统调用，应移至 runtime.rs"
    ERRORS=$((ERRORS + 1))
else
    echo "    ✅ 通过"
fi

# 2. 检查 API 层是否包含业务逻辑
echo "  检查 API 层业务逻辑隔离..."
if grep -r "git2::" hull/src/api/ --include="*.rs" 2>/dev/null; then
    echo "    ❌ 错误: API 层直接调用 Git 操作，应通过 ConfigManager"
    ERRORS=$((ERRORS + 1))
else
    echo "    ✅ 通过"
fi

# 3. 检查 Cargo.toml 依赖
echo "  检查依赖声明..."
if [ -f "hull/Cargo.toml" ]; then
    echo "    ✅ Cargo.toml 存在"
else
    echo "    ❌ 错误: 缺少 Cargo.toml"
    ERRORS=$((ERRORS + 1))
fi

# 4. 检查测试分层
echo "  检查测试文件组织..."
UNIT_TESTS=$(find hull/src -name "*.rs" -exec grep -l "#\[cfg(test)\]" {} \; | wc -l)
INTEG_TESTS=$(find hull/tests -name "*.rs" 2>/dev/null | wc -l)
echo "    单元测试模块: $UNIT_TESTS"
echo "    集成测试文件: $INTEG_TESTS"

if [ "$UNIT_TESTS" -eq 0 ]; then
    echo "    ⚠️  警告: 未发现单元测试"
fi

if [ "$INTEG_TESTS" -eq 0 ]; then
    echo "    ⚠️  警告: 未发现集成测试"
fi

# 5. 检查错误处理
echo "  检查错误类型定义..."
if grep -q "thiserror" hull/Cargo.toml; then
    echo "    ✅ 使用 thiserror 定义错误"
else
    echo "    ⚠️  警告: 未使用 thiserror"
fi

# 6. 检查单实例控制
echo "  检查单实例控制..."
if grep -q "InstanceLock\|try_lock_exclusive" hull/src/*.rs; then
    echo "    ✅ 单实例控制已实现"
else
    echo "    ⚠️  警告: 未发现单实例控制实现"
fi

# 7. 检查 AGENTS.md 和 harness.yaml 存在
echo "  检查 Harness 入口文件..."
if [ -f "AGENTS.md" ] && [ -f "harness.yaml" ]; then
    echo "    ✅ AGENTS.md 和 harness.yaml 存在"
else
    echo "    ❌ 错误: 缺少 Harness 入口文件"
    ERRORS=$((ERRORS + 1))
fi

echo ""
if [ $ERRORS -eq 0 ]; then
    echo "✅ 架构验证通过"
    exit 0
else
    echo "❌ 架构验证失败: 发现 $ERRORS 个问题"
    exit 1
fi
