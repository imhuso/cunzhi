# AI Review Makefile
# 提供传统的 make 命令支持

.PHONY: help dev build build-cli build-app clean install test release

# 默认目标
all: build

# 显示帮助信息
help:
	@echo "AI Review 构建系统"
	@echo ""
	@echo "可用命令:"
	@echo "  make dev        - 启动开发环境"
	@echo "  make build      - 完整编译"
	@echo "  make build-cli  - 仅编译 CLI"
	@echo "  make build-app  - 仅编译应用"
	@echo "  make clean      - 清理构建文件"
	@echo "  make install    - 安装依赖"
	@echo "  make test       - 运行测试"
	@echo "  make release    - 创建发布版本"
	@echo "  make help       - 显示此帮助"

# 开发模式
dev:
	@echo "🔧 启动开发环境..."
	./dev.sh

# 完整编译
build: install
	@echo "🚀 完整编译..."
	./build.sh

# 仅编译 CLI
build-cli:
	@echo "📦 编译 CLI..."
	cargo build --release --bin ai-review-cli

# 仅编译应用
build-app: install
	@echo "🖥️  编译应用..."
	npm run tauri build

# 清理构建文件
clean:
	@echo "🧹 清理构建文件..."
	cargo clean
	rm -rf node_modules dist release
	@echo "✅ 清理完成"

# 安装依赖
install:
	@echo "📦 安装依赖..."
	@if [ ! -d "node_modules" ]; then \
		npm install; \
	fi

# 运行测试
test:
	@echo "🧪 运行测试..."
	cargo test

# 创建发布版本
release: clean build
	@echo "📦 创建发布版本..."
	@if [ -d "release" ]; then \
		echo "✅ 发布文件已创建在 release/ 目录"; \
		ls -la release/; \
	fi

# 快速编译
quick:
	@echo "⚡ 快速编译..."
	./quick-build.sh
