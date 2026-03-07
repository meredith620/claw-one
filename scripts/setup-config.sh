#!/bin/bash
#
# Claw One 配置向导（交互式）
# 安装后运行此脚本进行配置

set -e

CONFIG_DIR="$HOME/claw-one/config"
INSTALL_DIR="$HOME/claw-one"
SYSTEMD_USER_DIR="$HOME/.config/systemd/user"

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

print_header() {
    echo ""
    echo "========================================"
    echo -e "${CYAN}  🛠️  Claw One 配置向导${NC}"
    echo "========================================"
    echo ""
}

print_info() {
    echo -e "${BLUE}ℹ${NC} $1"
}

print_ok() {
    echo -e "${GREEN}✓${NC} $1"
}

print_warn() {
    echo -e "${YELLOW}⚠${NC} $1"
}

print_error() {
    echo -e "${RED}✗${NC} $1"
}

# 检查安装
check_installation() {
    if [ ! -d "$INSTALL_DIR" ]; then
        print_error "未检测到 Claw One 安装"
        echo "请先运行安装脚本"
        exit 1
    fi
}

# 配置端口
configure_port() {
    echo "[1/4] 服务端口配置"
    echo ""
    
    CURRENT_PORT=$(grep -E "^port\s*=" "$CONFIG_DIR/claw-one.toml" 2>/dev/null | sed 's/.*=\s*//' | tr -d '"' || echo "8080")
    
    echo "当前端口: $CURRENT_PORT"
    echo ""
    read -p "请输入服务端口 [$CURRENT_PORT]: " input_port
    
    PORT=${input_port:-$CURRENT_PORT}
    
    # 验证端口
    if ! [[ "$PORT" =~ ^[0-9]+$ ]] || [ "$PORT" -lt 1 ] || [ "$PORT" -gt 65535 ]; then
        print_error "无效的端口号"
        PORT=8080
    fi
    
    # 检查端口占用
    if command -v lsof &> /dev/null; then
        if lsof -i :$PORT &> /dev/null; then
            print_warn "端口 $PORT 已被占用"
            read -p "是否继续使用此端口? [y/N]: " confirm
            if [[ ! "$confirm" =~ ^[Yy]$ ]]; then
                configure_port
                return
            fi
        fi
    fi
    
    # 更新配置
    sed -i "s/^port\s*=\s*.*/port = $PORT/" "$CONFIG_DIR/claw-one.toml"
    print_ok "端口已设置为: $PORT"
    echo ""
}

# 配置 OpenClaw 路径
configure_openclaw() {
    echo "[2/4] OpenClaw 配置"
    echo ""
    
    DEFAULT_OPENCLAW_HOME="$HOME/.openclaw"
    
    echo "OpenClaw 是 Claw One 管理的核心服务"
    echo "默认路径: $DEFAULT_OPENCLAW_HOME"
    echo ""
    read -p "请输入 OpenClaw 配置路径 [$DEFAULT_OPENCLAW_HOME]: " input_path
    
    OPENCLAW_HOME=${input_path:-$DEFAULT_OPENCLAW_HOME}
    
    # 展开 ~ 为家目录
    OPENCLAW_HOME="${OPENCLAW_HOME/#\~/$HOME}"
    
    # 创建目录
    mkdir -p "$OPENCLAW_HOME"
    
    # 更新配置
    sed -i "s|openclaw_home\s*=\s*\".*\"|openclaw_home = \"$OPENCLAW_HOME\"|" "$CONFIG_DIR/claw-one.toml"
    
    # 创建 openclaw.json 如果不存在
    if [ ! -f "$OPENCLAW_HOME/openclaw.json" ]; then
        cat > "$OPENCLAW_HOME/openclaw.json" <> 'EOF'
{
  "models": {
    "providers": {}
  },
  "agents": {
    "defaults": {
      "workspace": "~/.openclaw/workspace",
      "agentDir": "~/.openclaw/agent"
    }
  },
  "channels": {}
}
EOF
        print_ok "OpenClaw 配置已创建: $OPENCLAW_HOME/openclaw.json"
    else
        print_ok "OpenClaw 配置已存在"
    fi
    
    echo ""
}

