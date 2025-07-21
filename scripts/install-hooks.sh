#!/bin/bash

# TimeTracker Pre-commit Hooks 安装脚本

set -e

echo "🔧 安装 Git pre-commit hooks..."

# 创建 hooks 目录（如果不存在）
HOOKS_DIR=".git/hooks"
if [ ! -d "$HOOKS_DIR" ]; then
    mkdir -p "$HOOKS_DIR"
fi

# 创建 pre-commit hook
cat > "$HOOKS_DIR/pre-commit" << 'EOF'
#!/bin/bash

# TimeTracker Pre-commit Hook
# 在提交前自动检查代码质量

set -e

echo "🔍 运行 pre-commit 检查..."

# 检查是否有 Rust 代码变更
if git diff --cached --name-only | grep -q '\.rs$'; then
    echo "📝 检查 Rust 代码格式..."
    
    # 格式化代码
    if ! cargo fmt --check; then
        echo "❌ 代码格式不符合规范，正在自动格式化..."
        cargo fmt
        echo "✅ 代码已格式化，请重新提交"
        exit 1
    fi
    
    # 运行 Clippy 检查
    echo "🔍 运行 Clippy 代码检查..."
    if ! cargo clippy -- -D warnings; then
        echo "❌ Clippy 检查失败，请修复警告后重新提交"
        exit 1
    fi
    
    # 运行测试
    echo "🧪 运行测试..."
    if ! cargo test; then
        echo "❌ 测试失败，请修复后重新提交"
        exit 1
    fi
fi

# 检查提交信息格式（如果有的话）
if [ -f ".gitmessage" ]; then
    echo "📋 检查提交信息格式..."
    # 这里可以添加提交信息格式检查
fi

echo "✅ Pre-commit 检查通过！"
EOF

# 使 hook 可执行
chmod +x "$HOOKS_DIR/pre-commit"

echo "✅ Pre-commit hooks 安装完成！"
echo ""
echo "现在每次 git commit 都会自动运行代码检查"
echo "如需跳过检查，使用: git commit --no-verify"
echo ""
echo "包含的检查项目："
echo "  - Rust 代码格式化 (cargo fmt)"
echo "  - Clippy 代码质量检查"
echo "  - 单元测试"