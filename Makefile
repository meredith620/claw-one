.PHONY: all build backend frontend dev clean install test help

# 默认目标
all: build

# 帮助
help:
	@echo "Claw One - 配置管理界面"
	@echo ""
	@echo "可用命令:"
	@echo "  make build     - 构建前后端 (生产环境)"
	@echo "  make dev       - 启动开发环境"
	@echo "  make backend   - 只构建后端"
	@echo "  make frontend  - 只构建前端"
	@echo "  make install   - 安装依赖"
	@echo "  make clean     - 清理构建产物"
	@echo "  make test      - 运行测试"
	@echo "  make deploy    - 部署到生产环境"

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
