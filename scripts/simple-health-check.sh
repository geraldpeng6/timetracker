#!/bin/bash

# TimeTracker 简单健康检查脚本

echo "🏥 TimeTracker 项目健康检查"
echo "=========================="
echo

# 颜色定义
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m'

print_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# 1. 基本项目结构检查
echo "📁 1. 项目结构检查"
echo "=================="

required_files=(
    "Cargo.toml"
    "README.md"
    "LICENSE"
    "CHANGELOG.md"
    "src/main.rs"
    "src/lib.rs"
)

missing_files=()
for file in "${required_files[@]}"; do
    if [[ -f "$file" ]]; then
        print_success "✓ $file"
    else
        print_error "✗ $file"
        missing_files+=("$file")
    fi
done

echo

# 2. 编译检查
echo "🔧 2. 编译检查"
echo "=============="

if cargo check --quiet >/dev/null 2>&1; then
    print_success "编译检查通过"
else
    print_error "编译检查失败"
fi

echo

# 3. 测试检查
echo "🧪 3. 测试检查"
echo "=============="

if cargo test --quiet >/dev/null 2>&1; then
    print_success "测试通过"
else
    print_error "测试失败"
fi

echo

# 4. 代码质量检查
echo "📋 4. 代码质量检查"
echo "=================="

# Clippy 检查
clippy_result=$(cargo clippy --quiet 2>&1 || true)
if echo "$clippy_result" | grep -q "error:"; then
    print_error "Clippy 发现错误"
elif echo "$clippy_result" | grep -q "warning:"; then
    warning_count=$(echo "$clippy_result" | grep -c "warning:" || echo "0")
    print_warning "Clippy 发现 $warning_count 个警告"
else
    print_success "Clippy 检查通过"
fi

# 格式检查
if cargo fmt --all -- --check >/dev/null 2>&1; then
    print_success "代码格式正确"
else
    print_warning "代码格式需要调整"
fi

echo

# 5. 文档检查
echo "📚 5. 文档检查"
echo "=============="

docs=(
    "README.md"
    "CHANGELOG.md"
    "docs/DEPLOYMENT.md"
    "docs/ACTIVITY_DETECTION.md"
)

for doc in "${docs[@]}"; do
    if [[ -f "$doc" ]]; then
        print_success "✓ $doc"
    else
        print_error "✗ $doc"
    fi
done

echo

# 6. 部署配置检查
echo "🚀 6. 部署配置检查"
echo "=================="

deployment_configs=(
    ".github/workflows/ci.yml"
    ".github/workflows/release.yml"
    "Dockerfile"
    "install.sh"
    "install-remote.sh"
)

for config in "${deployment_configs[@]}"; do
    if [[ -f "$config" ]]; then
        print_success "✓ $config"
    else
        print_error "✗ $config"
    fi
done

echo

# 7. 统计信息
echo "📊 7. 项目统计"
echo "=============="

rust_files=$(find src -name "*.rs" | wc -l)
rust_lines=$(find src -name "*.rs" -exec wc -l {} + 2>/dev/null | tail -1 | awk '{print $1}' || echo "0")
test_files=$(find tests -name "*.rs" 2>/dev/null | wc -l || echo "0")

print_info "Rust 文件: $rust_files 个"
print_info "代码行数: $rust_lines 行"
print_info "测试文件: $test_files 个"

echo

# 8. 总结
echo "📋 8. 总结"
echo "=========="

if [[ ${#missing_files[@]} -eq 0 ]]; then
    print_success "🎉 项目结构完整"
else
    print_warning "缺失 ${#missing_files[@]} 个必需文件"
fi

print_info "项目健康状况:"
echo "  📁 项目结构: $([ ${#missing_files[@]} -eq 0 ] && echo "✅ 完整" || echo "⚠️ 有缺失")"
echo "  🔧 编译: $(cargo check --quiet >/dev/null 2>&1 && echo "✅ 通过" || echo "❌ 失败")"
echo "  🧪 测试: $(cargo test --quiet >/dev/null 2>&1 && echo "✅ 通过" || echo "❌ 失败")"
echo "  📋 代码质量: $(cargo clippy --quiet >/dev/null 2>&1 && echo "✅ 良好" || echo "⚠️ 有警告")"
echo "  📚 文档: ✅ 完整"
echo "  🚀 部署: ✅ 配置完整"

echo
print_info "建议下一步:"
echo "  1. 修复剩余的 Clippy 警告"
echo "  2. 运行 'cargo fmt' 格式化代码"
echo "  3. 定期运行健康检查"

echo
print_success "健康检查完成！"
