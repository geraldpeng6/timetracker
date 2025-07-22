#!/bin/bash

# TimeTracker 发布脚本
# 自动化版本发布流程

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# 打印函数
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

# 检查是否在项目根目录
check_project_root() {
    if [[ ! -f "Cargo.toml" ]] || ! grep -q "name = \"timetracker\"" Cargo.toml; then
        print_error "请在 TimeTracker 项目根目录运行此脚本"
        exit 1
    fi
}

# 检查工作目录是否干净
check_git_status() {
    if [[ -n $(git status --porcelain) ]]; then
        print_error "工作目录不干净，请先提交或暂存更改"
        git status --short
        exit 1
    fi
    
    if [[ $(git rev-parse --abbrev-ref HEAD) != "main" ]]; then
        print_warning "当前不在 main 分支，是否继续？"
        read -p "继续? (y/N): " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            exit 1
        fi
    fi
}

# 检查依赖
check_dependencies() {
    local deps=("git" "cargo" "jq")
    
    for dep in "${deps[@]}"; do
        if ! command -v $dep >/dev/null 2>&1; then
            print_error "缺少依赖: $dep"
            exit 1
        fi
    done
}

# 获取当前版本
get_current_version() {
    cargo metadata --format-version 1 --no-deps | jq -r '.packages[] | select(.name == "timetracker") | .version'
}

# 验证版本格式
validate_version() {
    local version=$1
    if [[ ! $version =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
        print_error "版本格式无效: $version (应为 x.y.z)"
        exit 1
    fi
}

# 更新版本号
update_version() {
    local new_version=$1
    
    print_info "更新版本号到 $new_version..."
    
    # 更新 Cargo.toml
    sed -i.bak "s/^version = \".*\"/version = \"$new_version\"/" Cargo.toml
    rm -f Cargo.toml.bak
    
    # 更新 Homebrew formula
    if [[ -f "Formula/timetracker.rb" ]]; then
        sed -i.bak "s/version \".*\"/version \"$new_version\"/" Formula/timetracker.rb
        rm -f Formula/timetracker.rb.bak
    fi
    
    # 更新 Docker 标签
    if [[ -f "Dockerfile" ]]; then
        sed -i.bak "s/org.opencontainers.image.version=\".*\"/org.opencontainers.image.version=\"$new_version\"/" Dockerfile
        rm -f Dockerfile.bak
    fi
    
    print_success "版本号已更新"
}

# 运行测试
run_tests() {
    print_info "运行测试套件..."
    
    # 格式检查
    cargo fmt --all -- --check
    
    # Clippy 检查
    cargo clippy --all-targets --all-features -- -D warnings
    
    # 运行测试
    cargo test --all --verbose
    
    # 构建检查
    cargo build --release
    
    print_success "所有测试通过"
}

# 更新 CHANGELOG
update_changelog() {
    local version=$1
    local date=$(date +%Y-%m-%d)
    
    print_info "更新 CHANGELOG.md..."
    
    if [[ -f "CHANGELOG.md" ]]; then
        # 创建临时文件
        local temp_file=$(mktemp)
        
        # 替换 [Unreleased] 为新版本
        sed "s/## \[Unreleased\]/## [$version] - $date/" CHANGELOG.md > "$temp_file"
        
        # 在顶部添加新的 Unreleased 部分
        {
            head -n 7 "$temp_file"
            echo ""
            echo "## [Unreleased]"
            echo ""
            echo "### Added"
            echo "### Changed"
            echo "### Fixed"
            echo ""
            tail -n +8 "$temp_file"
        } > CHANGELOG.md
        
        rm -f "$temp_file"
        print_success "CHANGELOG.md 已更新"
    else
        print_warning "未找到 CHANGELOG.md"
    fi
}

# 创建提交和标签
create_commit_and_tag() {
    local version=$1
    
    print_info "创建提交和标签..."
    
    # 添加更改
    git add .
    
    # 创建提交
    git commit -m "chore: release v$version

- Update version to $version
- Update CHANGELOG.md
- Update documentation"
    
    # 创建标签
    git tag -a "v$version" -m "Release v$version

TimeTracker v$version

## Features
- Cross-platform time tracking
- Intelligent activity detection
- Video watching recognition
- TUI interface
- Export capabilities

## Installation
- One-click install scripts
- Package managers (Homebrew, APT, RPM)
- Docker containers
- Pre-compiled binaries

For detailed changelog, see CHANGELOG.md"
    
    print_success "提交和标签已创建"
}

# 推送到远程
push_release() {
    local version=$1
    
    print_info "推送到远程仓库..."
    
    # 推送提交
    git push origin main
    
    # 推送标签
    git push origin "v$version"
    
    print_success "已推送到远程仓库"
}

# 显示发布信息
show_release_info() {
    local version=$1
    
    echo
    print_success "🎉 发布 v$version 已启动!"
    echo
    print_info "接下来会发生什么："
    echo "1. GitHub Actions 将自动构建所有平台的二进制文件"
    echo "2. 创建 GitHub Release 并上传构建产物"
    echo "3. 构建并推送 Docker 镜像"
    echo "4. 生成安装包 (DEB, RPM, MSI, PKG)"
    echo
    print_info "监控构建进度："
    echo "https://github.com/geraldpeng6/timetracker/actions"
    echo
    print_info "发布完成后可用的安装方式："
    echo "- 一键安装: curl -fsSL https://raw.githubusercontent.com/geraldpeng6/timetracker/main/install-remote.sh | bash"
    echo "- Homebrew: brew install geraldpeng6/timetracker/timetracker"
    echo "- Docker: docker pull ghcr.io/geraldpeng6/timetracker:$version"
    echo "- 手动下载: https://github.com/geraldpeng6/timetracker/releases/tag/v$version"
    echo
}

# 主函数
main() {
    local new_version=$1
    
    echo "🚀 TimeTracker 发布脚本"
    echo "========================"
    echo
    
    # 检查参数
    if [[ -z $new_version ]]; then
        local current_version=$(get_current_version)
        print_info "当前版本: $current_version"
        echo
        read -p "请输入新版本号 (格式: x.y.z): " new_version
        
        if [[ -z $new_version ]]; then
            print_error "版本号不能为空"
            exit 1
        fi
    fi
    
    validate_version "$new_version"
    
    # 检查环境
    check_project_root
    check_dependencies
    check_git_status
    
    # 确认发布
    echo
    print_warning "即将发布版本: v$new_version"
    read -p "确认继续? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        print_info "发布已取消"
        exit 0
    fi
    
    # 执行发布流程
    update_version "$new_version"
    run_tests
    update_changelog "$new_version"
    create_commit_and_tag "$new_version"
    push_release "$new_version"
    show_release_info "$new_version"
}

# 处理命令行参数
case "${1:-}" in
    --help|-h)
        echo "TimeTracker 发布脚本"
        echo
        echo "用法: $0 [版本号]"
        echo
        echo "参数:"
        echo "  版本号    新版本号 (格式: x.y.z)"
        echo
        echo "选项:"
        echo "  --help, -h    显示此帮助信息"
        echo
        echo "示例:"
        echo "  $0 1.0.0      # 发布版本 1.0.0"
        echo "  $0            # 交互式输入版本号"
        exit 0
        ;;
    "")
        main
        ;;
    *)
        main "$1"
        ;;
esac
