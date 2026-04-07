#!/bin/bash
# HARNESS METADATA
# type: script
# part-of: harness-architecture
# scope: pre-commit-hook
# managed-by: harness-system
# do-not-modify-manually: 如需修改请遵循 harness-evolution.spec.md
# version: 1.0
# created: 2026-04-01
#
# 提交前检查脚本
# 应在 git commit 前运行 (可作为 git pre-commit hook)

set -e

echo "🔍 提交前检查..."
echo ""

FAILED=0

# 1. 代码格式检查
echo "  检查 Rust 代码格式..."
if command -v cargo >/dev/null 2>&1; then
    cd hull
    if cargo fmt -- --check 2>/dev/null; then
        echo "    ✅ 代码格式正确"
    else
        echo "    ❌ 代码格式错误，运行 'cargo fmt' 修复"
        FAILED=$((FAILED + 1))
    fi
    cd ..
else
    echo "    ⏭️  跳过 (Rust 未安装)"
fi

# 2. 编译检查
echo "  检查编译..."
if command -v cargo >/dev/null 2>&1; then
    cd hull
    if cargo check 2>&1 | grep -q "^error"; then
        echo "    ❌ 编译错误"
        FAILED=$((FAILED + 1))
    else
        echo "    ✅ 编译通过"
    fi
    cd ..
else
    echo "    ⏭️  跳过 (Rust 未安装)"
fi

# 3. Clippy 检查
echo "  运行 Clippy..."
if command -v cargo >/dev/null 2>&1; then
    cd hull
    # 允许警告，只检查错误
    if cargo clippy -- -D warnings 2>&1 | grep -q "^error"; then
        echo "    ❌ Clippy 错误"
        FAILED=$((FAILED + 1))
    else
        echo "    ✅ Clippy 通过 (可能有警告)"
    fi
    cd ..
else
    echo "    ⏭️  跳过 (Rust 未安装)"
fi

# 4. 快速测试
echo "  运行快速测试..."
if command -v cargo >/dev/null 2>&1 && [ -f "Makefile" ]; then
    if make test-fast 2>&1 | tail -5 | grep -q "test result"; then
        echo "    ✅ 快速测试通过"
    else
        echo "    ❌ 快速测试失败"
        FAILED=$((FAILED + 1))
    fi
else
    echo "    ⏭️  跳过"
fi

# 5. 检查 AGENTS.md 是否过期
echo "  检查文档更新..."
if [ -f "AGENTS.md" ]; then
    # 检查关键文件是否比 AGENTS.md 新
    NEWER_FILES=$(find hull/src -name "*.rs" -newer AGENTS.md 2>/dev/null | head -5)
    if [ -n "$NEWER_FILES" ]; then
        echo "    ⚠️  以下源文件比 AGENTS.md 新，文档可能需要更新:"
        echo "$NEWER_FILES" | sed 's/^/       /'
    else
        echo "    ✅ 文档状态正常"
    fi
else
    echo "    ⚠️  缺少 AGENTS.md"
fi

echo ""
if [ $FAILED -eq 0 ]; then
    echo "✅ 提交前检查通过"
    exit 0
else
    echo "❌ 提交前检查失败: $FAILED 个问题"
    echo ""
    echo "💡 修复建议:"
    echo "   - cargo fmt          # 格式化代码"
    echo "   - cargo check        # 检查编译"
    echo "   - make test-fast     # 运行快速测试"
    exit 1
fi
