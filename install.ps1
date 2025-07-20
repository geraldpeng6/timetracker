# TimeTracker Windows 安装脚本
# PowerShell 版本

param(
    [string]$InstallDir = "$env:LOCALAPPDATA\Programs\TimeTracker",
    [switch]$AddToPath = $true
)

# 设置错误处理
$ErrorActionPreference = "Stop"

# 项目信息
$REPO = "yourusername/timetracker"
$BINARY_NAME = "timetracker.exe"

# 颜色函数
function Write-ColorOutput($ForegroundColor) {
    $fc = $host.UI.RawUI.ForegroundColor
    $host.UI.RawUI.ForegroundColor = $ForegroundColor
    if ($args) {
        Write-Output $args
    }
    $host.UI.RawUI.ForegroundColor = $fc
}

function Write-Info($message) {
    Write-ColorOutput Blue $message
}

function Write-Success($message) {
    Write-ColorOutput Green $message
}

function Write-Warning($message) {
    Write-ColorOutput Yellow $message
}

function Write-Error($message) {
    Write-ColorOutput Red $message
}

# 检测架构
function Get-Architecture {
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

# 获取最新版本
function Get-LatestVersion {
    Write-Info "获取最新版本信息..."
    try {
        $response = Invoke-RestMethod -Uri "https://api.github.com/repos/$REPO/releases/latest"
        $version = $response.tag_name
        Write-Success "最新版本: $version"
        return $version
    }
    catch {
        Write-Error "无法获取最新版本信息: $_"
        exit 1
    }
}

# 下载二进制文件
function Download-Binary($version, $arch) {
    $assetName = "timetracker-windows-$arch.exe"
    $downloadUrl = "https://github.com/$REPO/releases/download/$version/$assetName"
    $tempFile = "$env:TEMP\$BINARY_NAME"
    
    Write-Info "下载 $assetName..."
    Write-Info "下载地址: $downloadUrl"
    
    try {
        Invoke-WebRequest -Uri $downloadUrl -OutFile $tempFile -UseBasicParsing
        Write-Success "下载完成"
        return $tempFile
    }
    catch {
        Write-Error "下载失败: $_"
        exit 1
    }
}

# 安装二进制文件
function Install-Binary($tempFile) {
    Write-Info "安装到 $InstallDir..."
    
    # 创建安装目录
    if (!(Test-Path $InstallDir)) {
        New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
    }
    
    $targetFile = Join-Path $InstallDir $BINARY_NAME
    
    # 如果文件已存在，先停止可能运行的进程
    if (Test-Path $targetFile) {
        Write-Info "停止现有的 TimeTracker 进程..."
        Get-Process -Name "timetracker" -ErrorAction SilentlyContinue | Stop-Process -Force
        Start-Sleep -Seconds 2
    }
    
    # 复制文件
    Copy-Item $tempFile $targetFile -Force
    Remove-Item $tempFile -Force
    
    Write-Success "安装完成!"
    return $targetFile
}

# 添加到 PATH
function Add-ToPath($installDir) {
    if (!$AddToPath) {
        return
    }
    
    Write-Info "添加到 PATH..."
    
    # 获取当前用户的 PATH
    $currentPath = [Environment]::GetEnvironmentVariable("PATH", "User")
    
    # 检查是否已经在 PATH 中
    if ($currentPath -split ";" -contains $installDir) {
        Write-Info "已在 PATH 中"
        return
    }
    
    # 添加到 PATH
    $newPath = if ($currentPath) { "$currentPath;$installDir" } else { $installDir }
    [Environment]::SetEnvironmentVariable("PATH", $newPath, "User")
    
    # 更新当前会话的 PATH
    $env:PATH = "$env:PATH;$installDir"
    
    Write-Success "已添加到 PATH"
}

# 验证安装
function Test-Installation {
    Write-Info "验证安装..."
    
    try {
        $version = & timetracker --version 2>$null
        Write-Success "✓ TimeTracker 安装成功!"
        Write-Output "版本: $version"
        Write-Output ""
        Write-Output "使用方法:"
        Write-Output "  timetracker --help          # 查看帮助"
        Write-Output "  timetracker permissions     # 检查权限"
        Write-Output "  timetracker start           # 开始追踪"
        Write-Output "  timetracker stats           # 查看统计"
    }
    catch {
        Write-Warning "✗ 安装验证失败"
        Write-Warning "请重新启动 PowerShell 或命令提示符"
        Write-Warning "或者直接运行: $InstallDir\$BINARY_NAME"
    }
}

# 主函数
function Main {
    Write-Success "TimeTracker Windows 安装脚本"
    Write-Output "================================"
    
    $arch = Get-Architecture
    Write-Info "检测到架构: $arch"
    
    $version = Get-LatestVersion
    $tempFile = Download-Binary $version $arch
    $targetFile = Install-Binary $tempFile
    Add-ToPath $InstallDir
    Test-Installation
    
    Write-Output ""
    Write-Success "🎉 安装完成!"
    Write-Output ""
    Write-Output "接下来的步骤:"
    Write-Output "1. 重新启动 PowerShell 或命令提示符"
    Write-Output "2. 运行 'timetracker permissions' 检查和请求必要权限"
    Write-Output "3. 运行 'timetracker start' 开始时间追踪"
    Write-Output "4. 运行 'timetracker stats' 查看统计信息"
    Write-Output ""
    Write-Output "更多信息请访问: https://github.com/$REPO"
}

# 运行主函数
Main