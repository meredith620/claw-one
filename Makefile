.PHONY: all build hull bridge dev clean deps test help dist dist-check self-extract musl musl-dist

# 版本和架构
VERSION := $(shell git describe --tags --always --dirty 2>/dev/null || echo "0.1.0")
ARCH := $(shell uname -m)
OS := $(shell uname -s | tr '[:upper:]' '[:lower:]')
DIST_NAME := claw-one-$(VERSION)-$(ARCH)
MUSL_DIST_NAME := claw-one-$(VERSION)-$(ARCH)-musl

# 默认目标
all: build

# 帮助
help:
	@echo "Claw One - 配置管理界面"
	@echo ""
	@echo "可用命令:"
	@echo "  make deps        - 安装前后端环境依赖"
	@echo "  make dev         - 启动开发环境 (前台运行，Ctrl+C 停止)"
	@echo "  make build       - 构建前后端 (生产环境)"
	@echo "  make hull        - 只构建 hull (核心)"
	@echo "  make bridge      - 只构建 bridge (界面)"
	@echo "  make dist        - 打包独立安装包"
	@echo "  make dist-check  - 打包并生成校验和"
	@echo "  make self-extract- 创建自解压安装脚本"
	@echo "  make musl        - 静态编译 (musl)"
	@echo "  make musl-dist   - 创建 musl 自解压安装包"
	@echo "  make clean       - 清理构建产物"
	@echo "  make test        - 运行测试"

# 构建前后端
build: hull bridge
	@echo "✅ 构建完成"

# 构建 hull (原 backend)
hull:
	@echo "🔨 构建 hull..."
	cd hull && cargo build --release
	@echo "✅ hull 构建完成"

# 构建 bridge (原 frontend)
bridge:
	@echo "🎨 构建 bridge..."
	cd bridge && npm install && npx vite build
	@echo "✅ bridge 构建完成"

# 开发模式 - 前台运行，Ctrl+C 停止
dev: hull
	@echo "🚀 启动开发服务器..."
	@echo "按 Ctrl+C 停止"
	@echo ""
	@export CLAW_ONE_CONFIG="$(HOME)/claw-one/config/claw-one.toml" && \
	./hull/target/release/claw-one run

# 安装依赖
deps:
	@echo "📦 安装前后端环境依赖..."
	cd bridge && npm install
	cd hull && cargo fetch
	@echo "✅ 依赖安装完成"

# 清理
clean:
	@echo "🧹 清理构建产物..."
	cd hull && cargo clean
	cd bridge && rm -rf dist node_modules
	rm -rf dist/
	@echo "✅ 清理完成"

# 测试
test:
	@echo "🧪 运行测试..."
	cd hull && cargo test
	cd bridge && npm run test || echo "前端测试跳过"

