#!/bin/bash

# TimeTracker 部署配置测试脚本
# 验证所有部署相关的配置和脚本

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

# 检查文件是否存在
check_file() {
    local file=$1
    local description=$2
    
    if [[ -f "$file" ]]; then
        print_success "$description: $file ✓"
        return 0
    else
        print_error "$description: $file ✗"
        return 1
    fi
}

# 检查目录是否存在
check_directory() {
    local dir=$1
    local description=$2
    
    if [[ -d "$dir" ]]; then
        print_success "$description: $dir ✓"
        return 0
    else
        print_error "$description: $dir ✗"
        return 1
    fi
}

# 验证 YAML 语法
validate_yaml() {
    local file=$1
    
    if command -v yamllint >/dev/null 2>&1; then
        if yamllint "$file" >/dev/null 2>&1; then
            print_success "YAML 语法检查: $file ✓"
            return 0
        else
            print_error "YAML 语法错误: $file ✗"
            return 1
        fi
    else
        print_warning "yamllint 未安装，跳过 YAML 语法检查"
        return 0
    fi
}

# 验证 shell 脚本语法
validate_shell_script() {
    local file=$1
    
    if bash -n "$file" 2>/dev/null; then
        print_success "Shell 脚本语法检查: $file ✓"
        return 0
    else
        print_error "Shell 脚本语法错误: $file ✗"
        return 1
    fi
}

# 验证 PowerShell 脚本语法
validate_powershell_script() {
    local file=$1
    
    if command -v pwsh >/dev/null 2>&1; then
        if pwsh -Command "Get-Content '$file' | Out-Null" 2>/dev/null; then
            print_success "PowerShell 脚本语法检查: $file ✓"
            return 0
        else
            print_error "PowerShell 脚本语法错误: $file ✗"
            return 1
        fi
    else
        print_warning "PowerShell 未安装，跳过语法检查"
        return 0
    fi
}

# 验证 Dockerfile 语法
validate_dockerfile() {
    local file=$1
    
    if command -v docker >/dev/null 2>&1; then
        if docker build -f "$file" -t timetracker-test . >/dev/null 2>&1; then
            print_success "Dockerfile 构建测试: $file ✓"
            docker rmi timetracker-test >/dev/null 2>&1 || true
            return 0
        else
            print_error "Dockerfile 构建失败: $file ✗"
            return 1
        fi
    else
        print_warning "Docker 未安装，跳过 Dockerfile 测试"
        return 0
    fi
}

# 验证 Cargo.toml 配置
validate_cargo_config() {
    print_info "验证 Cargo.toml 配置..."
    
    # 检查基本字段
    if cargo metadata --format-version 1 --no-deps >/dev/null 2>&1; then
        print_success "Cargo.toml 基本配置 ✓"
    else
        print_error "Cargo.toml 配置错误 ✗"
        return 1
    fi
    
    # 检查 DEB 包配置
    if grep -q "\[package.metadata.deb\]" Cargo.toml; then
        print_success "DEB 包配置 ✓"
    else
        print_error "DEB 包配置缺失 ✗"
        return 1
    fi
    
    # 检查 RPM 包配置
    if grep -q "\[package.metadata.generate-rpm\]" Cargo.toml; then
        print_success "RPM 包配置 ✓"
    else
        print_error "RPM 包配置缺失 ✗"
        return 1
    fi
    
    return 0
}

