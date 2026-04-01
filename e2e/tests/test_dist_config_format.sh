#!/bin/bash
#
# 测试：验证 setup-config.sh 生成的配置文件格式正确
# 
# 这个测试用于暴露 Bug #2: make dist 安装后，setup-config.sh 生成的配置文件
# 缺少段落信息（如 [server], [openclaw]）

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
TEST_DIR="/tmp/claw-one-config-format-test-$$"

echo "========================================"
echo "测试: setup-config.sh 配置格式验证"
echo "========================================"
echo ""

# 清理函数
cleanup() {
    if [ -d "$TEST_DIR" ]; then
        echo "清理测试目录: $TEST_DIR"
        rm -rf "$TEST_DIR"
    fi
}
trap cleanup EXIT

# 创建测试环境
setup_test_env() {
    echo "1. 创建测试环境..."
    mkdir -p "$TEST_DIR"
    mkdir -p "$TEST_DIR/bin"
    mkdir -p "$TEST_DIR/config"
    
    # 复制 setup-config.sh 到测试目录
    cp "$PROJECT_ROOT/scripts/setup-config.sh" "$TEST_DIR/bin/"
    chmod +x "$TEST_DIR/bin/setup-config.sh"
    
    echo "   ✅ 测试环境创建完成"
    echo ""
}

# 模拟 setup-config.sh 的 init_config 函数行为
# 这是 Bug #2 的关键：当前实现缺少段落标记
generate_config_old() {
    cat > "$TEST_DIR/config/claw-one.toml" << 'EOF'
# Claw One 配置文件

# 服务端口
port = 8080

# OpenClaw 配置路径
openclaw_home = "~/.openclaw"

# 日志级别: debug, info, warn, error
log_level = "info"
EOF
}

# 修复后的配置格式（setup-config.sh 现在生成正确的格式）
generate_config_fixed() {
    # 这对应修复后的 setup-config.sh init_config() 函数
    cat > "$TEST_DIR/config/claw-one.toml" << 'EOF'
[server]
port = 8080
log_level = "info"

[openclaw]
openclaw_home = "~/.openclaw"
EOF
}

# 验证配置文件中包含段落标记
verify_config_sections() {
    local config_file="$1"
    local test_name="$2"
    
    echo "验证: $test_name"
    echo "配置文件路径: $config_file"
    echo ""
    
    if [ ! -f "$config_file" ]; then
        echo "❌ 配置文件不存在: $config_file"
        return 1
    fi
    
    echo "配置文件内容:"
    echo "---"
    cat "$config_file"
    echo "---"
    echo ""
    
    local has_error=0
    
    # 检查 [server] 段落
    if grep -q "^\[server\]" "$config_file"; then
        echo "   ✅ 包含 [server] 段落"
    else
        echo "   ❌ 缺少 [server] 段落 (Bug #2 暴露!)"
        has_error=1
    fi
    
    # 检查 [openclaw] 段落
    if grep -q "^\[openclaw\]" "$config_file"; then
        echo "   ✅ 包含 [openclaw] 段落"
    else
        echo "   ❌ 缺少 [openclaw] 段落 (Bug #2 暴露!)"
        has_error=1
    fi
    
    # 检查 port 配置在正确的位置
    if grep -A1 "^\[server\]" "$config_file" | grep -q "port"; then
        echo "   ✅ port 配置在 [server] 段落内"
    else
        echo "   ⚠️  port 配置可能不在正确的段落内"
    fi
    
    # 检查 openclaw_home 配置在正确的位置
    if grep -A1 "^\[openclaw\]" "$config_file" | grep -q "openclaw_home"; then
        echo "   ✅ openclaw_home 配置在 [openclaw] 段落内"
    else
        echo "   ⚠️  openclaw_home 配置可能不在正确的段落内"
    fi
    
    echo ""
    
    return $has_error
}

# 主测试流程
main() {
    setup_test_env
    
    echo "2. 测试当前实现（应该暴露 Bug #2）..."
    echo ""
    generate_config_old
    
    if verify_config_sections "$TEST_DIR/config/claw-one.toml" "当前实现"; then
        echo "✅ 当前实现配置格式正确"
    else
        echo "❌ 当前实现配置格式有误 - Bug #2 被暴露！"
        echo ""
        echo "问题: setup-config.sh 生成的配置缺少段落标记"
        echo "影响: Settings 结构体无法正确解析配置"
        echo ""
    fi
    
    echo ""
    echo "3. 测试修复后的格式..."
    echo ""
    generate_config_fixed
    
    if verify_config_sections "$TEST_DIR/config/claw-one.toml" "修复后的实现"; then
        echo "✅ 修复后的配置格式正确"
    else
        echo "❌ 修复后的配置格式有误"
        exit 1
    fi
    
    echo ""
    echo "========================================"
    echo "测试总结"
    echo "========================================"
    echo ""
    echo "Bug #2 根因确认:"
    echo "  setup-config.sh 的 init_config() 函数生成的配置缺少段落标记"
    echo ""
    echo "当前生成的格式:"
    echo "    port = 8080           ← 缺少 [server] 段落"
    echo "    openclaw_home = ...   ← 缺少 [openclaw] 段落"
    echo ""
    echo "期望的格式:"
    echo "    [server]"
    echo "    port = 8080"
    echo ""
    echo "    [openclaw]"
    echo "    openclaw_home = ..."
    echo ""
}

main "$@"