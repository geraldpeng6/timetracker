#!/bin/bash

# TimeTracker 本地测试脚本
# 在每次修改代码后运行此脚本进行完整检查

set -e  # 遇到错误立即退出

echo "🚀 开始 TimeTracker 本地测试..."
echo "=================================="

# 1. 代码格式检查
echo "📝 检查代码格式..."
if cargo fmt --all -- --check; then
    echo "✅ 代码格式检查通过"
else
    echo "❌ 代码格式检查失败"
    echo "💡 运行 'cargo fmt --all' 修复格式问题"
    exit 1
fi

# 2. Clippy 静态分析
echo ""
echo "🔍 运行 Clippy 静态分析..."
if cargo clippy --all-targets --all-features -- -D warnings; then
    echo "✅ Clippy 检查通过"
else
    echo "❌ Clippy 检查失败"
    echo "💡 请修复上述警告和错误"
    exit 1
fi

# 3. 单元测试
echo ""
echo "🧪 运行单元测试..."
if cargo test --verbose; then
    echo "✅ 单元测试通过"
else
    echo "❌ 单元测试失败"
    exit 1
fi

# 4. 编译检查
echo ""
echo "🔨 编译检查..."
if cargo build --verbose; then
    echo "✅ 编译成功"
else
    echo "❌ 编译失败"
    exit 1
fi

# 5. Release 编译检查
echo ""
echo "🚀 Release 编译检查..."
if cargo build --release; then
    echo "✅ Release 编译成功"
else
    echo "❌ Release 编译失败"
    exit 1
fi

# 6. 基本功能测试
echo ""
echo "⚡ 基本功能测试..."

# 检查帮助信息
if ./target/release/timetracker --help > /dev/null 2>&1; then
    echo "✅ 帮助信息正常"
else
    echo "❌ 帮助信息异常"
    exit 1
fi

# 检查版本信息
if ./target/release/timetracker --version > /dev/null 2>&1; then
    echo "✅ 版本信息正常"
else
    echo "❌ 版本信息异常"
    exit 1
fi

# 检查权限命令
if ./target/release/timetracker permissions > /dev/null 2>&1; then
    echo "✅ 权限检查命令正常"
else
    echo "❌ 权限检查命令异常"
    exit 1
fi

# 检查状态命令
if ./target/release/timetracker status > /dev/null 2>&1; then
    echo "✅ 状态检查命令正常"
else
    echo "❌ 状态检查命令异常"
    exit 1
fi

echo ""
echo "🎉 所有测试通过！"
echo "=================================="
echo "✅ 代码格式检查"
echo "✅ Clippy 静态分析"
echo "✅ 单元测试"
echo "✅ 编译检查"
echo "✅ Release 编译"
echo "✅ 基本功能测试"
echo ""
echo "🚀 代码已准备好提交！"