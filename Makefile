# TimeTracker Makefile
# 简化常用的开发任务

.PHONY: help test fmt clippy build release clean install dev check all setup-hooks

# 默认目标
help:
	@echo "TimeTracker 开发工具"
	@echo "===================="
	@echo ""
	@echo "可用命令:"
	@echo "  make test        - 运行完整测试套件"
	@echo "  make fmt         - 格式化代码"
	@echo "  make clippy      - 运行 Clippy 静态分析"
	@echo "  make build       - 编译项目"
	@echo "  make release     - 编译 Release 版本"
	@echo "  make clean       - 清理构建文件"
	@echo "  make install     - 安装到系统"
	@echo "  make dev         - 开发模式（格式化 + 检查 + 测试）"
	@echo "  make check       - 快速检查（格式 + Clippy）"
	@echo "  make all         - 完整流程（检查 + 测试 + 构建）"
	@echo "  make setup-hooks - 安装 Git pre-commit hooks"
	@echo ""

# 格式化代码
fmt:
	@echo "📝 格式化代码..."
	cargo fmt --all

# 运行 Clippy
clippy:
	@echo "🔍 运行 Clippy 静态分析..."
	cargo clippy --all-targets --all-features -- -D warnings

# 运行测试
test:
	@echo "🧪 运行测试..."
	cargo test --verbose

# 编译项目
build:
	@echo "🔨 编译项目..."
	cargo build

# 编译 Release 版本
release:
	@echo "🚀 编译 Release 版本..."
	cargo build --release

# 清理构建文件
clean:
	@echo "🧹 清理构建文件..."
	cargo clean

# 安装到系统
install: release
	@echo "📦 安装到系统..."
	cargo install --path .

# 快速检查（格式 + Clippy）
check:
	@echo "⚡ 快速检查..."
	@echo "📝 检查代码格式..."
	@cargo fmt --all -- --check || (echo "❌ 代码格式检查失败，运行 'make fmt' 修复" && exit 1)
	@echo "✅ 代码格式检查通过"
	@echo "🔍 运行 Clippy..."
	@cargo clippy --all-targets --all-features -- -D warnings
	@echo "✅ Clippy 检查通过"

# 开发模式（格式化 + 检查 + 测试）
dev: fmt check test
	@echo "🎉 开发检查完成！"

# 完整流程（检查 + 测试 + 构建）
all: check test build
	@echo "🎉 所有检查通过，项目已准备就绪！"

# 运行完整测试套件（使用脚本）
test-full:
	@echo "🚀 运行完整测试套件..."
	./scripts/test.sh

# 启动守护进程
start:
	@echo "🚀 启动 TimeTracker 守护进程..."
	./target/release/timetracker start

# 停止守护进程
stop:
	@echo "🛑 停止 TimeTracker 守护进程..."
	./target/release/timetracker stop

# 查看状态
status:
	@echo "📊 查看 TimeTracker 状态..."
	./target/release/timetracker status

# 查看统计
stats:
	@echo "📈 查看使用统计..."
	./target/release/timetracker stats

# 安装 Git pre-commit hooks
setup-hooks:
	@echo "🔧 安装 Git pre-commit hooks..."
	./scripts/install-hooks.sh