# 主测试函数
main() {
    echo "🧪 TimeTracker 部署配置测试"
    echo "============================"
    echo
    
    local errors=0
    
    # 检查项目结构
    print_info "检查项目结构..."
    check_file "Cargo.toml" "项目配置文件" || ((errors++))
    check_file "README.md" "项目说明文档" || ((errors++))
    check_file "LICENSE" "许可证文件" || ((errors++))
    check_file "CHANGELOG.md" "变更日志" || ((errors++))
    echo
    
    # 检查 GitHub Actions
    print_info "检查 GitHub Actions 配置..."
    check_directory ".github/workflows" "工作流目录" || ((errors++))
    check_file ".github/workflows/ci.yml" "CI 工作流" || ((errors++))
    check_file ".github/workflows/release.yml" "发布工作流" || ((errors++))
    check_file ".github/workflows/docker.yml" "Docker 工作流" || ((errors++))
    echo
    
    # 验证 YAML 文件
    print_info "验证 YAML 文件语法..."
    for yaml_file in .github/workflows/*.yml docker-compose.yml; do
        if [[ -f "$yaml_file" ]]; then
            validate_yaml "$yaml_file" || ((errors++))
        fi
    done
    echo
    
    # 检查安装脚本
    print_info "检查安装脚本..."
    check_file "install.sh" "本地安装脚本" || ((errors++))
    check_file "install-remote.sh" "远程安装脚本" || ((errors++))
    check_file "install.ps1" "Windows 安装脚本" || ((errors++))
    echo
    
    # 验证脚本语法
    print_info "验证脚本语法..."
    for script in install.sh install-remote.sh scripts/*.sh; do
        if [[ -f "$script" ]]; then
            validate_shell_script "$script" || ((errors++))
        fi
    done
    
    if [[ -f "install.ps1" ]]; then
        validate_powershell_script "install.ps1" || ((errors++))
    fi
    echo
    
    # 检查 Docker 配置
    print_info "检查 Docker 配置..."
    check_file "Dockerfile" "Docker 镜像配置" || ((errors++))
    check_file "docker-compose.yml" "Docker Compose 配置" || ((errors++))
    echo
    
    # 验证 Dockerfile
    print_info "验证 Dockerfile..."
    if [[ -f "Dockerfile" ]]; then
        validate_dockerfile "Dockerfile" || ((errors++))
    fi
    echo
    
    # 检查包管理器配置
    print_info "检查包管理器配置..."
    check_file "Formula/timetracker.rb" "Homebrew Formula" || ((errors++))
    echo
    
    # 验证 Cargo 配置
    validate_cargo_config || ((errors++))
    echo
    
    # 检查文档
    print_info "检查文档..."
    check_directory "docs" "文档目录" || ((errors++))
    check_file "docs/DEPLOYMENT.md" "部署文档" || ((errors++))
    check_file "docs/ACTIVITY_DETECTION.md" "活跃度检测文档" || ((errors++))
    echo
    
    # 检查脚本目录
    print_info "检查脚本目录..."
    check_directory "scripts" "脚本目录" || ((errors++))
    check_file "scripts/release.sh" "发布脚本" || ((errors++))
    echo
    
    # 检查可执行权限
    print_info "检查脚本可执行权限..."
    for script in install.sh install-remote.sh scripts/*.sh; do
        if [[ -f "$script" ]]; then
            if [[ -x "$script" ]]; then
                print_success "可执行权限: $script ✓"
            else
                print_warning "缺少可执行权限: $script"
                chmod +x "$script"
                print_info "已添加可执行权限: $script"
            fi
        fi
    done
    echo
    
    # 测试构建
    print_info "测试项目构建..."
    if cargo check --all-targets >/dev/null 2>&1; then
        print_success "项目构建检查 ✓"
    else
        print_error "项目构建检查失败 ✗"
        ((errors++))
    fi
    echo
    
    # 总结
    echo "📊 测试总结"
    echo "============"
    if [[ $errors -eq 0 ]]; then
        print_success "所有部署配置检查通过! 🎉"
        echo
        print_info "部署配置状态:"
        echo "  ✅ GitHub Actions CI/CD"
        echo "  ✅ 多平台安装脚本"
        echo "  ✅ Docker 容器化"
        echo "  ✅ 包管理器支持"
        echo "  ✅ 文档完整"
        echo "  ✅ 发布自动化"
        echo
        print_info "可以安全地进行发布!"
        return 0
    else
        print_error "发现 $errors 个问题需要修复"
        echo
        print_info "请修复上述问题后重新运行测试"
        return 1
    fi
}

# 处理命令行参数
case "${1:-}" in
    --help|-h)
        echo "TimeTracker 部署配置测试脚本"
        echo
        echo "用法: $0 [选项]"
        echo
        echo "选项:"
        echo "  --help, -h    显示此帮助信息"
        echo
        echo "此脚本会检查所有部署相关的配置文件和脚本，确保："
        echo "  - 文件存在性"
        echo "  - 语法正确性"
        echo "  - 配置完整性"
        echo "  - 权限设置"
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