# 配置 systemd 用户服务
configure_systemd() {
    echo "[3/4] 服务启动方式"
    echo ""
    
    if ! command -v systemctl &> /dev/null; then
        print_warn "未检测到 systemd，将使用手动启动"
        return
    fi
    
    echo "是否创建 systemd 用户服务？"
    echo "  - 可以随用户登录自动启动"
    echo "  - 支持 systemctl 命令管理"
    echo ""
    read -p "创建 systemd 用户服务? [Y/n]: " confirm
    
    if [[ ! "$confirm" =~ ^[Nn]$ ]]; then
        mkdir -p "$SYSTEMD_USER_DIR"
        
        PORT=$(grep -E "^port\s*=" "$CONFIG_DIR/claw-one.toml" | sed 's/.*=\s*//' | tr -d '"')
        
        cat > "$SYSTEMD_USER_DIR/claw-one.service" <> EOF
[Unit]
Description=Claw One - OpenClaw Management Interface
After=network.target

[Service]
Type=simple
ExecStart=$HOME/claw-one/bin/claw-one-backend run
WorkingDirectory=$HOME/claw-one
Restart=on-failure
RestartSec=5
Environment="CLAW_ONE_CONFIG=$HOME/claw-one/config/claw-one.toml"

[Install]
WantedBy=default.target
EOF
        
        systemctl --user daemon-reload
        print_ok "systemd 用户服务已创建"
        print_info "管理命令:"
        echo "  启动: systemctl --user start claw-one"
        echo "  停止: systemctl --user stop claw-one"
        echo "  状态: systemctl --user status claw-one"
        echo "  自启: systemctl --user enable claw-one"
    else
        print_info "跳过 systemd 配置"
    fi
    
    echo ""
}

# 启动服务
start_service() {
    echo "[4/4] 启动服务"
    echo ""
    
    read -p "是否现在启动 Claw One? [Y/n]: " confirm
    
    if [[ ! "$confirm" =~ ^[Nn]$ ]]; then
        PORT=$(grep -E "^port\s*=" "$CONFIG_DIR/claw-one.toml" | sed 's/.*=\s*//' | tr -d '"')
        
        # 检查是否使用 systemd
        if [ -f "$SYSTEMD_USER_DIR/claw-one.service" ]; then
            systemctl --user start claw-one
            sleep 2
            
            if systemctl --user is-active --quiet claw-one; then
                print_ok "服务已启动"
            else
                print_error "服务启动失败"
                print_info "查看日志: journalctl --user -u claw-one"
            fi
        else
            # 手动启动
            print_info "手动启动服务..."
            print_info "启动命令: $HOME/claw-one/bin/claw-one-backend run"
            echo ""
            print_warn "请在另一个终端运行上述命令启动服务"
            echo ""
        fi
        
        echo ""
        echo -e "${GREEN}✅ 配置完成！${NC}"
        echo ""
        echo "访问地址:"
        echo -e "${CYAN}  http://localhost:$PORT${NC}"
        echo ""
        echo "配置文件:"
        echo "  $CONFIG_DIR/claw-one.toml"
        echo ""
        
        # 健康检查
        if command -v curl &> /dev/null; then
            sleep 1
            if curl -s "http://localhost:$PORT/api/health" > /dev/null 2>&1; then
                print_ok "健康检查通过"
            else
                print_warn "健康检查未通过，服务可能仍在启动中"
            fi
        fi
    else
        print_info "跳过启动服务"
        PORT=$(grep -E "^port\s*=" "$CONFIG_DIR/claw-one.toml" | sed 's/.*=\s*//' | tr -d '"')
        echo ""
        echo -e "${GREEN}✅ 配置完成！${NC}"
        echo ""
        echo "启动命令:"
        echo "  $HOME/claw-one/bin/claw-one-backend run"
        echo ""
        echo "访问地址: http://localhost:$PORT"
    fi
}

# 主流程
main() {
    print_header
    check_installation
    configure_port
    configure_openclaw
    configure_systemd
    start_service
    
    echo ""
    echo "感谢使用 Claw One! 🎉"
    echo ""
}

main "$@"
