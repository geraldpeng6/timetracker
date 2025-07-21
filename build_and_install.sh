#!/bin/bash

# TimeTracker 构建和安装脚本
# 用于构建最新版本并安装到本地系统

set -e  # 遇到错误立即退出

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 日志函数
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# 检查依赖
check_dependencies() {
    log_info "检查依赖..."
    
    if ! command -v cargo &> /dev/null; then
        log_error "Rust/Cargo 未安装，请先安装 Rust"
        exit 1
    fi
    
    if ! command -v git &> /dev/null; then
        log_error "Git 未安装，请先安装 Git"
        exit 1
    fi
    
    log_success "依赖检查完成"
}

# 清理旧的构建文件
clean_build() {
    log_info "清理旧的构建文件..."
    
    if [ -d "target" ]; then
        rm -rf target
        log_success "已清理 target 目录"
    fi
    
    if [ -d "release" ]; then
        rm -rf release
        log_success "已清理 release 目录"
    fi
    
    # 清理可能存在的数据文件
    if [ -f "timetracker_data.json" ]; then
        rm -f timetracker_data.json
        log_success "已清理数据文件"
    fi
}

# 更新依赖
update_dependencies() {
    log_info "更新 Rust 工具链和依赖..."
    
    rustup update
    cargo update
    
    log_success "依赖更新完成"
}

# 构建项目
build_project() {
    log_info "开始构建项目..."
    
    # 优化构建
    export RUSTFLAGS="-C target-cpu=native"
    
    cargo build --release
    
    if [ $? -eq 0 ]; then
        log_success "项目构建成功"
    else
        log_error "项目构建失败"
        exit 1
    fi
}

# 运行测试
run_tests() {
    log_info "运行测试..."
    
    cargo test --release
    
    if [ $? -eq 0 ]; then
        log_success "所有测试通过"
    else
        log_warning "部分测试失败，但继续安装"
    fi
}

# 安装到系统
install_to_system() {
    log_info "安装到系统..."
    
    local binary_path="target/release/timetracker"
    local install_dir="/usr/local/bin"
    
    if [ ! -f "$binary_path" ]; then
        log_error "构建的二进制文件不存在: $binary_path"
        exit 1
    fi
    
    # 检查是否需要 sudo
    if [ ! -w "$install_dir" ]; then
        log_info "需要管理员权限安装到 $install_dir"
        sudo cp "$binary_path" "$install_dir/"
        sudo chmod +x "$install_dir/timetracker"
    else
        cp "$binary_path" "$install_dir/"
        chmod +x "$install_dir/timetracker"
    fi
    
    log_success "已安装到 $install_dir/timetracker"
}

# 验证安装
verify_installation() {
    log_info "验证安装..."
    
    if command -v timetracker &> /dev/null; then
        local version=$(timetracker --version 2>/dev/null || echo "未知版本")
        log_success "安装验证成功: $version"
        
        log_info "可用命令:"
        echo "  timetracker start    - 启动时间跟踪"
        echo "  timetracker stop     - 停止时间跟踪"
        echo "  timetracker status   - 查看状态"
        echo "  timetracker stats    - 查看统计信息"
        echo "  timetracker tui      - 启动交互界面"
        echo "  timetracker --help   - 查看帮助"
    else
        log_error "安装验证失败，timetracker 命令不可用"
        log_info "请检查 /usr/local/bin 是否在 PATH 中"
        exit 1
    fi
}

# 清理守护进程
cleanup_daemon() {
    log_info "清理可能存在的守护进程..."
    
    # 尝试停止现有的守护进程
    if command -v timetracker &> /dev/null; then
        timetracker stop 2>/dev/null || true
    fi
    
    # 查找并终止可能的进程
    local pids=$(pgrep -f "timetracker" 2>/dev/null || true)
    if [ ! -z "$pids" ]; then
        log_info "发现运行中的 timetracker 进程，正在终止..."
        echo "$pids" | xargs kill -TERM 2>/dev/null || true
        sleep 2
        echo "$pids" | xargs kill -KILL 2>/dev/null || true
    fi
    
    log_success "守护进程清理完成"
}

# 主函数
main() {
    echo "========================================"
    echo "    TimeTracker 构建和安装脚本"
    echo "========================================"
    echo
    
    # 检查是否在项目根目录
    if [ ! -f "Cargo.toml" ]; then
        log_error "请在项目根目录运行此脚本"
        exit 1
    fi
    
    # 解析命令行参数
    SKIP_TESTS=false
    SKIP_CLEAN=false
    
    while [[ $# -gt 0 ]]; do
        case $1 in
            --skip-tests)
                SKIP_TESTS=true
                shift
                ;;
            --skip-clean)
                SKIP_CLEAN=true
                shift
                ;;
            -h|--help)
                echo "用法: $0 [选项]"
                echo "选项:"
                echo "  --skip-tests    跳过测试"
                echo "  --skip-clean    跳过清理"
                echo "  -h, --help      显示帮助"
                exit 0
                ;;
            *)
                log_error "未知选项: $1"
                exit 1
                ;;
        esac
    done
    
    # 执行构建和安装步骤
    check_dependencies
    cleanup_daemon
    
    if [ "$SKIP_CLEAN" = false ]; then
        clean_build
    fi
    
    update_dependencies
    build_project
    
    if [ "$SKIP_TESTS" = false ]; then
        run_tests
    fi
    
    install_to_system
    verify_installation
    
    echo
    log_success "TimeTracker 构建和安装完成！"
    echo
    log_info "现在你可以运行 'timetracker start' 开始使用"
}

# 运行主函数
main "$@"