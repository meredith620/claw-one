# Claw One - Makefile
# Claw One 配置守护程序 - 构建和部署工具

# ============================================
# 变量定义
# ============================================

# 目录
FRONTEND_DIR := frontend
BACKEND_DIR := backend
STATIC_DIR := static/dist
BUILD_DIR := build
DIST_DIR := dist

# 工具
CARGO := cargo
NPM := npm
VITE := npx vite

# 安装路径 (用户级，默认 ~/claw-one)
INSTALL_DIR ?= $(HOME)/claw-one
BINDIR := $(INSTALL_DIR)/bin
DATADIR := $(INSTALL_DIR)/share
CONFIGDIR := $(INSTALL_DIR)/config
SYSTEMD_USER_DIR := $(HOME)/.config/systemd/user

# 版本和元数据
VERSION := 0.1.0
APP_NAME := claw-one

# 开发环境配置
DEV_CONFIG := $(BACKEND_DIR)/config.dev.toml

# ============================================
# 默认目标
# ============================================

.DEFAULT_GOAL := help

.PHONY: help deps compile install uninstall dev clean dist check

# ============================================
# 开发工作流 (deps → compile → install)
# ============================================

## 安装所有依赖 (deps)
deps:
	@echo "📥 安装依赖..."
	@echo "  [1/2] 前端依赖..."
	cd $(FRONTEND_DIR) && $(NPM) install
	@echo "  [2/2] 后端依赖..."
	cd $(BACKEND_DIR) && $(CARGO) fetch
	@echo "✅ 依赖安装完成"

## 编译项目 (compile) - 依赖: deps
compile: deps
	@echo "🔧 编译项目..."
	@echo "  [1/2] 编译前端..."
	cd $(FRONTEND_DIR) && $(VITE) build
	@echo "  [2/2] 编译后端..."
	cd $(BACKEND_DIR) && $(CARGO) build --release
	@echo "✅ 编译完成"
	@echo "  后端二进制: $(BACKEND_DIR)/target/release/claw-one-backend"
	@echo "  前端产物: $(STATIC_DIR)/"

