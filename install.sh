#!/bin/bash

# TimeTracker 安装脚本
# 支持 Linux, macOS 和 Windows (WSL)

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 项目信息
REPO="yourusername/timetracker"
BINARY_NAME="timetracker"

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
            echo -e "${RED}错误: 不支持的操作系统 $os${NC}"
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
            echo -e "${RED}错误: 不支持的架构 $arch${NC}"
            exit 1
            ;;
    esac
    
    if [ "$OS" = "windows" ]; then
        BINARY_NAME="${BINARY_NAME}.exe"
        ASSET_NAME="timetracker-windows-${ARCH}.exe"
    else
        ASSET_NAME="timetracker-${OS}-${ARCH}"
    fi
}

# 获取最新版本
get_latest_version() {
    echo -e "${BLUE}获取最新版本信息...${NC}"
    LATEST_VERSION=$(curl -s "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')
    
    if [ -z "$LATEST_VERSION" ]; then
        echo -e "${RED}错误: 无法获取最新版本信息${NC}"
        exit 1
    fi
    
    echo -e "${GREEN}最新版本: $LATEST_VERSION${NC}"
}

# 下载二进制文件
download_binary() {
    local download_url="https://github.com/${REPO}/releases/download/${LATEST_VERSION}/${ASSET_NAME}"
    local temp_file="/tmp/${BINARY_NAME}"
    
    echo -e "${BLUE}下载 $ASSET_NAME...${NC}"
    echo "下载地址: $download_url"
    
    if command -v curl >/dev/null 2>&1; then
        curl -L -o "$temp_file" "$download_url"
    elif command -v wget >/dev/null 2>&1; then
        wget -O "$temp_file" "$download_url"
    else
        echo -e "${RED}错误: 需要 curl 或 wget 来下载文件${NC}"
        exit 1
    fi
    
    if [ ! -f "$temp_file" ]; then
        echo -e "${RED}错误: 下载失败${NC}"
        exit 1
    fi
    
    chmod +x "$temp_file"
    echo -e "${GREEN}下载完成${NC}"
}

# 安装二进制文件
install_binary() {
    local temp_file="/tmp/${BINARY_NAME}"
    local install_dir
    
    # 确定安装目录
    if [ -w "/usr/local/bin" ]; then
        install_dir="/usr/local/bin"
    elif [ -w "$HOME/.local/bin" ]; then
        install_dir="$HOME/.local/bin"
        mkdir -p "$install_dir"
    else
        install_dir="$HOME/bin"
        mkdir -p "$install_dir"
    fi
    
    echo -e "${BLUE}安装到 $install_dir...${NC}"
    
    # 如果需要 sudo 权限
    if [ "$install_dir" = "/usr/local/bin" ] && [ ! -w "/usr/local/bin" ]; then
        sudo mv "$temp_file" "$install_dir/$BINARY_NAME"
    else
        mv "$temp_file" "$install_dir/$BINARY_NAME"
    fi
    
    echo -e "${GREEN}安装完成!${NC}"
    
    # 检查 PATH
    if ! echo "$PATH" | grep -q "$install_dir"; then
        echo -e "${YELLOW}警告: $install_dir 不在 PATH 中${NC}"
        echo "请将以下行添加到你的 shell 配置文件 (~/.bashrc, ~/.zshrc 等):"
        echo "export PATH=\"$install_dir:\$PATH\""
    fi
}

# 验证安装
verify_installation() {
    echo -e "${BLUE}验证安装...${NC}"
    
    if command -v $BINARY_NAME >/dev/null 2>&1; then
        local version=$($BINARY_NAME --version 2>/dev/null || echo "unknown")
        echo -e "${GREEN}✓ TimeTracker 安装成功!${NC}"
        echo "版本: $version"
        echo ""
        echo "使用方法:"
        echo "  $BINARY_NAME --help          # 查看帮助"
        echo "  $BINARY_NAME permissions     # 检查权限"
        echo "  $BINARY_NAME start           # 开始追踪"
        echo "  $BINARY_NAME stats           # 查看统计"
    else
        echo -e "${RED}✗ 安装验证失败${NC}"
        echo "请检查 $BINARY_NAME 是否在 PATH 中"
        exit 1
    fi
}

# 主函数
main() {
    echo -e "${GREEN}TimeTracker 安装脚本${NC}"
    echo "================================"
    
    detect_platform
    echo -e "${BLUE}检测到平台: $OS-$ARCH${NC}"
    
    get_latest_version
    download_binary
    install_binary
    verify_installation
    
    echo ""
    echo -e "${GREEN}🎉 安装完成!${NC}"
    echo ""
    echo "接下来的步骤:"
    echo "1. 运行 'timetracker permissions' 检查和请求必要权限"
    echo "2. 运行 'timetracker start' 开始时间追踪"
    echo "3. 运行 'timetracker stats' 查看统计信息"
    echo ""
    echo "更多信息请访问: https://github.com/${REPO}"
}

# 运行主函数
main "$@"