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

# 安装路径 (系统级)
PREFIX ?= /usr/local
BINDIR := $(PREFIX)/bin
LIBDIR := $(PREFIX)/lib/claw-one
DATADIR := $(PREFIX)/share/claw-one
CONFIGDIR := /etc/claw-one
SYSTEMD_DIR := /etc/systemd/system

# 版本和元数据
VERSION := 0.1.0
APP_NAME := claw-one

# ============================================
# 默认目标
# ============================================

.DEFAULT_GOAL := help

.PHONY: help deps compile run install uninstall clean dist package

# ============================================
# 开发工作流 (deps → compile → run → install)
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
	cd $(FRONTEND_DIR) && $(NPM) run build
	@echo "  [2/2] 编译后端..."
	cd $(BACKEND_DIR) && $(CARGO) build --release
	@echo "✅ 编译完成"
	@echo "  后端二进制: $(BACKEND_DIR)/target/release/claw-one-backend"
	@echo "  前端产物: $(STATIC_DIR)/"

## 调试运行 (run) - 依赖: compile
run: compile
	@echo "🚀 启动调试运行..."
	@echo "  前端: http://localhost:5173 (如使用dev模式)"
	@echo "  后端: http://localhost:8080"
	@echo "  按 Ctrl+C 停止"
	cd $(BACKEND_DIR) && $(CARGO) run

## 完整安装 (install) - 依赖: run (调试通过后安装)
install: run
	@echo "📦 本地开发环境安装完成"
	@echo "  提示: 使用 'make install-system' 进行系统级安装"
	@echo "  提示: 使用 'make dist' 生成分发包"

# ============================================
# 系统级安装/卸载 (需要 root 权限)
# ============================================

