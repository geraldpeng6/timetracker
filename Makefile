# TimeTracker Makefile

.PHONY: all build install clean test help

# 默认目标
all: build

# 构建项目
build:
	@echo "🔨 构建 TimeTracker..."
	cargo build --release

# 安装到系统
install: build
	@echo "📦 安装 TimeTracker..."
	./build_and_install.sh --skip-clean

# 快速安装（跳过测试）
install-fast: build
	@echo "⚡ 快速安装 TimeTracker..."
	./build_and_install.sh --skip-tests --skip-clean

# 完整构建和安装
full-install:
	@echo "🚀 完整构建和安装 TimeTracker..."
	./build_and_install.sh

# 清理构建文件
clean:
	@echo "🧹 清理构建文件..."
	cargo clean
	rm -rf release/

# 运行测试
test:
	@echo "🧪 运行测试..."
	cargo test

# 检查代码
check:
	@echo "🔍 检查代码..."
	cargo check
	cargo clippy

# 格式化代码
fmt:
	@echo "✨ 格式化代码..."
	cargo fmt

# 安装 Git pre-commit hooks
setup-hooks:
	@echo "🔧 安装 pre-commit hooks..."
	@chmod +x scripts/install-hooks.sh
	@./scripts/install-hooks.sh

# 显示帮助
help:
	@echo "TimeTracker 构建命令:"
	@echo "  make build        - 构建项目"
	@echo "  make install      - 构建并安装"
	@echo "  make install-fast - 快速安装（跳过测试）"
	@echo "  make full-install - 完整构建和安装"
	@echo "  make clean        - 清理构建文件"
	@echo "  make test         - 运行测试"
	@echo "  make check        - 检查代码"
	@echo "  make fmt          - 格式化代码"
	@echo "  make setup-hooks  - 安装 pre-commit hooks"
	@echo "  make help         - 显示此帮助"