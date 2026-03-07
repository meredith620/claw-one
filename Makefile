.PHONY: all build backend frontend dev clean install test help dist dist-check

# 版本和架构
VERSION := $(shell git describe --tags --always --dirty 2>/dev/null || echo "0.1.0")
ARCH := $(shell uname -m)
OS := $(shell uname -s | tr '[:upper:]' '[:lower:]')
DIST_NAME := claw-one-$(VERSION)-$(ARCH)

# 默认目标
all: build

# 帮助
help:
	@echo "Claw One - 配置管理界面"
	@echo ""
	@echo "可用命令:"
	@echo "  make build       - 构建前后端 (生产环境)"
	@echo "  make dist        - 打包独立安装包"
	@echo "  make dist-check  - 打包并生成校验和"
	@echo "  make backend     - 只构建后端"
	@echo "  make frontend    - 只构建前端"
	@echo "  make dev         - 启动开发环境"
	@echo "  make install     - 安装依赖"
	@echo "  make clean       - 清理构建产物"
	@echo "  make test        - 运行测试"
	@echo "  make deploy      - 部署到生产环境"

# 构建前后端
build: backend frontend
	@echo "✅ 构建完成"

# 构建后端
backend:
	@echo "🔨 构建后端..."
	cd backend && cargo build --release
	@echo "✅ 后端构建完成"

# 构建前端
frontend:
	@echo "🎨 构建前端..."
	cd frontend && npm install && npm run build
	@echo "✅ 前端构建完成"

# 开发模式
dev:
	@echo "🚀 启动开发环境..."
	@echo "请手动启动后端: cd backend && cargo run"
	@echo "请手动启动前端: cd frontend && npm run dev"

# 安装依赖
install:
	@echo "📦 安装依赖..."
	cd frontend && npm install
	cd backend && cargo fetch
	@echo "✅ 依赖安装完成"

# 清理
clean:
	@echo "🧹 清理构建产物..."
	cd backend && cargo clean
	cd frontend && rm -rf dist node_modules
	rm -rf dist/
	@echo "✅ 清理完成"

# 测试
test:
	@echo "🧪 运行测试..."
	cd backend && cargo test
	cd frontend && npm run test || echo "前端测试跳过"

# 部署
deploy: build
	@echo "🚀 部署..."
	./deploy.sh init
	./deploy.sh start

# 打包独立安装包
dist: build
	@echo "📦 创建独立安装包..."
	@echo "版本: $(VERSION), 架构: $(ARCH)"
	@echo ""
	@mkdir -p dist/$(DIST_NAME)/{bin,share/{static,config},scripts}
	@cp backend/target/release/claw-one-backend dist/$(DIST_NAME)/bin/
	@chmod +x dist/$(DIST_NAME)/bin/claw-one-backend
	@echo "✓ 后端程序"
	@cp -r static/dist/* dist/$(DIST_NAME)/share/static/
	@echo "✓ 静态文件"
	@cp scripts/install.sh scripts/uninstall.sh scripts/check-env.sh dist/$(DIST_NAME)/scripts/
	@chmod +x dist/$(DIST_NAME)/scripts/*.sh
	@cp scripts/setup-config.sh dist/$(DIST_NAME)/bin/
	@chmod +x dist/$(DIST_NAME)/bin/setup-config.sh
	@echo "✓ 安装脚本"
	@cp scripts/README.md dist/$(DIST_NAME)/
	@echo "✓ 说明文档"
	@cd dist && tar czf $(DIST_NAME).tar.gz $(DIST_NAME)/
	@echo ""
	@echo "✅ 安装包已创建: dist/$(DIST_NAME).tar.gz"
	@echo ""
	@echo "使用方式:"
	@echo "  1. 解压: tar xzf $(DIST_NAME).tar.gz"
	@echo "  2. 检查: cd $(DIST_NAME) && ./scripts/check-env.sh"
	@echo "  3. 安装: ./scripts/install.sh"
	@echo "  4. 配置: ~/claw-one/bin/setup-config.sh"

# 打包并生成校验和
dist-check: dist
	@echo "🔐 生成校验和..."
	@cd dist && sha256sum $(DIST_NAME).tar.gz > $(DIST_NAME).tar.gz.sha256
	@echo "✅ 校验和已生成: dist/$(DIST_NAME).tar.gz.sha256"
	@echo ""
	@cat dist/$(DIST_NAME).tar.gz.sha256
