#!/bin/bash

# TimeTracker 自动编译和安装脚本
# 编译 release 版本并更新本地 /usr/local/bin/timetracker

set -e  # 遇到错误立即退出

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# 配置
PROJECT_NAME="timetracker"
INSTALL_DIR="/usr/local/bin"
BACKUP_DIR="$HOME/.timetracker_backup"

# 打印带颜色的消息
print_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

print_step() {
    echo -e "${PURPLE}🔧 $1${NC}"
}

# 检查是否在项目根目录
check_project_directory() {
    if [[ ! -f "Cargo.toml" ]]; then
        print_error "未找到 Cargo.toml 文件，请确保在项目根目录运行此脚本"
        exit 1
    fi
    
    if ! grep -q "name = \"timetracker\"" Cargo.toml; then
        print_error "这不是 timetracker 项目目录"
        exit 1
    fi
    
    print_success "项目目录验证通过"
}

# 检查 Rust 环境
check_rust_environment() {
    print_step "检查 Rust 环境..."
    
    if ! command -v cargo &> /dev/null; then
        print_error "未找到 cargo 命令，请先安装 Rust"
        print_info "安装命令: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
        exit 1
    fi
    
    print_success "Rust 环境检查通过"
    print_info "Cargo 版本: $(cargo --version)"
}

# 清理之前的构建
clean_build() {
    print_step "清理之前的构建..."
    cargo clean
    print_success "构建清理完成"
}

# 运行测试
run_tests() {
    print_step "运行测试..."
    
    # 检查编译
    if ! cargo check --all-targets; then
        print_error "代码检查失败"
        exit 1
    fi
    print_success "代码检查通过"
    
    # 运行集成测试（可选）
    if [[ "$1" == "--with-tests" ]]; then
        print_step "运行集成测试..."
        if cargo run --bin integration_test; then
            print_success "集成测试通过"
        else
            print_warning "集成测试失败，但继续构建"
        fi
    fi
}

# 编译 release 版本
build_release() {
    print_step "编译 release 版本..."
    
    # 显示编译进度
    if cargo build --release; then
        print_success "Release 版本编译成功"
    else
        print_error "Release 版本编译失败"
        exit 1
    fi
    
    # 验证二进制文件
    if [[ ! -f "target/release/$PROJECT_NAME" ]]; then
        print_error "未找到编译后的二进制文件"
        exit 1
    fi
    
    # 显示文件信息
    local binary_size=$(du -h "target/release/$PROJECT_NAME" | cut -f1)
    print_info "二进制文件大小: $binary_size"
}

# 备份现有版本
backup_existing() {
    if [[ -f "$INSTALL_DIR/$PROJECT_NAME" ]]; then
        print_step "备份现有版本..."
        
        # 创建备份目录
        mkdir -p "$BACKUP_DIR"
        
        # 获取当前版本信息
        local current_version=""
        if "$INSTALL_DIR/$PROJECT_NAME" --version &> /dev/null; then
            current_version=$("$INSTALL_DIR/$PROJECT_NAME" --version 2>/dev/null || echo "unknown")
        fi
        
        # 备份文件
        local backup_file="$BACKUP_DIR/${PROJECT_NAME}_$(date +%Y%m%d_%H%M%S)"
        cp "$INSTALL_DIR/$PROJECT_NAME" "$backup_file"
        
        print_success "已备份到: $backup_file"
        if [[ -n "$current_version" ]]; then
            print_info "当前版本: $current_version"
        fi
    else
        print_info "未找到现有安装，跳过备份"
    fi
}

# 安装新版本
install_binary() {
    print_step "安装新版本..."
    
    # 检查权限
    if [[ ! -w "$INSTALL_DIR" ]]; then
        print_warning "需要管理员权限安装到 $INSTALL_DIR"
        if sudo cp "target/release/$PROJECT_NAME" "$INSTALL_DIR/"; then
            print_success "使用 sudo 安装成功"
        else
            print_error "安装失败"
            exit 1
        fi
    else
        if cp "target/release/$PROJECT_NAME" "$INSTALL_DIR/"; then
            print_success "安装成功"
        else
            print_error "安装失败"
            exit 1
        fi
    fi
    
    # 设置执行权限
    chmod +x "$INSTALL_DIR/$PROJECT_NAME"
    print_success "已设置执行权限"
}

# 验证安装
verify_installation() {
    print_step "验证安装..."
    
    if command -v "$PROJECT_NAME" &> /dev/null; then
        local new_version=$("$PROJECT_NAME" --version 2>/dev/null || echo "unknown")
        print_success "安装验证成功"
        print_info "新版本: $new_version"
        print_info "安装路径: $(which $PROJECT_NAME)"
    else
        print_error "安装验证失败，请检查 PATH 环境变量"
        print_info "手动添加到 PATH: export PATH=\"$INSTALL_DIR:\$PATH\""
        exit 1
    fi
}

# 显示使用说明
show_usage() {
    print_step "使用说明:"
    echo -e "${CYAN}# 查看帮助${NC}"
    echo "timetracker --help"
    echo ""
    echo -e "${CYAN}# 启动 TUI 界面${NC}"
    echo "timetracker tui"
    echo ""
    echo -e "${CYAN}# 检查权限${NC}"
    echo "timetracker permissions check"
    echo ""
    echo -e "${CYAN}# 启动守护进程${NC}"
    echo "timetracker start"
    echo ""
    print_info "更多命令请查看: timetracker --help"
}

# 清理函数
cleanup() {
    if [[ $? -ne 0 ]]; then
        print_error "脚本执行失败"
        print_info "如需恢复，备份文件位于: $BACKUP_DIR"
    fi
}

# 主函数
main() {
    echo -e "${CYAN}"
    echo "╔══════════════════════════════════════╗"
    echo "║        TimeTracker 安装脚本          ║"
    echo "║     编译 Release 版本并自动安装      ║"
    echo "╚══════════════════════════════════════╝"
    echo -e "${NC}"
    
    # 设置错误处理
    trap cleanup EXIT
    
    # 解析参数
    local with_tests=false
    local force_clean=false
    
    while [[ $# -gt 0 ]]; do
        case $1 in
            --with-tests)
                with_tests=true
                shift
                ;;
            --clean)
                force_clean=true
                shift
                ;;
            --help|-h)
                echo "用法: $0 [选项]"
                echo "选项:"
                echo "  --with-tests    运行集成测试"
                echo "  --clean         强制清理构建"
                echo "  --help, -h      显示此帮助信息"
                exit 0
                ;;
            *)
                print_error "未知选项: $1"
                exit 1
                ;;
        esac
    done
    
    # 执行安装步骤
    check_project_directory
    check_rust_environment
    
    if [[ "$force_clean" == true ]]; then
        clean_build
    fi
    
    if [[ "$with_tests" == true ]]; then
        run_tests --with-tests
    else
        run_tests
    fi
    
    build_release
    backup_existing
    install_binary
    verify_installation
    show_usage
    
    print_success "🎉 TimeTracker 安装完成！"
}

# 运行主函数
main "$@"
