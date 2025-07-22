# TimeTracker Makefile

.PHONY: all build install install-fast full-install clean test check fmt run-tui run-test run-integration check-permissions backup version help

# 默认目标
all: build

# 构建项目
build:
	@echo "🔨 构建 TimeTracker..."
	cargo build --release

# 安装到系统
install: build
	@echo "📦 安装 TimeTracker..."
	@chmod +x quick_install.sh
	@./quick_install.sh

# 快速安装（跳过测试）
install-fast: build
	@echo "⚡ 快速安装 TimeTracker..."
	@chmod +x quick_install.sh
	@./quick_install.sh

# 完整构建和安装
full-install:
	@echo "🚀 完整构建和安装 TimeTracker..."
	@chmod +x install.sh
	@./install.sh

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

# 运行 TUI 界面
run-tui:
	@echo "🖥️  启动 TUI 界面..."
	cargo run -- tui

# 运行测试程序
run-test:
	@echo "🧪 运行测试程序..."
	cargo run --bin test_monitor

# 运行集成测试
run-integration:
	@echo "🔬 运行集成测试..."
	cargo run --bin integration_test

# 检查权限
check-permissions:
	@echo "🔐 检查权限..."
	cargo run -- permissions check

# 备份当前版本
backup:
	@if [ -f /usr/local/bin/timetracker ]; then \
		echo "💾 备份当前版本..."; \
		sudo cp /usr/local/bin/timetracker /usr/local/bin/timetracker.backup.$$(date +%Y%m%d_%H%M%S); \
		echo "✅ 备份完成"; \
	else \
		echo "ℹ️  未找到现有安装，跳过备份"; \
	fi

# 显示版本信息
version:
	@echo "📋 版本信息:"
	@echo "项目版本: $$(grep '^version' Cargo.toml | cut -d'"' -f2)"
	@if command -v timetracker >/dev/null 2>&1; then \
		echo "已安装版本: $$(timetracker --version)"; \
		echo "安装路径: $$(which timetracker)"; \
	else \
		echo "已安装版本: 未安装"; \
	fi

# 显示帮助
help:
	@echo "TimeTracker 构建命令:"
	@echo ""
	@echo "🔨 构建命令:"
	@echo "  make build           - 构建项目"
	@echo "  make clean           - 清理构建文件"
	@echo ""
	@echo "📦 安装命令:"
	@echo "  make install         - 构建并安装"
	@echo "  make install-fast    - 快速安装（跳过测试）"
	@echo "  make full-install    - 完整构建和安装"
	@echo "  make backup          - 备份当前版本"
	@echo ""
	@echo "🧪 测试命令:"
	@echo "  make test            - 运行测试"
	@echo "  make run-test        - 运行测试程序"
	@echo "  make run-integration - 运行集成测试"
	@echo ""
	@echo "🔍 检查命令:"
	@echo "  make check           - 检查代码"
	@echo "  make fmt             - 格式化代码"
	@echo "  make check-permissions - 检查权限"
	@echo ""
	@echo "🖥️  运行命令:"
	@echo "  make run-tui         - 启动 TUI 界面"
	@echo "  make version         - 显示版本信息"
	@echo "  make help            - 显示此帮助"