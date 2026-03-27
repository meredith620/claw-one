#!/bin/bash
# 测试框架统计报告

echo "=========================================="
echo "Claw One 测试框架统计报告"
echo "=========================================="
echo ""

cd "$(dirname "$0")/.."

echo "📊 测试文件统计"
echo "------------------------------------------"

echo ""
echo "Layer 1: 单元测试 (hull/src/ 中的 #[cfg(test)])"
echo "------------------------------------------"
UNIT_TESTS=$(grep -r "#\[tokio::test\]" hull/src/ 2>/dev/null | wc -l)
echo "  单元测试数量: $UNIT_TESTS"
echo ""
echo "  覆盖模块:"
echo "    - config.rs (ConfigManager CRUD)"
echo "    - state.rs (StateManager 状态机)"
echo "    - validation.rs (配置验证)"

echo ""
echo "Layer 2: 集成测试 (hull/tests/)"
echo "------------------------------------------"
for file in hull/tests/*.rs; do
    if [ -f "$file" ]; then
        count=$(grep -c "#\[tokio::test\]" "$file" 2>/dev/null || echo 0)
        name=$(basename "$file" .rs)
        printf "  %-30s %3d 个测试\n" "$name" "$count"
    fi
done

INTEGRATION_TOTAL=$(grep -r "#\[tokio::test\]" hull/tests/ 2>/dev/null | wc -l)
echo ""
echo "  集成测试总计: $INTEGRATION_TOTAL 个"

echo ""
echo "Layer 3: E2E 测试 (e2e/tests/)"
echo "------------------------------------------"
for file in e2e/tests/*.sh; do
    if [ -f "$file" ]; then
        name=$(basename "$file")
        printf "  %-40s\n" "$name"
    fi
done

E2E_COUNT=$(ls e2e/tests/*.sh 2>/dev/null | wc -l)
echo ""
echo "  E2E 测试总计: $E2E_COUNT 个"

echo ""
echo "=========================================="
echo "测试分类"
echo "=========================================="
echo ""
echo "✅ 基础功能测试:"
echo "  - Health API"
echo "  - Config 读写"
echo "  - Provider CRUD"
echo "  - Agent/Memory/Channel 读写"
echo ""
echo "✅ 数据完整性测试:"
echo "  - Save 操作保留未修改字段"
echo "  - Deep merge 验证"
echo ""
echo "✅ 错误场景测试:"
echo "  - 非法 JSON"
echo "  - 重复 ID"
echo "  - 404 处理"
echo "  - 空请求体"
echo "  - XSS 防护"
echo ""
echo "✅ 模块联动测试:"
echo "  - Provider 删除与 Model Priority"
echo "  - 多账号 Channel"
echo "  - Provider 多实例"
echo ""
echo "✅ 状态机测试:"
echo "  - Normal → SafeMode 转换"
echo "  - SafeMode 恢复"
echo "  - 并发状态访问"
echo ""
echo "✅ Git 操作测试:"
echo "  - 自动提交"
echo "  - 快照列表"
echo "  - 回滚功能"
echo ""
echo "✅ 用户流程测试:"
echo "  - 流程 A: 首次启动配置"
echo "  - 流程 B: 正常配置更新"
echo "  - 流程 C: 配置错误回滚"
echo "  - 流程 D: 出厂设置"

echo ""
echo "=========================================="
echo "总计"
echo "=========================================="
TOTAL=$((UNIT_TESTS + INTEGRATION_TOTAL))
echo ""
echo "  Layer 1 (单元测试): $UNIT_TESTS 个"
echo "  Layer 2 (集成测试): $INTEGRATION_TOTAL 个"
echo "  Layer 3 (E2E 测试): $E2E_COUNT 个"
echo ""
echo "  总计: $TOTAL 个自动化测试"
echo ""
echo "=========================================="
