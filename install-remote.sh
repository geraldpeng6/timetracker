#!/bin/bash

# TimeTracker 远程安装脚本
# 从 GitHub Releases 下载预编译二进制文件
# 支持 Linux, macOS, Windows (WSL)

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 版本信息
VERSION="${VERSION:-0.2.2}"
REPO="geraldpeng6/timetracker"
BASE_URL="https://github.com/${REPO}/releases/download/v${VERSION}"

# 打印带颜色的消息
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

# 检测操作系统和架构
detect_platform() {
    local os=$(uname -s | tr '[:upper:]' '[:lower:]')
    local arch=$(uname -m)
    
    case $os in
        linux*)
            OS="linux"
            ;;
        darwin*)
            OS="macos"
            ;;
        mingw*|msys*|cygwin*)
            OS="windows"
            ;;
        *)
            print_error "不支持的操作系统: $os"
            exit 1
            ;;
    esac
    
    case $arch in
        x86_64|amd64)
            ARCH="x86_64"
            ;;
        aarch64|arm64)
            ARCH="aarch64"
            ;;
        *)
            print_error "不支持的架构: $arch"
            exit 1
            ;;
    esac
    
    print_info "检测到平台: $OS-$ARCH"
}

# 检查依赖
check_dependencies() {
    local deps=("curl" "tar")
    
    for dep in "${deps[@]}"; do
        if ! command -v $dep >/dev/null 2>&1; then
            print_error "缺少依赖: $dep"
            print_info "请安装 $dep 后重试"
            exit 1
        fi
    done
    
    # Linux 特定依赖检查
    if [ "$OS" = "linux" ]; then
        print_info "检查 Linux 依赖..."
        local missing_deps=()
        
        # 检查 X11 库
        if ! ldconfig -p | grep -q libX11 2>/dev/null; then
            missing_deps+=("libx11-dev")
        fi
        
        if ! ldconfig -p | grep -q libxcb 2>/dev/null; then
            missing_deps+=("libxcb1-dev")
        fi
        
        if [ ${#missing_deps[@]} -gt 0 ]; then
            print_warning "建议安装以下依赖以获得最佳体验:"
            for dep in "${missing_deps[@]}"; do
                echo "  - $dep"
            done
            print_info "Ubuntu/Debian: sudo apt-get install ${missing_deps[*]}"
            print_info "CentOS/RHEL: sudo yum install libX11-devel libxcb-devel"
        fi
    fi
}

# 下载并安装
install_timetracker() {
    local filename
    local url
    
    if [ "$OS" = "windows" ]; then
        filename="timetracker-windows-${ARCH}.exe.zip"
        url="${BASE_URL}/${filename}"
    else
        filename="timetracker-${OS}-${ARCH}.tar.gz"
        url="${BASE_URL}/${filename}"
    fi
    
    print_info "下载 TimeTracker v${VERSION}..."
    print_info "URL: $url"
    
    # 创建临时目录
    local temp_dir=$(mktemp -d)
    cd "$temp_dir"
    
    # 下载文件
    if ! curl -L -f -o "$filename" "$url"; then
        print_error "下载失败，请检查网络连接或版本号"
        print_info "可用版本请查看: https://github.com/${REPO}/releases"
        exit 1
    fi
    
    print_success "下载完成"
    
    # 解压文件
    print_info "解压文件..."
    if [ "$OS" = "windows" ]; then
        if command -v unzip >/dev/null 2>&1; then
            unzip -q "$filename"
        else
            print_error "需要 unzip 命令来解压 Windows 包"
            exit 1
        fi
        binary_name="timetracker.exe"
    else
        tar -xzf "$filename"
        binary_name="timetracker"
    fi
    
    # 确定安装目录
    local install_dir
    if [ "$OS" = "windows" ]; then
        install_dir="$HOME/bin"
    else
        if [ -w "/usr/local/bin" ] 2>/dev/null; then
            install_dir="/usr/local/bin"
        elif [ -w "$HOME/.local/bin" ] 2>/dev/null; then
            install_dir="$HOME/.local/bin"
        else
            install_dir="$HOME/.local/bin"
        fi
    fi
    
    # 创建安装目录
    mkdir -p "$install_dir"
    
    # 备份现有版本
    if [ -f "$install_dir/$binary_name" ]; then
        print_info "备份现有版本..."
        cp "$install_dir/$binary_name" "$install_dir/${binary_name}.backup.$(date +%Y%m%d_%H%M%S)"
    fi
    
    # 安装二进制文件
    print_info "安装到 $install_dir..."
    cp "$binary_name" "$install_dir/"
    chmod +x "$install_dir/$binary_name"
    
    # 清理临时文件
    cd /
    rm -rf "$temp_dir"
    
    print_success "TimeTracker 安装完成!"
    
    # 检查 PATH
    if ! echo "$PATH" | grep -q "$install_dir"; then
        print_warning "$install_dir 不在 PATH 中"
        print_info "请将以下行添加到你的 shell 配置文件 (~/.bashrc, ~/.zshrc 等):"
        echo "export PATH=\"$install_dir:\$PATH\""
        echo
        print_info "然后运行: source ~/.bashrc (或重新打开终端)"
    fi
}

# 验证安装
verify_installation() {
    print_info "验证安装..."
    
    if command -v timetracker >/dev/null 2>&1; then
        local version_output=$(timetracker --version 2>/dev/null || echo "unknown")
        print_success "TimeTracker 安装成功!"
        print_info "版本: $version_output"
        return 0
    else
        print_error "安装验证失败"
        print_info "请检查 PATH 设置或手动运行二进制文件"
        return 1
    fi
}

# 显示使用说明
show_usage() {
    echo
    print_success "🎉 安装完成!"
    echo
    print_info "快速开始:"
    echo "  timetracker --help              # 查看帮助"
    echo "  timetracker start               # 开始时间追踪"
    echo "  timetracker tui                 # 打开交互界面"
    echo "  timetracker activity status     # 查看活跃度状态"
    echo "  timetracker activity config     # 查看活跃度配置"
    echo
    
    if [ "$OS" = "macos" ]; then
        print_info "macOS 权限设置:"
        echo "  timetracker permissions check   # 检查权限状态"
        echo "  timetracker permissions request # 请求必要权限"
        echo
        print_warning "macOS 用户需要授予辅助功能权限才能监控窗口活动"
    fi
    
    if [ "$OS" = "linux" ]; then
        print_info "Linux 用户注意:"
        echo "  确保已安装 X11 相关库以获得最佳体验"
        echo "  某些 Wayland 环境可能需要额外配置"
        echo
    fi
    
    print_info "更多信息:"
    echo "  GitHub: https://github.com/${REPO}"
    echo "  文档: https://github.com/${REPO}#readme"
    echo "  活跃度检测: https://github.com/${REPO}/blob/main/docs/ACTIVITY_DETECTION.md"
    echo
}

# 主函数
main() {
    echo "🎯 TimeTracker 远程安装脚本"
    echo "============================="
    echo
    
    # 检查是否为 root 用户
    if [ "$EUID" -eq 0 ]; then
        print_warning "不建议以 root 用户运行此脚本"
        read -p "是否继续? (y/N): " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            exit 1
        fi
    fi
    
    detect_platform
    check_dependencies
    install_timetracker
    
    if verify_installation; then
        show_usage
    else
        print_error "安装过程中出现问题"
        print_info "请尝试手动下载并安装: https://github.com/${REPO}/releases"
        exit 1
    fi
}

# 处理命令行参数
case "${1:-}" in
    --help|-h)
        echo "TimeTracker 远程安装脚本"
        echo
        echo "用法: $0 [选项]"
        echo
        echo "选项:"
        echo "  --help, -h     显示此帮助信息"
        echo "  --version, -v  显示版本信息"
        echo
        echo "环境变量:"
        echo "  VERSION        指定要安装的版本 (默认: $VERSION)"
        echo
        echo "示例:"
        echo "  $0                    # 安装默认版本"
        echo "  VERSION=0.2.1 $0     # 安装指定版本"
        echo
        echo "一键安装命令:"
        echo "  curl -fsSL https://raw.githubusercontent.com/${REPO}/main/install-remote.sh | bash"
        exit 0
        ;;
    --version|-v)
        echo "TimeTracker 远程安装脚本 v1.0.0"
        echo "默认安装版本: v$VERSION"
        exit 0
        ;;
    "")
        main
        ;;
    *)
        print_error "未知选项: $1"
        print_info "使用 --help 查看帮助"
        exit 1
        ;;
esac
