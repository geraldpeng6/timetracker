#!/bin/bash

# 安装 pre-commit hook 脚本

echo "🔧 安装 TimeTracker pre-commit hook..."

# 检查是否在 Git 仓库中
if [ ! -d ".git" ]; then
    echo "❌ 错误：当前目录不是 Git 仓库"
    exit 1
fi

# 创建 .git/hooks 目录（如果不存在）
mkdir -p .git/hooks

# 复制 pre-commit hook
cp scripts/pre-commit .git/hooks/pre-commit

# 确保有执行权限
chmod +x .git/hooks/pre-commit

echo "✅ Pre-commit hook 安装成功！"
echo ""
echo "现在每次 git commit 时都会自动运行代码质量检查。"
echo ""
echo "如果需要跳过检查，可以使用："
echo "  git commit --no-verify"
echo ""
echo "如果需要卸载，可以删除："
echo "  rm .git/hooks/pre-commit"