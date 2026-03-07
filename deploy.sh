#!/bin/bash

# Claw One 部署脚本
# 用于构建和启动前后端服务

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BACKEND_DIR="$SCRIPT_DIR/backend"
FRONTEND_DIR="$SCRIPT_DIR/frontend"
CONFIG_DIR="$HOME/claw-one/config"

echo "🚀 Claw One 部署脚本"
echo "===================="

# 检查依赖
check_deps() {
    echo "📦 检查依赖..."
    
    # 检查 Rust/Cargo
    if ! command -v cargo &> /dev/null; then
        echo "❌ 未找到 Rust/Cargo，请先安装"
        exit 1
    fi
    
    # 检查 Node.js/npm
    if ! command -v npm &> /dev/null; then
        echo "❌ 未找到 Node.js/npm，请先安装"
        exit 1
    fi
    
    echo "✅ 依赖检查通过"
}

# 构建后端
build_backend() {
    echo "🔨 构建后端..."
    cd "$BACKEND_DIR"
    cargo build --release
    echo "✅ 后端构建完成"
}

# 构建前端
build_frontend() {
    echo "🎨 构建前端..."
    cd "$FRONTEND_DIR"
    npm install
    npm run build
    echo "✅ 前端构建完成"
}

# 初始化配置
init_config() {
    echo "⚙️ 初始化配置..."
    
    mkdir -p "$CONFIG_DIR"
    
    if [ ! -f "$CONFIG_DIR/claw-one.toml" ]; then
        cat > "$CONFIG_DIR/claw-one.toml" << 'EOF'
[server]
host = "0.0.0.0"
port = 8080
log_level = "info"

[openclaw]
openclaw_home = "~/.openclaw"
service_name = "openclaw"
health_port = 18790
health_timeout = 30

[paths]
data_dir = "~/.config/claw-one"

[features]
auto_backup = true
safe_mode = true
first_run_wizard = true
EOF
        echo "✅ 配置文件已创建: $CONFIG_DIR/claw-one.toml"
    else
        echo "ℹ️ 配置文件已存在"
    fi
}

# 启动服务
start() {
    echo "🚀 启动 Claw One..."
    
    # 检查是否已在运行
    if pgrep -f "claw-one-backend" > /dev/null; then
        echo "⚠️ Claw One 已在运行"
        exit 0
    fi
    
    # 启动后端
    export CLAW_ONE_CONFIG="$CONFIG_DIR/claw-one.toml"
    nohup "$BACKEND_DIR/target/release/claw-one-backend" run > /tmp/claw-one.log 2>&1 &
    
    echo "✅ 后端已启动 (PID: $!)"
    echo "📋 日志: tail -f /tmp/claw-one.log"
    
    # 等待后端启动
    sleep 2
    
    # 检查健康状态
    if curl -s http://localhost:8080/api/health > /dev/null; then
        echo "✅ 服务健康检查通过"
        echo "🌐 访问: http://localhost:8080"
    else
        echo "⚠️ 服务启动中，请稍后检查日志"
    fi
}

# 停止服务
stop() {
    echo "🛑 停止 Claw One..."
    pkill -f claw-one-backend || true
    echo "✅ 服务已停止"
}

# 查看状态
status() {
    if pgrep -f "claw-one-backend" > /dev/null; then
        echo "✅ Claw One 运行中"
        curl -s http://localhost:8080/api/health | jq . 2>/dev/null || echo "健康检查失败"
    else
        echo "❌ Claw One 未运行"
    fi
}

# 查看日志
logs() {
    tail -f /tmp/claw-one.log
}

# 使用说明
usage() {
    echo "用法: $0 {build|init|start|stop|restart|status|logs}"
    echo ""
    echo "命令:"
    echo "  build    - 构建前后端"
    echo "  init     - 初始化配置"
    echo "  start    - 启动服务"
    echo "  stop     - 停止服务"
    echo "  restart  - 重启服务"
    echo "  status   - 查看状态"
    echo "  logs     - 查看日志"
}

# 主逻辑
case "${1:-}" in
    build)
        check_deps
        build_backend
        build_frontend
        ;;
    init)
        init_config
        ;;
    start)
        start
        ;;
    stop)
        stop
        ;;
    restart)
        stop
        sleep 1
        start
        ;;
    status)
        status
        ;;
    logs)
        logs
        ;;
    *)
        usage
        exit 1
        ;;
esac
