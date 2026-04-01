#!/bin/bash
#
# E2E 测试辅助函数
# 提供标准断言函数，简化测试脚本
#

# 颜色输出
if [[ -t 1 ]]; then
    RED='\033[0;31m'
    GREEN='\033[0;32m'
    YELLOW='\033[1;33m'
    BLUE='\033[0;34m'
    NC='\033[0m'
else
    RED='' GREEN='' YELLOW='' BLUE='' NC=''
fi

# 打印信息
info() { echo -e "${BLUE}ℹ${NC} $1"; }
ok() { echo -e "${GREEN}✓${NC} $1"; }
warn() { echo -e "${YELLOW}⚠${NC} $1"; }
error() { echo -e "${RED}✗${NC} $1"; }

# 断言相等
# 用法: assert_eq "$actual" "$expected" "描述"
assert_eq() {
    local actual="$1"
    local expected="$2"
    local message="${3:-}"
    
    if [[ "$actual" == "$expected" ]]; then
        ok "${message:-断言通过}"
        return 0
    else
        error "${message:-断言失败} (expected: $expected, actual: $actual)"
        return 1
    fi
}

# 断言包含
# 用法: assert_contains "$haystack" "$needle" "描述"
assert_contains() {
    local haystack="$1"
    local needle="$2"
    local message="${3:-}"
    
    if echo "$haystack" | grep -q "$needle"; then
        ok "${message:-包含断言通过}"
        return 0
    else
        error "${message:-包含断言失败} (expected to contain: $needle)"
        return 1
    fi
}

# 断言不包含
# 用法: assert_not_contains "$haystack" "$needle" "描述"
assert_not_contains() {
    local haystack="$1"
    local needle="$2"
    local message="${3:-}"
    
    if ! echo "$haystack" | grep -q "$needle"; then
        ok "${message:-不包含断言通过}"
        return 0
    else
        error "${message:-不包含断言失败} (expected not to contain: $needle)"
        return 1
    fi
}

# 断言为真
# 用法: assert_true "$condition" "描述"
assert_true() {
    local condition="$1"
    local message="${2:-}"
    
    if eval "$condition"; then
        ok "${message:-条件为真}"
        return 0
    else
        error "${message:-条件为假}"
        return 1
    fi
}

# 断言为假
# 用法: assert_false "$condition" "描述"
assert_false() {
    local condition="$1"
    local message="${2:-}"
    
    if ! eval "$condition"; then
        ok "${message:-条件为假}"
        return 0
    else
        error "${message:-条件为真（期望为假）}"
        return 1
    fi
}

# 断言数值大于
# 用法: assert_gt "$actual" "$expected" "描述"
assert_gt() {
    local actual="$1"
    local expected="$2"
    local message="${3:-}"
    
    if [[ "$actual" -gt "$expected" ]]; then
        ok "${message:-数值断言通过}"
        return 0
    else
        error "${message:-数值断言失败} (expected > $expected, actual: $actual)"
        return 1
    fi
}

# 断言 JSON 字段存在
# 用法: assert_json_has "$json" "$jq_path" "描述"
assert_json_has() {
    local json="$1"
    local jq_path="$2"
    local message="${3:-}"
    
    local value
    value=$(echo "$json" | jq -r "$jq_path" 2>/dev/null)
    
    if [[ "$value" != "null" ]] && [[ "$value" != "" ]]; then
        ok "${message:-JSON 字段存在} ($jq_path: $value)"
        return 0
    else
        error "${message:-JSON 字段不存在或为空} ($jq_path)"
        return 1
    fi
}

# 断言 JSON 字段值
# 用法: assert_json_eq "$json" "$jq_path" "$expected" "描述"
assert_json_eq() {
    local json="$1"
    local jq_path="$2"
    local expected="$3"
    local message="${4:-}"
    
    local actual
    actual=$(echo "$json" | jq -r "$jq_path" 2>/dev/null)
    
    assert_eq "$actual" "$expected" "$message"
}
