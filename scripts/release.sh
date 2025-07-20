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

# 项目信息
PROJECT_NAME="timetracker"
CARGO_TOML="Cargo.toml"

# 帮助信息
show_help() {
    echo "TimeTracker 发布脚本"
    echo ""
    echo "用法: $0 [选项] <版本类型>"
    echo ""
    echo "版本类型:"
    echo "  major    主版本号 (1.0.0 -> 2.0.0)"
    echo "  minor    次版本号 (1.0.0 -> 1.1.0)"
    echo "  patch    补丁版本 (1.0.0 -> 1.0.1)"
    echo "  <版本号>  指定版本号 (如: 1.2.3)"
    echo ""
    echo "选项:"
    echo "  -h, --help     显示帮助信息"
    echo "  -n, --dry-run  预览模式，不实际执行"
    echo "  --no-push     不推送到远程仓库"
    echo "  --no-build    跳过构建步骤"
    echo ""
    echo "示例:"
    echo "  $0 patch              # 发布补丁版本"
    echo "  $0 minor              # 发布次版本"
    echo "  $0 1.2.3              # 发布指定版本"
    echo "  $0 --dry-run patch    # 预览补丁版本发布"
}

# 解析命令行参数
DRY_RUN=false
NO_PUSH=false
NO_BUILD=false
VERSION_TYPE=""

while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help)
            show_help
            exit 0
            ;;
        -n|--dry-run)
            DRY_RUN=true
            shift
            ;;
        --no-push)
            NO_PUSH=true
            shift
            ;;
        --no-build)
            NO_BUILD=true
            shift
            ;;
        major|minor|patch)
            VERSION_TYPE="$1"
            shift
            ;;
        [0-9]*.[0-9]*.[0-9]*)
            VERSION_TYPE="$1"
            shift
            ;;
        *)
            echo -e "${RED}错误: 未知参数 '$1'${NC}"
            show_help
            exit 1
            ;;
    esac
done

if [ -z "$VERSION_TYPE" ]; then
    echo -e "${RED}错误: 请指定版本类型${NC}"
    show_help
    exit 1
fi

# 获取当前版本
get_current_version() {
    grep '^version = ' "$CARGO_TOML" | sed 's/version = "\(.*\)"/\1/'
}

# 计算新版本
calculate_new_version() {
    local current_version="$1"
    local version_type="$2"
    
    if [[ "$version_type" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
        echo "$version_type"
        return
    fi
    
    local major minor patch
    IFS='.' read -r major minor patch <<< "$current_version"
    
    case "$version_type" in
        major)
            echo "$((major + 1)).0.0"
            ;;
        minor)
            echo "$major.$((minor + 1)).0"
            ;;
        patch)
            echo "$major.$minor.$((patch + 1))"
            ;;
        *)
            echo -e "${RED}错误: 无效的版本类型 '$version_type'${NC}"
            exit 1
            ;;
    esac
}

# 更新版本号
update_version() {
    local new_version="$1"
    
    if [ "$DRY_RUN" = true ]; then
        echo -e "${YELLOW}[预览] 将更新 $CARGO_TOML 中的版本号为 $new_version${NC}"
        return
    fi
    
    # 使用 sed 更新版本号
    if [[ "$OSTYPE" == "darwin"* ]]; then
        # macOS
        sed -i '' "s/^version = \".*\"/version = \"$new_version\"/" "$CARGO_TOML"
    else
        # Linux
        sed -i "s/^version = \".*\"/version = \"$new_version\"/" "$CARGO_TOML"
    fi
    
    echo -e "${GREEN}✓ 已更新版本号为 $new_version${NC}"
}

# 运行完整测试套件
run_tests() {
    if [ "$NO_BUILD" = true ]; then
        echo -e "${YELLOW}跳过测试${NC}"
        return
    fi
    
    echo -e "${BLUE}运行完整测试套件...${NC}"
    
    if [ "$DRY_RUN" = true ]; then
        echo -e "${YELLOW}[预览] 代码格式检查${NC}"
        echo -e "${YELLOW}[预览] Clippy 静态分析${NC}"
        echo -e "${YELLOW}[预览] 单元测试${NC}"
        echo -e "${YELLOW}[预览] 编译检查${NC}"
        return
    fi
    
    # 1. 代码格式检查
    echo -e "${BLUE}  📝 检查代码格式...${NC}"
    if ! cargo fmt --all -- --check; then
        echo -e "${RED}❌ 代码格式检查失败${NC}"
        echo -e "${YELLOW}💡 运行 'cargo fmt --all' 修复格式问题${NC}"
        exit 1
    fi
    echo -e "${GREEN}  ✓ 代码格式检查通过${NC}"
    
    # 2. Clippy 静态分析
    echo -e "${BLUE}  🔍 运行 Clippy 静态分析...${NC}"
    if ! cargo clippy --all-targets --all-features -- -D warnings; then
        echo -e "${RED}❌ Clippy 检查失败${NC}"
        echo -e "${YELLOW}💡 请修复上述警告和错误${NC}"
        exit 1
    fi
    echo -e "${GREEN}  ✓ Clippy 检查通过${NC}"
    
    # 3. 单元测试
    echo -e "${BLUE}  🧪 运行单元测试...${NC}"
    if ! cargo test --verbose; then
        echo -e "${RED}❌ 单元测试失败${NC}"
        exit 1
    fi
    echo -e "${GREEN}  ✓ 单元测试通过${NC}"
    
    # 4. 编译检查
    echo -e "${BLUE}  🔨 编译检查...${NC}"
    if ! cargo build --verbose; then
        echo -e "${RED}❌ 编译失败${NC}"
        exit 1
    fi
    echo -e "${GREEN}  ✓ 编译检查通过${NC}"
    
    echo -e "${GREEN}✓ 所有测试通过${NC}"
}

