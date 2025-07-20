#!/bin/bash

# 版本更新脚本
# 用法: ./scripts/update-version.sh <new_version>
# 例如: ./scripts/update-version.sh 0.2.2

if [ $# -eq 0 ]; then
    echo "用法: $0 <new_version>"
    echo "例如: $0 0.2.2"
    exit 1
fi

NEW_VERSION=$1

echo "更新版本号到 $NEW_VERSION..."

# 更新 Cargo.toml
sed -i '' "s/^version = \".*\"/version = \"$NEW_VERSION\"/" Cargo.toml

echo "✅ 已更新 Cargo.toml"

# 清理并重新编译
echo "🧹 清理编译缓存..."
cargo clean

echo "🔨 编译 release 版本..."
cargo build --release

if [ $? -eq 0 ]; then
    echo "📦 安装新版本..."
    cp ./target/release/timetracker ~/.local/bin/timetracker
    
    echo "✅ 版本更新完成！"
    echo "当前版本: $(timetracker --version)"
else
    echo "❌ 编译失败，版本更新中止"
    exit 1
fi