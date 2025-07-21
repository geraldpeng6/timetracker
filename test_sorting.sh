#!/bin/bash

# TimeTracker TUI 排序功能测试脚本

echo "🧪 TimeTracker TUI 排序功能测试"
echo "================================"
echo ""

echo "📋 测试说明："
echo "1. 启动 TimeTracker TUI"
echo "2. 按 '2' 切换到数据分析页面"
echo "3. 按 'v' 切换视图模式 (应用程序 -> 窗口 -> 最近活动)"
echo "4. 按 's' 循环排序字段"
echo "5. 按 'r' 切换排序顺序 (升序/降序)"
echo "6. 观察排序是否正常工作，窗口是否还会闪动"
echo ""

echo "🔧 修复内容："
echo "✅ 修复了排序功能 (s键) 在窗口视图中的问题"
echo "✅ 修复了倒序功能 (r键) 的问题"
echo "✅ 修复了相同时间窗口的闪动问题"
echo "✅ 增强了排序稳定性"
echo ""

echo "🚀 启动 TimeTracker TUI..."
echo "按 Ctrl+C 退出测试"
echo ""

# 构建并运行
cargo build --release
if [ $? -eq 0 ]; then
    ./target/release/timetracker tui
else
    echo "❌ 构建失败，请检查代码"
    exit 1
fi