# 构建项目
build_project() {
    if [ "$NO_BUILD" = true ]; then
        echo -e "${YELLOW}跳过构建${NC}"
        return
    fi
    
    echo -e "${BLUE}构建项目...${NC}"
    
    if [ "$DRY_RUN" = true ]; then
        echo -e "${YELLOW}[预览] cargo build --release${NC}"
        return
    fi
    
    cargo build --release
    echo -e "${GREEN}✓ 构建完成${NC}"
}

# 创建 Git 标签
create_git_tag() {
    local version="$1"
    local tag="v$version"
    
    echo -e "${BLUE}创建 Git 标签...${NC}"
    
    if [ "$DRY_RUN" = true ]; then
        echo -e "${YELLOW}[预览] git add .${NC}"
        echo -e "${YELLOW}[预览] git commit -m \"chore: bump version to $version\"${NC}"
        echo -e "${YELLOW}[预览] git tag -a $tag -m \"Release $tag\"${NC}"
        return
    fi
    
    # 检查是否有未提交的更改
    if ! git diff --quiet || ! git diff --cached --quiet; then
        git add .
        git commit -m "chore: bump version to $version"
    fi
    
    # 创建标签
    git tag -a "$tag" -m "Release $tag"
    echo -e "${GREEN}✓ 已创建标签 $tag${NC}"
}

# 推送到远程仓库
push_to_remote() {
    local version="$1"
    local tag="v$version"
    
    if [ "$NO_PUSH" = true ]; then
        echo -e "${YELLOW}跳过推送到远程仓库${NC}"
        return
    fi
    
    echo -e "${BLUE}推送到远程仓库...${NC}"
    
    if [ "$DRY_RUN" = true ]; then
        echo -e "${YELLOW}[预览] git push origin main${NC}"
        echo -e "${YELLOW}[预览] git push origin $tag${NC}"
        return
    fi
    
    git push origin main
    git push origin "$tag"
    echo -e "${GREEN}✓ 已推送到远程仓库${NC}"
}

# 主函数
main() {
    echo -e "${GREEN}TimeTracker 发布脚本${NC}"
    echo "=========================="
    
    # 检查是否在 Git 仓库中
    if ! git rev-parse --git-dir > /dev/null 2>&1; then
        echo -e "${RED}错误: 当前目录不是 Git 仓库${NC}"
        exit 1
    fi
    
    # 检查是否有未提交的更改（除了版本更新）
    if ! git diff --quiet HEAD -- . ':!Cargo.toml' ':!Cargo.lock'; then
        echo -e "${RED}错误: 有未提交的更改，请先提交或暂存${NC}"
        exit 1
    fi
    
    # 获取当前版本
    local current_version
    current_version=$(get_current_version)
    echo -e "${BLUE}当前版本: $current_version${NC}"
    
    # 计算新版本
    local new_version
    new_version=$(calculate_new_version "$current_version" "$VERSION_TYPE")
    echo -e "${BLUE}新版本: $new_version${NC}"
    
    if [ "$DRY_RUN" = true ]; then
        echo -e "${YELLOW}=== 预览模式 ===${NC}"
    fi
    
    # 确认发布
    if [ "$DRY_RUN" = false ]; then
        echo ""
        read -p "确认发布版本 $new_version? (y/N) " -n 1 -r
        echo ""
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            echo "取消发布"
            exit 0
        fi
    fi
    
    # 执行发布步骤
    run_tests
    build_project
    update_version "$new_version"
    create_git_tag "$new_version"
    push_to_remote "$new_version"
    
    echo ""
    if [ "$DRY_RUN" = true ]; then
        echo -e "${GREEN}🎉 预览完成!${NC}"
        echo "使用 '$0 $VERSION_TYPE' 执行实际发布"
    else
        echo -e "${GREEN}🎉 发布完成!${NC}"
        echo "版本 $new_version 已发布"
        echo "GitHub Actions 将自动构建和发布二进制文件"
        echo "查看发布状态: https://github.com/geraldpeng6/timetracker/actions"
    fi
}

# 运行主函数
main "$@"