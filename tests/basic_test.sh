#!/bin/bash

# TimeTracker 基本功能测试脚本

set -e

echo "🧪 开始 TimeTracker 基本功能测试..."

# 检查二进制文件是否存在
if [ ! -f "./target/release/timetracker" ]; then
    echo "❌ 未找到编译后的二进制文件，请先运行 'cargo build --release'"
    exit 1
fi

TIMETRACKER="./target/release/timetracker"

echo "✅ 找到二进制文件"

# 测试帮助信息
echo "📋 测试帮助信息..."
$TIMETRACKER --help > /dev/null
echo "✅ 帮助信息正常"

# 测试权限检查
echo "🔐 测试权限检查..."
$TIMETRACKER permissions
echo "✅ 权限检查完成"

# 测试守护进程启动和停止
echo "🚀 测试守护进程启动..."
$TIMETRACKER start --daemon --interval 10 --data-file test_data.json

sleep 2

echo "📊 测试状态查看..."
$TIMETRACKER status

echo "🛑 测试守护进程停止..."
$TIMETRACKER stop

# 清理测试文件
echo "🧹 清理测试文件..."
rm -f test_data.json
rm -f /tmp/timetracker.pid
rm -f /tmp/timetracker.log

echo "🎉 所有测试通过！"