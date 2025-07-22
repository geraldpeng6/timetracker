# TimeTracker Windows PowerShell 安装脚本
# 从 GitHub Releases 下载预编译二进制文件

param(
    [string]$Version = "0.2.2",
    [switch]$Help,
    [switch]$Force
)

# 设置错误处理
$ErrorActionPreference = "Stop"

# 常量定义
$REPO = "geraldpeng6/timetracker"
$BASE_URL = "https://github.com/$REPO/releases/download/v$Version"

# 颜色函数
function Write-ColorOutput {
    param(
        [string]$Message,
        [string]$Color = "White"
    )
    Write-Host $Message -ForegroundColor $Color
}

function Write-Info {
    param([string]$Message)
    Write-ColorOutput "[INFO] $Message" "Cyan"
}

function Write-Success {
    param([string]$Message)
    Write-ColorOutput "[SUCCESS] $Message" "Green"
}

function Write-Warning {
    param([string]$Message)
    Write-ColorOutput "[WARNING] $Message" "Yellow"
}

function Write-Error {
    param([string]$Message)
    Write-ColorOutput "[ERROR] $Message" "Red"
}

# 显示帮助信息
function Show-Help {
    Write-Host "TimeTracker Windows 安装脚本"
    Write-Host ""
    Write-Host "用法: .\install.ps1 [选项]"
    Write-Host ""
    Write-Host "选项:"
    Write-Host "  -Version <版本>    指定要安装的版本 (默认: $Version)"
    Write-Host "  -Force            强制覆盖现有安装"
    Write-Host "  -Help             显示此帮助信息"
    Write-Host ""
    Write-Host "示例:"
    Write-Host "  .\install.ps1                    # 安装默认版本"
    Write-Host "  .\install.ps1 -Version 0.2.1     # 安装指定版本"
    Write-Host "  .\install.ps1 -Force             # 强制重新安装"
    Write-Host ""
    Write-Host "一键安装命令:"
    Write-Host "  iwr -useb https://raw.githubusercontent.com/$REPO/main/install.ps1 | iex"
}

# 检测系统架构
function Get-SystemArchitecture {
    $arch = $env:PROCESSOR_ARCHITECTURE
    switch ($arch) {
        "AMD64" { return "x86_64" }
        "ARM64" { return "aarch64" }
        default { 
            Write-Error "不支持的架构: $arch"
            exit 1
        }
    }
}

# 检查依赖
function Test-Dependencies {
    Write-Info "检查系统依赖..."
    
    # 检查 PowerShell 版本
    if ($PSVersionTable.PSVersion.Major -lt 5) {
        Write-Error "需要 PowerShell 5.0 或更高版本"
        exit 1
    }
    
    # 检查网络连接
    try {
        $null = Invoke-WebRequest -Uri "https://github.com" -UseBasicParsing -TimeoutSec 10
        Write-Info "网络连接正常"
    }
    catch {
        Write-Error "无法连接到 GitHub，请检查网络连接"
        exit 1
    }
}

