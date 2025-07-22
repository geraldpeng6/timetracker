#!/bin/bash

# TimeTracker 项目清理脚本
# 删除不必要的文件和测试遗留文件

set -e

echo "🧹 TimeTracker 项目清理"
echo "======================="
echo

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
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

# 确认操作
echo "此脚本将删除以下类型的文件："
echo "  📄 重复的文档文件"
echo "  🧪 测试遗留文件"
echo "  🔧 临时脚本文件"
echo "  📦 构建产物"
echo "  🗂️  空目录"
echo
read -p "确认继续清理? (y/N): " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    print_info "清理已取消"
    exit 0
fi

echo

# 1. 删除重复和过时的文档文件
print_info "1. 清理重复和过时的文档文件..."

# 删除重复的状态报告
files_to_remove=(
    "ACTIVITY_DETECTION_SUMMARY.md"
    "ACTIVITY_DETECTION_VERIFICATION_REPORT.md"
    "BUILD_INSTALL.md"
    "COMPILATION_FIX_SUMMARY.md"
    "DAEMON_FIX_SUMMARY.md"
    "FINAL_IMPROVEMENT_REPORT.md"
    "IMPROVEMENTS.md"
    "INSTALLATION_SUMMARY.md"
    "MACOS_BLOCKING_SOLUTION.md"
    "MACOS_OPTIMIZATION_SUMMARY.md"
    "MIGRATION_GUIDE.md"
    "PROJECT_COMPLETION_REPORT.md"
    "PROJECT_STATUS.md"
    "QUICKSTART.md"
    "REFACTORING_SUMMARY.md"
    "WINDOW_MONITORING_OPTIMIZATION_PLAN.md"
    "Cargo.toml.orig"
)

for file in "${files_to_remove[@]}"; do
    if [[ -f "$file" ]]; then
        rm "$file"
        print_success "删除: $file"
    fi
done

# 2. 删除测试遗留文件
print_info "2. 清理测试遗留文件..."

test_files_to_remove=(
    "demo_activity.sh"
    "test_activity_effectiveness.sh"
    "test_activity_real.sh"
    "test_functionality.sh"
    "test_real_world_activity.rs"
    "test_sorting.sh"
    "src/main_new.rs"
    "src/main_test.rs"
)

for file in "${test_files_to_remove[@]}"; do
    if [[ -f "$file" ]]; then
        rm "$file"
        print_success "删除: $file"
    fi
done

# 删除测试结果目录
if [[ -d "test_results" ]]; then
    rm -rf "test_results"
    print_success "删除: test_results/ 目录"
fi

# 3. 删除重复的安装脚本
print_info "3. 清理重复的安装脚本..."

install_scripts_to_remove=(
    "build_and_install.sh"
    "install_fixed.sh"
    "quick_install.sh"
)

for file in "${install_scripts_to_remove[@]}"; do
    if [[ -f "$file" ]]; then
        rm "$file"
        print_success "删除: $file"
    fi
done

# 4. 删除测试二进制文件
print_info "4. 清理测试二进制文件..."

test_bins_to_remove=(
    "src/bin/integration_test.rs"
    "src/bin/test_minimal.rs"
    "src/bin/test_monitor.rs"
    "src/bin/test_tui.rs"
)

for file in "${test_bins_to_remove[@]}"; do
    if [[ -f "$file" ]]; then
        rm "$file"
        print_success "删除: $file"
    fi
done

# 如果 bin 目录为空，删除它
if [[ -d "src/bin" ]] && [[ -z "$(ls -A src/bin)" ]]; then
    rmdir "src/bin"
    print_success "删除: src/bin/ 空目录"
fi

# 5. 删除过时的核心文件
print_info "5. 清理过时的核心文件..."

core_files_to_remove=(
    "src/core/enhanced_daemon.rs"
    "src/core/permissions_check.rs"
)