# 打包独立安装包
dist: build
	@echo "📦 创建独立安装包..."
	@echo "版本: $(VERSION), 架构: $(ARCH)"
	@echo ""
	@rm -rf dist/$(DIST_NAME)
	@mkdir -p dist/$(DIST_NAME)/bin
	@mkdir -p dist/$(DIST_NAME)/share/static
	@mkdir -p dist/$(DIST_NAME)/share/config
	@mkdir -p dist/$(DIST_NAME)/scripts
	@cp hull/target/release/claw-one dist/$(DIST_NAME)/bin/
	@chmod +x dist/$(DIST_NAME)/bin/claw-one
	@echo "✓ 核心程序"
	@cp -r static/dist/* dist/$(DIST_NAME)/share/static/
	@echo "✓ 静态文件"
	@cp scripts/install.sh scripts/uninstall.sh scripts/check-env.sh dist/$(DIST_NAME)/scripts/
	@chmod +x dist/$(DIST_NAME)/scripts/*.sh
	@cp scripts/setup-config.sh dist/$(DIST_NAME)/bin/
	@chmod +x dist/$(DIST_NAME)/bin/setup-config.sh
	@echo "✓ 安装脚本"
	@cp scripts/README.md dist/$(DIST_NAME)/
	@echo "✓ 说明文档"
	@printf '%s\n' '[server]' 'host = "0.0.0.0"' 'port = 8080' 'log_level = "info"' '' '[openclaw]' 'openclaw_home = "~/.openclaw"' 'service_name = "openclaw"' 'health_port = 18790' 'health_timeout = 30' > dist/$(DIST_NAME)/share/config/claw-one.toml.template
	@echo "✓ 配置模板"
	@cd dist && tar czf $(DIST_NAME).tar.gz $(DIST_NAME)/
	@rm -rf dist/$(DIST_NAME)
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

# 创建自解压安装脚本
self-extract: dist
	@echo "📦 创建自解压安装脚本..."
	@echo "版本: $(VERSION), 架构: $(ARCH)"
	@echo ""
	@# 创建临时目录
	@rm -rf dist/self-extract-tmp
	@mkdir -p dist/self-extract-tmp
	@# 复制分发包内容
	@cd dist && tar xzf $(DIST_NAME).tar.gz -C self-extract-tmp/
	@# 创建 tar.gz 并转换为 base64
	@cd dist/self-extract-tmp && tar czf ../$(DIST_NAME).tmp.tar.gz *
	@cd dist && base64 -w0 $(DIST_NAME).tmp.tar.gz > $(DIST_NAME).tmp.tar.gz.b64
	@# 替换模板变量并创建自解压脚本
	@sed \
		-e 's|__VERSION__|$(VERSION)|g' \
		-e 's|__ARCH__|$(ARCH)|g' \
		-e 's|__BUILD_DATE__|$(shell date -Iseconds)|g' \
		scripts/self-extract-header.sh > dist/$(DIST_NAME)-install.sh
	@echo "" >> dist/$(DIST_NAME)-install.sh
	@echo "__ARCHIVE_MARKER__" >> dist/$(DIST_NAME)-install.sh
	@cat dist/$(DIST_NAME).tmp.tar.gz.b64 >> dist/$(DIST_NAME)-install.sh
	@echo "" >> dist/$(DIST_NAME)-install.sh
	@chmod +x dist/$(DIST_NAME)-install.sh
	@# 清理临时文件
	@rm -f dist/$(DIST_NAME).tmp.tar.gz dist/$(DIST_NAME).tmp.tar.gz.b64
	@rm -rf dist/self-extract-tmp
	@echo "✅ 自解压脚本已创建: dist/$(DIST_NAME)-install.sh"
	@echo ""
	@echo "使用方法:"
	@echo "  ./dist/$(DIST_NAME)-install.sh           # 交互式安装"
	@echo "  ./dist/$(DIST_NAME)-install.sh -y        # 自动安装"
	@echo "  ./dist/$(DIST_NAME)-install.sh --check   # 仅检查环境"

# musl 静态编译
musl:
	@echo "🔨 构建 musl 静态版本..."
	@echo "检查 musl 工具链..."
	@rustup target list --installed | grep -q x86_64-unknown-linux-musl || (echo "❌ 未安装 musl 目标，运行: rustup target add x86_64-unknown-linux-musl" && exit 1)
	@echo "✅ musl 工具链已安装"
	@echo ""
	@echo "构建 hull (musl)..."
	cd hull && cargo build --release --target x86_64-unknown-linux-musl
	@echo "✅ musl 构建完成"
	@ls -lh hull/target/x86_64-unknown-linux-musl/release/claw-one

# musl 分发包
musl-dist: bridge musl
	@echo "📦 创建 musl 分发包..."
	@echo "版本: $(VERSION), 架构: $(ARCH)-musl"
	@echo ""
	@rm -rf dist/$(MUSL_DIST_NAME)
	@mkdir -p dist/$(MUSL_DIST_NAME)/bin
	@mkdir -p dist/$(MUSL_DIST_NAME)/share/static
	@mkdir -p dist/$(MUSL_DIST_NAME)/share/config
	@mkdir -p dist/$(MUSL_DIST_NAME)/scripts
	@cp hull/target/x86_64-unknown-linux-musl/release/claw-one dist/$(MUSL_DIST_NAME)/bin/
	@chmod +x dist/$(MUSL_DIST_NAME)/bin/claw-one
	@echo "✓ 核心程序 (musl静态链接)"
	@cp -r static/dist/* dist/$(MUSL_DIST_NAME)/share/static/
	@echo "✓ 静态文件"
	@cp scripts/install.sh scripts/uninstall.sh scripts/check-env.sh dist/$(MUSL_DIST_NAME)/scripts/
	@chmod +x dist/$(MUSL_DIST_NAME)/scripts/*.sh
	@cp scripts/setup-config.sh dist/$(MUSL_DIST_NAME)/bin/
	@chmod +x dist/$(MUSL_DIST_NAME)/bin/setup-config.sh
	@echo "✓ 安装脚本"
	@cp scripts/README.md dist/$(MUSL_DIST_NAME)/
	@echo "✓ 说明文档"
	@printf '%s\n' '[server]' 'host = "0.0.0.0"' 'port = 8080' 'log_level = "info"' '' '[openclaw]' 'openclaw_home = "~/.openclaw"' 'service_name = "openclaw"' 'health_port = 18790' 'health_timeout = 30' > dist/$(MUSL_DIST_NAME)/share/config/claw-one.toml.template
	@echo "✓ 配置模板"
	@cd dist && tar czf $(MUSL_DIST_NAME).tar.gz $(MUSL_DIST_NAME)/
	@rm -rf dist/$(MUSL_DIST_NAME)
	@echo ""
	@echo "✅ musl 安装包已创建: dist/$(MUSL_DIST_NAME).tar.gz"

# musl 自解压脚本
musl-self-extract: musl-dist
	@echo "📦 创建 musl 自解压安装脚本..."
	@echo "版本: $(VERSION), 架构: $(ARCH)-musl"
	@echo ""
	@rm -rf dist/self-extract-tmp
	@mkdir -p dist/self-extract-tmp
	@cd dist && tar xzf $(MUSL_DIST_NAME).tar.gz -C self-extract-tmp/
	@cd dist/self-extract-tmp && tar czf ../$(MUSL_DIST_NAME).tmp.tar.gz *
	@cd dist && base64 -w0 $(MUSL_DIST_NAME).tmp.tar.gz > $(MUSL_DIST_NAME).tmp.tar.gz.b64
	@sed \
		-e 's|__VERSION__|$(VERSION)|g' \
		-e 's|__ARCH__|$(ARCH)-musl|g' \
		-e 's|__BUILD_DATE__|$(shell date -Iseconds)|g' \
		scripts/self-extract-header.sh > dist/$(MUSL_DIST_NAME)-install.sh
	@echo "" >> dist/$(MUSL_DIST_NAME)-install.sh
	@echo "__ARCHIVE_MARKER__" >> dist/$(MUSL_DIST_NAME)-install.sh
	@cat dist/$(MUSL_DIST_NAME).tmp.tar.gz.b64 >> dist/$(MUSL_DIST_NAME)-install.sh
	@echo "" >> dist/$(MUSL_DIST_NAME)-install.sh
	@chmod +x dist/$(MUSL_DIST_NAME)-install.sh
	@rm -f dist/$(MUSL_DIST_NAME).tmp.tar.gz dist/$(MUSL_DIST_NAME).tmp.tar.gz.b64
	@rm -rf dist/self-extract-tmp
	@echo "✅ musl 自解压脚本已创建: dist/$(MUSL_DIST_NAME)-install.sh"