# 下载并安装
function Install-TimeTracker {
    $arch = Get-SystemArchitecture
    $filename = "timetracker-windows-$arch.exe.zip"
    $url = "$BASE_URL/$filename"
    
    Write-Info "下载 TimeTracker v$Version..."
    Write-Info "URL: $url"
    
    # 创建临时目录
    $tempDir = New-TemporaryFile | ForEach-Object { Remove-Item $_; New-Item -ItemType Directory -Path $_ }
    $tempFile = Join-Path $tempDir $filename
    
    try {
        # 下载文件
        Write-Info "正在下载..."
        Invoke-WebRequest -Uri $url -OutFile $tempFile -UseBasicParsing
        Write-Success "下载完成"
        
        # 解压文件
        Write-Info "解压文件..."
        Expand-Archive -Path $tempFile -DestinationPath $tempDir -Force
        
        # 确定安装目录
        $installDir = "$env:USERPROFILE\bin"
        if (-not (Test-Path $installDir)) {
            New-Item -ItemType Directory -Path $installDir -Force | Out-Null
        }
        
        $binaryPath = Join-Path $installDir "timetracker.exe"
        $sourceBinary = Join-Path $tempDir "timetracker.exe"
        
        # 检查现有安装
        if (Test-Path $binaryPath) {
            if (-not $Force) {
                Write-Warning "TimeTracker 已安装在 $binaryPath"
                $response = Read-Host "是否覆盖现有安装? (y/N)"
                if ($response -notmatch '^[Yy]$') {
                    Write-Info "安装已取消"
                    return
                }
            }
            
            # 备份现有版本
            $backupPath = "$binaryPath.backup.$(Get-Date -Format 'yyyyMMdd_HHmmss')"
            Write-Info "备份现有版本到 $backupPath"
            Copy-Item $binaryPath $backupPath
        }
        
        # 安装二进制文件
        Write-Info "安装到 $installDir..."
        Copy-Item $sourceBinary $binaryPath -Force
        
        Write-Success "TimeTracker 安装完成!"
        
        # 检查 PATH
        $userPath = [Environment]::GetEnvironmentVariable("PATH", "User")
        if ($userPath -notlike "*$installDir*") {
            Write-Warning "$installDir 不在 PATH 中"
            $response = Read-Host "是否将 $installDir 添加到 PATH? (Y/n)"
            if ($response -notmatch '^[Nn]$') {
                $newPath = "$userPath;$installDir"
                [Environment]::SetEnvironmentVariable("PATH", $newPath, "User")
                Write-Success "已将 $installDir 添加到 PATH"
                Write-Info "请重新启动 PowerShell 或命令提示符以使更改生效"
            }
        }
    }
    catch {
        Write-Error "安装失败: $($_.Exception.Message)"
        exit 1
    }
    finally {
        # 清理临时文件
        if (Test-Path $tempDir) {
            Remove-Item $tempDir -Recurse -Force
        }
    }
}

# 验证安装
function Test-Installation {
    Write-Info "验证安装..."
    
    try {
        $version = & timetracker --version 2>$null
        Write-Success "TimeTracker 安装成功!"
        Write-Info "版本: $version"
        return $true
    }
    catch {
        Write-Error "安装验证失败"
        Write-Info "请检查 PATH 设置或重新启动 PowerShell"
        return $false
    }
}

# 显示使用说明
function Show-Usage {
    Write-Host ""
    Write-Success "🎉 安装完成!"
    Write-Host ""
    Write-Info "快速开始:"
    Write-Host "  timetracker --help              # 查看帮助"
    Write-Host "  timetracker start               # 开始时间追踪"
    Write-Host "  timetracker tui                 # 打开交互界面"
    Write-Host "  timetracker activity status     # 查看活跃度状态"
    Write-Host "  timetracker activity config     # 查看活跃度配置"
    Write-Host ""
    Write-Info "Windows 特定说明:"
    Write-Host "  TimeTracker 在 Windows 上可能需要管理员权限来监控某些应用"
    Write-Host "  建议在 PowerShell 或命令提示符中运行"
    Write-Host ""
    Write-Info "更多信息:"
    Write-Host "  GitHub: https://github.com/$REPO"
    Write-Host "  文档: https://github.com/$REPO#readme"
    Write-Host "  活跃度检测: https://github.com/$REPO/blob/main/docs/ACTIVITY_DETECTION.md"
    Write-Host ""
}

# 主函数
function Main {
    if ($Help) {
        Show-Help
        return
    }
    
    Write-Host "🎯 TimeTracker Windows 安装脚本" -ForegroundColor Magenta
    Write-Host "================================" -ForegroundColor Magenta
    Write-Host ""
    
    # 检查管理员权限
    $isAdmin = ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole] "Administrator")
    if ($isAdmin) {
        Write-Warning "正在以管理员权限运行"
    }
    
    Test-Dependencies
    Install-TimeTracker
    
    if (Test-Installation) {
        Show-Usage
    }
    else {
        Write-Error "安装过程中出现问题"
        Write-Info "请尝试手动下载并安装: https://github.com/$REPO/releases"
        exit 1
    }
}

# 执行主函数
Main