## 用户级安装 (install) - 依赖: compile，安装到 ~/claw-one
install: compile
	@echo "📦 安装到 $(INSTALL_DIR)..."
	@echo "  [1/5] 创建目录结构..."
	mkdir -p $(BINDIR)
	mkdir -p $(DATADIR)/static
	mkdir -p $(CONFIGDIR)
	mkdir -p $(SYSTEMD_USER_DIR)
	@echo "  [2/5] 安装后端二进制..."
	cp $(BACKEND_DIR)/target/release/claw-one-backend $(BINDIR)/
	@echo "  [3/5] 安装前端静态文件..."
	cp -r $(STATIC_DIR)/* $(DATADIR)/static/
	@echo "  [4/5] 安装配置文件..."
	@if [ ! -f "$(CONFIGDIR)/claw-one.toml" ]; then \
		cp config/claw-one.toml.example $(CONFIGDIR)/claw-one.toml; \
		echo "  ✅ 已创建默认配置: $(CONFIGDIR)/claw-one.toml"; \
		echo "  ⚠️  请编辑配置文件，设置 OpenClaw 服务信息"; \
	else \
		echo "  ℹ️  配置文件已存在，保留现有配置"; \
	fi
	@echo "  [5/5] 安装 systemd 用户服务..."
	@sed -e "s|{INSTALL_DIR}|$(INSTALL_DIR)|g" \
	    -e "s|{HOME}|$(HOME)|g" \
	    scripts/claw-one.user.service > $(SYSTEMD_USER_DIR)/claw-one.service
	@echo "✅ 安装完成"
	@echo ""
	@echo "后续步骤:"
	@echo "  1. 编辑配置: nano $(CONFIGDIR)/claw-one.toml"
	@echo "  2. 启动服务: $(BINDIR)/claw-one-backend start"
	@echo "  3. 查看状态: $(BINDIR)/claw-one-backend status"
	@echo "  4. 配置开机自启: $(BINDIR)/claw-one-backend enable"
	@echo ""
	@echo "访问界面: http://localhost:8080"

## 用户级卸载 (uninstall) - 从 ~/claw-one 卸载
uninstall:
	@echo "🗑️  从 $(INSTALL_DIR) 卸载..."
	@echo "  [1/3] 停止并禁用服务..."
	-systemctl --user stop claw-one 2>/dev/null || true
	-systemctl --user disable claw-one 2>/dev/null || true
	@echo "  [2/3] 删除程序文件..."
	rm -rf $(BINDIR)
	rm -rf $(DATADIR)
	rm -f $(SYSTEMD_USER_DIR)/claw-one.service
	@echo "  [3/3] 处理配置文件..."
	@if [ -d "$(CONFIGDIR)" ]; then \
		echo "  ⚠️  保留配置: $(CONFIGDIR)"; \
		echo "      如需完全删除，请手动执行: rm -rf $(CONFIGDIR)"; \
	fi
	-systemctl --user daemon-reload 2>/dev/null || true
	@echo "✅ 卸载完成"

# ============================================
# 分发和打包
# ============================================

## 创建分发目录结构
dist-structure:
	@echo "📁 创建分发结构..."
	mkdir -p $(DIST_DIR)/bin
	mkdir -p $(DIST_DIR)/share/static
	mkdir -p $(DIST_DIR)/config
	mkdir -p $(DIST_DIR)/scripts
	mkdir -p $(DIST_DIR)/install

## 复制构建产物到分发目录
dist-copy: dist-structure compile
	@echo "📋 复制构建产物..."
	cp $(BACKEND_DIR)/target/release/claw-one-backend $(DIST_DIR)/bin/
	cp -r $(STATIC_DIR)/* $(DIST_DIR)/share/static/
	cp config/claw-one.toml.example $(DIST_DIR)/config/ 2>/dev/null || echo "⚠️  config/claw-one.toml.example 不存在"
	cp scripts/install.sh $(DIST_DIR)/install/
	cp scripts/uninstall.sh $(DIST_DIR)/install/
	cp scripts/claw-one.user.service $(DIST_DIR)/scripts/

## 生成分发包 (dist) - 默认安装路径 ~/claw-one
dist: dist-copy
	@echo "📦 生成分发包..."
	@echo "  创建: $(APP_NAME)-$(VERSION)-linux-amd64.tar.gz"
	tar -czf $(APP_NAME)-$(VERSION)-linux-amd64.tar.gz -C $(DIST_DIR) .
	@echo "  创建: $(APP_NAME)-$(VERSION)-install.sh (自解压安装脚本)"
	@cat scripts/self-extract-header.sh > $(APP_NAME)-$(VERSION)-install.sh
	@echo 'INSTALL_DIR="$${HOME}/claw-one"' >> $(APP_NAME)-$(VERSION)-install.sh
	@echo 'export INSTALL_DIR' >> $(APP_NAME)-$(VERSION)-install.sh
	@tail -n +2 $(APP_NAME)-$(VERSION)-linux-amd64.tar.gz >> $(APP_NAME)-$(VERSION)-install.sh
	@chmod +x $(APP_NAME)-$(VERSION)-install.sh
	@echo "✅ 分发包生成完成"
	@ls -lh $(APP_NAME)-$(VERSION)-*

# ============================================
# 开发辅助
# ============================================

## 开发模式 (热重载)
dev:
	@echo "🚀 启动开发模式..."
	@echo "  检查开发配置..."
	@if [ ! -f "$(DEV_CONFIG)" ]; then \
		echo "  ⚠️  未找到开发配置文件: $(DEV_CONFIG)"; \
		echo "  📝 正在从模板创建..."; \
		cp config/claw-one.toml.example $(DEV_CONFIG); \
		echo "  ✅ 已创建: $(DEV_CONFIG)"; \
		echo ""; \
		echo "  ⚠️  请编辑 $(DEV_CONFIG) 设置 OpenClaw 连接信息"; \
		echo "     然后重新运行 'make dev'"; \
		exit 1; \
	fi
	@echo "  前端: http://localhost:5173"
	@echo "  后端: http://localhost:8080"
	@echo "  按 Ctrl+C 停止"
	@make -j2 dev-frontend dev-backend

dev-frontend:
	cd $(FRONTEND_DIR) && $(VITE) --host --port 5173

dev-backend:
	cd $(BACKEND_DIR) && CLAW_ONE_CONFIG=$(DEV_CONFIG) $(CARGO) run

## 清理构建产物
clean:
	@echo "🧹 清理构建产物..."
	cd $(FRONTEND_DIR) && rm -rf node_modules dist
	cd $(BACKEND_DIR) && $(CARGO) clean
	rm -rf $(STATIC_DIR) $(BUILD_DIR) $(DIST_DIR)
	rm -f $(APP_NAME)-*-linux-amd64.tar.gz
	rm -f $(APP_NAME)-*-install.sh
	@echo "✅ 清理完成"

## 代码检查
check:
	@echo "🔍 代码检查..."
	cd $(BACKEND_DIR) && $(CARGO) check
	cd $(BACKEND_DIR) && $(CARGO) clippy 2>/dev/null || echo "⚠️ clippy 未安装"
	cd $(FRONTEND_DIR) && $(NPM) run lint 2>/dev/null || echo "⚠️ lint 未配置"

## 检测安装环境
check-env:
	@echo "🔍 检测安装环境..."
	@bash scripts/install.sh check

# ============================================
# 帮助信息
# ============================================

help:
	@echo "Claw One - 构建和部署工具"
	@echo ""
	@echo "📋 开发工作流"
	@echo "  make deps        - 安装依赖 (npm + cargo)"
	@echo "  make compile     - 编译项目 (前端 + 后端)"
	@echo "  make install     - 安装到 ~/claw-one (用户级)"
	@echo "  make uninstall   - 从 ~/claw-one 卸载"
	@echo ""
	@echo "🔧 开发辅助"
	@echo "  make dev         - 开发模式 (热重载，需要 config.dev.toml)"
	@echo "  make check       - 代码检查"
	@echo "  make check-env   - 检测安装环境依赖"
	@echo "  make clean       - 清理所有构建产物"
	@echo ""
	@echo "📦 分发打包"
	@echo "  make dist        - 生成 tar.gz + 自解压脚本"
	@echo "                   默认安装路径: ~/claw-one"
	@echo ""
	@echo "变量:"
	@echo "  INSTALL_DIR=~/my-path  - 自定义安装路径"
	@echo "  VERSION=1.0.0          - 自定义版本号"