for file in "${core_files_to_remove[@]}"; do
    if [[ -f "$file" ]]; then
        rm "$file"
        print_success "删除: $file"
    fi
done

# 6. 清理构建产物和缓存
print_info "6. 清理构建产物..."

if [[ -d "target" ]]; then
    # 只保留必要的构建缓存，删除具体的构建产物
    if [[ -d "target/debug" ]]; then
        find target/debug -name "timetracker*" -type f -delete 2>/dev/null || true
        print_success "清理: target/debug/ 中的构建产物"
    fi
    
    if [[ -d "target/release" ]]; then
        find target/release -name "timetracker*" -type f -delete 2>/dev/null || true
        print_success "清理: target/release/ 中的构建产物"
    fi
fi

# 7. 清理临时文件
print_info "7. 清理临时文件..."

# 删除备份文件
find . -name "*.bak" -type f -delete 2>/dev/null || true
find . -name "*.orig" -type f -delete 2>/dev/null || true
find . -name "*.tmp" -type f -delete 2>/dev/null || true

# 删除 macOS 系统文件
find . -name ".DS_Store" -type f -delete 2>/dev/null || true

print_success "清理临时文件完成"

# 8. 整理脚本目录
print_info "8. 整理脚本目录..."

scripts_to_remove=(
    "scripts/fix-clippy-warnings.sh"
    "scripts/fix-format-strings.py"
    "scripts/fix-warnings.sh"
    "scripts/project-health-check.sh"
    "scripts/run_tests.sh"
)

for file in "${scripts_to_remove[@]}"; do
    if [[ -f "$file" ]]; then
        rm "$file"
        print_success "删除: $file"
    fi
done

# 9. 清理 exports 目录
print_info "9. 清理导出目录..."

if [[ -d "exports" ]]; then
    # 如果 exports 目录为空或只包含测试文件，删除它
    if [[ -z "$(ls -A exports)" ]] || [[ $(ls exports | wc -l) -eq 0 ]]; then
        rmdir "exports" 2>/dev/null || rm -rf "exports"
        print_success "删除: exports/ 空目录"
    else
        print_warning "exports/ 目录不为空，请手动检查"
    fi
fi

# 10. 验证重要文件存在
print_info "10. 验证重要文件完整性..."

important_files=(
    "README.md"
    "CHANGELOG.md"
    "LICENSE"
    "Cargo.toml"
    "src/main.rs"
    "src/lib.rs"
    "docs/DEPLOYMENT.md"
    "docs/ACTIVITY_DETECTION.md"
)

missing_files=()
for file in "${important_files[@]}"; do
    if [[ ! -f "$file" ]]; then
        missing_files+=("$file")
    fi
done

if [[ ${#missing_files[@]} -eq 0 ]]; then
    print_success "所有重要文件完整"
else
    print_error "缺少重要文件:"
    for file in "${missing_files[@]}"; do
        echo "  - $file"
    done
fi

echo
print_info "📊 清理统计:"
echo "  🗑️  删除的文档文件: ${#files_to_remove[@]} 个"
echo "  🧪 删除的测试文件: ${#test_files_to_remove[@]} 个"
echo "  📜 删除的脚本文件: ${#install_scripts_to_remove[@]} 个"
echo "  🔧 删除的二进制文件: ${#test_bins_to_remove[@]} 个"
echo "  🧹 清理的临时文件: 所有 .bak, .orig, .tmp 文件"

echo
print_success "🎉 项目清理完成!"
echo
print_info "保留的重要文件:"
echo "  📚 文档: README.md, CHANGELOG.md, docs/"
echo "  🔧 配置: Cargo.toml, Dockerfile, docker-compose.yml"
echo "  📦 安装: install.sh, install-remote.sh, install.ps1"
echo "  🏗️  源码: src/ (完整保留)"
echo "  🧪 测试: tests/ (完整保留)"
echo "  📜 脚本: scripts/ (保留核心脚本)"
echo
print_info "项目现在更加整洁，可以进行发布!"