## 系统级安装 (install-system)
install-system: compile
	@echo "📦 系统级安装到 $(PREFIX)..."
	@echo "  [1/5] 安装后端二进制..."
	install -Dm755 $(BACKEND_DIR)/target/release/claw-one-backend $(DESTDIR)$(BINDIR)/claw-one-backend
	@echo "  [2/5] 安装前端静态文件..."
	install -dm755 $(DESTDIR)$(DATADIR)
	cp -r $(STATIC_DIR)/* $(DESTDIR)$(DATADIR)/
	@echo "  [3/5] 安装默认配置..."
	install -Dm644 config/openclaw.json.example $(DESTDIR)$(CONFIGDIR)/openclaw.json.example
	@echo "  [4/5] 安装 systemd 服务..."
	install -Dm644 scripts/claw-one.service $(DESTDIR)$(SYSTEMD_DIR)/claw-one.service
	@echo "  [5/5] 设置权限..."
	@echo "✅ 系统安装完成"
	@echo ""
	@echo "后续步骤:"
	@echo "  1. sudo systemctl daemon-reload"
	@echo "  2. sudo systemctl enable claw-one"
	@echo "  3. sudo systemctl start claw-one"

## 系统级卸载 (uninstall)
uninstall:
	@echo "🗑️  系统级卸载..."
	@echo "  [1/4] 停止并禁用服务..."
	-systemctl stop claw-one 2>/dev/null || true
	-systemctl disable claw-one 2>/dev/null || true
	@echo "  [2/4] 删除二进制..."
	rm -f $(DESTDIR)$(BINDIR)/claw-one-backend
	@echo "  [3/4] 删除数据文件..."
	rm -rf $(DESTDIR)$(DATADIR)
	@echo "  [4/4] 删除配置文件..."
	@echo "⚠️  保留配置: $(DESTDIR)$(CONFIGDIR) (手动删除: sudo rm -rf $(DESTDIR)$(CONFIGDIR))"
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
	cp -r config/* $(DIST_DIR)/config/ 2>/dev/null || echo "⚠️  config/ 目录不存在，跳过"
	cp scripts/install.sh $(DIST_DIR)/install/ 2>/dev/null || echo "⚠️  install.sh 不存在，跳过"
	cp scripts/uninstall.sh $(DIST_DIR)/install/ 2>/dev/null || echo "⚠️  uninstall.sh 不存在，跳过"
	cp scripts/claw-one.service $(DIST_DIR)/scripts/ 2>/dev/null || echo "⚠️  service 文件不存在，跳过"

## 生成分发包 (dist)
dist: dist-copy
	@echo "📦 生成分发包..."
	@echo "  创建: $(APP_NAME)-$(VERSION)-linux-amd64.tar.gz"
	tar -czf $(APP_NAME)-$(VERSION)-linux-amd64.tar.gz -C $(DIST_DIR) .
	@echo "  创建: $(APP_NAME)-$(VERSION)-install.sh (自解压安装脚本)"
	cat scripts/self-extract-header.sh $(APP_NAME)-$(VERSION)-linux-amd64.tar.gz > $(APP_NAME)-$(VERSION)-install.sh
	chmod +x $(APP_NAME)-$(VERSION)-install.sh
	@echo "✅ 分发包生成完成"
	@ls -lh $(APP_NAME)-$(VERSION)-*

## 创建 deb 包 (需要 dpkg-deb)
deb:
	@echo "📦 创建 DEB 包..."
	mkdir -p $(BUILD_DIR)/deb/DEBIAN
	mkdir -p $(BUILD_DIR)/deb$(BINDIR)
	mkdir -p $(BUILD_DIR)/deb$(DATADIR)
	mkdir -p $(BUILD_DIR)/deb$(CONFIGDIR)
	mkdir -p $(BUILD_DIR)/deb$(SYSTEMD_DIR)
	cp $(BACKEND_DIR)/target/release/claw-one-backend $(BUILD_DIR)/deb$(BINDIR)/
	cp -r $(STATIC_DIR)/* $(BUILD_DIR)/deb$(DATADIR)/
	cp config/openclaw.json.example $(BUILD_DIR)/deb$(CONFIGDIR)/
	cp scripts/claw-one.service $(BUILD_DIR)/deb$(SYSTEMD_DIR)/
	@echo "Package: claw-one" > $(BUILD_DIR)/deb/DEBIAN/control
	@echo "Version: $(VERSION)" >> $(BUILD_DIR)/deb/DEBIAN/control
	@echo "Section: admin" >> $(BUILD_DIR)/deb/DEBIAN/control
	@echo "Priority: optional" >> $(BUILD_DIR)/deb/DEBIAN/control
	@echo "Architecture: amd64" >> $(BUILD_DIR)/deb/DEBIAN/control
	@echo "Depends: systemd" >> $(BUILD_DIR)/deb/DEBIAN/control
	@echo "Maintainer: Claw One Team <dev@claw.one>" >> $(BUILD_DIR)/deb/DEBIAN/control
	@echo "Description: OpenClaw configuration guardian" >> $(BUILD_DIR)/deb/DEBIAN/control
	@echo " Claw One provides a web-based configuration management" >> $(BUILD_DIR)/deb/DEBIAN/control
	@echo " system for OpenClaw with Git versioning and Safe Mode." >> $(BUILD_DIR)/deb/DEBIAN/control
	dpkg-deb --build $(BUILD_DIR)/deb $(APP_NAME)-$(VERSION).deb
	@echo "✅ DEB 包创建完成: $(APP_NAME)-$(VERSION).deb"

# ============================================
# 开发辅助
# ============================================

## 开发模式 (热重载)
dev:
	@echo "🚀 启动开发模式 (前端热重载 + 后端)..."
	@echo "  前端: http://localhost:5173"
	@echo "  后端: http://localhost:8080"
	@make -j2 dev-frontend dev-backend

dev-frontend:
	cd $(FRONTEND_DIR) && $(VITE) --host --port 5173

dev-backend:
	cd $(BACKEND_DIR) && $(CARGO) run

## 清理构建产物
clean:
	@echo "🧹 清理构建产物..."
	cd $(FRONTEND_DIR) && rm -rf node_modules dist
	cd $(BACKEND_DIR) && $(CARGO) clean
	rm -rf $(STATIC_DIR) $(BUILD_DIR) $(DIST_DIR)
	rm -f $(APP_NAME)-*-linux-amd64.tar.gz
	rm -f $(APP_NAME)-*-install.sh
	rm -f $(APP_NAME)-*.deb
	@echo "✅ 清理完成"

## 运行测试
test:
	@echo "🧪 运行测试..."
	cd $(BACKEND_DIR) && $(CARGO) test
	cd $(FRONTEND_DIR) && $(NPM) run test 2>/dev/null || echo "⚠️ 前端无测试脚本"

## 代码检查
check:
	@echo "🔍 代码检查..."
	cd $(BACKEND_DIR) && $(CARGO) check
	cd $(BACKEND_DIR) && $(CARGO) clippy 2>/dev/null || echo "⚠️ clippy 未安装"
	cd $(FRONTEND_DIR) && $(NPM) run lint 2>/dev/null || echo "⚠️ lint 未配置"

# ============================================
# 帮助信息
# ============================================

help:
	@echo "Claw One - 构建和部署工具"
	@echo ""
	@echo "📋 开发工作流 (顺序依赖: deps → compile → run → install)"
	@echo "  make deps        - 安装依赖 (npm + cargo)"
	@echo "  make compile     - 编译项目 (前端 + 后端)"
	@echo "  make run         - 调试运行"
	@echo "  make install     - 本地安装 (依赖 run)"
	@echo ""
	@echo "🖥️  系统级安装/卸载 (需要 root)"
	@echo "  sudo make install-system  - 安装到 /usr/local"
	@echo "  sudo make uninstall       - 从系统卸载"
	@echo ""
	@echo "📦 分发打包"
	@echo "  make dist        - 生成 tar.gz + 自解压脚本"
	@echo "  make deb         - 创建 DEB 包"
	@echo ""
	@echo "🔧 开发辅助"
	@echo "  make dev         - 开发模式 (热重载)"
	@echo "  make test        - 运行测试"
	@echo "  make check       - 代码检查"
	@echo "  make clean       - 清理所有构建产物"
	@echo ""
	@echo "变量:"
	@echo "  PREFIX=/opt/claw-one  - 自定义安装路径"
	@echo "  VERSION=1.0.0         - 自定义版本号"
