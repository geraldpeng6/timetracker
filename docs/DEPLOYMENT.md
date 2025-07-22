# TimeTracker 部署指南

本文档详细介绍了 TimeTracker 的各种部署方式，包括预编译二进制文件、包管理器、Docker 容器等。

## 📦 安装方式概览

| 方式 | 平台 | 难度 | 推荐度 |
|------|------|------|--------|
| 一键安装脚本 | Linux, macOS | ⭐ | ⭐⭐⭐⭐⭐ |
| PowerShell 脚本 | Windows | ⭐ | ⭐⭐⭐⭐⭐ |
| Homebrew | macOS | ⭐ | ⭐⭐⭐⭐ |
| APT/DEB | Ubuntu/Debian | ⭐ | ⭐⭐⭐⭐ |
| RPM | CentOS/RHEL/Fedora | ⭐ | ⭐⭐⭐⭐ |
| Docker | 所有平台 | ⭐⭐ | ⭐⭐⭐ |
| 手动下载 | 所有平台 | ⭐⭐ | ⭐⭐⭐ |
| 源码编译 | 所有平台 | ⭐⭐⭐ | ⭐⭐ |

## 🚀 一键安装（推荐）

### Linux / macOS
```bash
# 使用 curl
curl -fsSL https://raw.githubusercontent.com/geraldpeng6/timetracker/main/install-remote.sh | bash

# 或使用 wget
wget -qO- https://raw.githubusercontent.com/geraldpeng6/timetracker/main/install-remote.sh | bash

# 安装指定版本
VERSION=0.2.1 curl -fsSL https://raw.githubusercontent.com/geraldpeng6/timetracker/main/install-remote.sh | bash
```

### Windows (PowerShell)
```powershell
# 管理员权限运行 PowerShell
iwr -useb https://raw.githubusercontent.com/geraldpeng6/timetracker/main/install.ps1 | iex

# 或下载后运行
Invoke-WebRequest -Uri "https://raw.githubusercontent.com/geraldpeng6/timetracker/main/install.ps1" -OutFile "install.ps1"
.\install.ps1

# 安装指定版本
.\install.ps1 -Version 0.2.1
```

## 📱 包管理器安装

### Homebrew (macOS)
```bash
# 添加 tap（如果已发布到 Homebrew）
brew tap geraldpeng6/timetracker
brew install timetracker

# 或直接从 formula 安装
brew install https://raw.githubusercontent.com/geraldpeng6/timetracker/main/Formula/timetracker.rb
```

### APT (Ubuntu/Debian)
```bash
# 下载 DEB 包
wget https://github.com/geraldpeng6/timetracker/releases/latest/download/timetracker_0.2.2_amd64.deb

# 安装
sudo dpkg -i timetracker_0.2.2_amd64.deb

# 修复依赖（如果需要）
sudo apt-get install -f
```

### RPM (CentOS/RHEL/Fedora)
```bash
# 下载 RPM 包
wget https://github.com/geraldpeng6/timetracker/releases/latest/download/timetracker-0.2.2-1.x86_64.rpm

# 安装
sudo rpm -i timetracker-0.2.2-1.x86_64.rpm

# 或使用 dnf/yum
sudo dnf install timetracker-0.2.2-1.x86_64.rpm
```

## 🐳 Docker 部署

### 基本使用
```bash
# 拉取镜像
docker pull ghcr.io/geraldpeng6/timetracker:latest

# 运行容器
docker run -it --rm \
  -v $(pwd)/data:/data \
  -e DISPLAY=$DISPLAY \
  -v /tmp/.X11-unix:/tmp/.X11-unix:rw \
  ghcr.io/geraldpeng6/timetracker:latest

# 后台运行
docker run -d \
  --name timetracker \
  -v timetracker_data:/data \
  --restart unless-stopped \
  ghcr.io/geraldpeng6/timetracker:latest timetracker start --daemon
```

### Docker Compose
```bash
# 克隆仓库
git clone https://github.com/geraldpeng6/timetracker.git
cd timetracker

# 启动服务
docker-compose up -d

# 查看日志
docker-compose logs -f

# 停止服务
docker-compose down
```

### 自定义 Docker 镜像
```dockerfile
FROM ghcr.io/geraldpeng6/timetracker:latest

# 复制自定义配置
COPY config.toml /home/timetracker/.timetracker/

# 设置环境变量
ENV RUST_LOG=debug
ENV TIMETRACKER_IDLE_TIMEOUT=600

# 自定义启动命令
CMD ["timetracker", "start", "--config", "/home/timetracker/.timetracker/config.toml"]
```

## 📥 手动下载安装

### 1. 下载预编译二进制文件
访问 [Releases 页面](https://github.com/geraldpeng6/timetracker/releases) 下载对应平台的文件：

- **Linux x86_64**: `timetracker-linux-x86_64.tar.gz`
- **Linux ARM64**: `timetracker-linux-aarch64.tar.gz`
- **macOS x86_64**: `timetracker-macos-x86_64.tar.gz`
- **macOS ARM64**: `timetracker-macos-aarch64.tar.gz`
- **Windows x86_64**: `timetracker-windows-x86_64.exe.zip`
- **Windows ARM64**: `timetracker-windows-aarch64.exe.zip`

### 2. 解压并安装
```bash
# Linux/macOS
tar -xzf timetracker-*.tar.gz
sudo mv timetracker /usr/local/bin/
chmod +x /usr/local/bin/timetracker

# Windows
# 解压 ZIP 文件，将 timetracker.exe 移动到 PATH 中的目录
```

## 🔧 源码编译安装

### 前置要求
- Rust 1.70+
- Git
- 系统依赖（见下文）

### 系统依赖

#### Ubuntu/Debian
```bash
sudo apt-get update
sudo apt-get install -y \
  build-essential \
  pkg-config \
  libx11-dev \
  libxcb1-dev \
  libxcb-randr0-dev \
  libxcb-xtest0-dev \
  libxcb-xinerama0-dev \
  libxcb-shape0-dev \
  libxcb-xkb-dev
```

#### CentOS/RHEL/Fedora
```bash
sudo dnf install -y \
  gcc \
  pkg-config \
  libX11-devel \
  libxcb-devel \
  libxcb-randr-devel \
  libxcb-xtest-devel \
  libxcb-xinerama-devel \
  libxcb-shape-devel \
  libxcb-xkb-devel
```

#### macOS
```bash
# 安装 Xcode Command Line Tools
xcode-select --install

# 或安装 Xcode
```

#### Windows
```bash
# 安装 Visual Studio Build Tools 或 Visual Studio
# 包含 MSVC 编译器和 Windows SDK
```

### 编译步骤
```bash
# 1. 克隆仓库
git clone https://github.com/geraldpeng6/timetracker.git
cd timetracker

# 2. 编译 release 版本
cargo build --release

# 3. 安装到系统
sudo cp target/release/timetracker /usr/local/bin/

# 4. 验证安装
timetracker --version
```

## 🔐 权限配置

### macOS
TimeTracker 需要辅助功能权限来监控窗口活动：

1. 打开 **系统偏好设置** > **安全性与隐私** > **隐私**
2. 选择左侧的 **辅助功能**
3. 点击锁图标并输入密码
4. 添加终端应用或 TimeTracker 到列表中

```bash
# 检查权限状态
timetracker permissions check

# 请求权限（会打开系统设置）
timetracker permissions request
```

### Linux
某些发行版可能需要额外配置：

```bash
# 确保用户在正确的组中
sudo usermod -a -G input $USER

# 重新登录或重启
```

### Windows
Windows 用户可能需要以管理员权限运行以监控某些系统应用。

## 🚀 服务部署

### systemd 服务 (Linux)
```bash
# 创建服务文件
sudo tee /etc/systemd/system/timetracker.service > /dev/null <<EOF
[Unit]
Description=TimeTracker Service
After=network.target

[Service]
Type=simple
User=$USER
ExecStart=/usr/local/bin/timetracker start --daemon
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
EOF

# 启用并启动服务
sudo systemctl enable timetracker
sudo systemctl start timetracker

# 查看状态
sudo systemctl status timetracker
```

### launchd 服务 (macOS)
```bash
# 创建 plist 文件
mkdir -p ~/Library/LaunchAgents
tee ~/Library/LaunchAgents/com.timetracker.plist > /dev/null <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.timetracker</string>
    <key>ProgramArguments</key>
    <array>
        <string>/usr/local/bin/timetracker</string>
        <string>start</string>
        <string>--daemon</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
</dict>
</plist>
EOF

# 加载服务
launchctl load ~/Library/LaunchAgents/com.timetracker.plist

# 启动服务
launchctl start com.timetracker
```

### Windows 服务
```powershell
# 使用 NSSM (Non-Sucking Service Manager)
# 1. 下载 NSSM: https://nssm.cc/download
# 2. 安装服务
nssm install TimeTracker "C:\Program Files\TimeTracker\timetracker.exe"
nssm set TimeTracker Arguments "start --daemon"
nssm set TimeTracker DisplayName "TimeTracker Service"
nssm set TimeTracker Description "TimeTracker window monitoring service"

# 启动服务
nssm start TimeTracker
```

## 🔄 更新和卸载

### 更新
```bash
# 使用安装脚本更新到最新版本
curl -fsSL https://raw.githubusercontent.com/geraldpeng6/timetracker/main/install-remote.sh | bash

# 或手动下载新版本替换现有文件
```

### 卸载
```bash
# 停止服务
timetracker stop

# 删除二进制文件
sudo rm /usr/local/bin/timetracker

# 删除配置和数据（可选）
rm -rf ~/.timetracker

# 删除系统服务（如果安装了）
sudo systemctl stop timetracker
sudo systemctl disable timetracker
sudo rm /etc/systemd/system/timetracker.service
```

## 🐛 故障排除

### 常见问题

#### 1. 权限错误
```bash
# macOS: 检查辅助功能权限
timetracker permissions check

# Linux: 检查用户组
groups $USER
```

#### 2. 依赖缺失
```bash
# Ubuntu/Debian
sudo apt-get install libx11-6 libxcb1

# CentOS/RHEL
sudo dnf install libX11 libxcb
```

#### 3. 网络问题
```bash
# 检查网络连接
curl -I https://github.com

# 使用代理
export https_proxy=http://proxy:port
```

#### 4. Docker 问题
```bash
# 检查 X11 转发
echo $DISPLAY
xhost +local:docker

# 检查容器日志
docker logs timetracker
```

### 获取帮助
- GitHub Issues: https://github.com/geraldpeng6/timetracker/issues
- 文档: https://github.com/geraldpeng6/timetracker#readme
- 活跃度检测: https://github.com/geraldpeng6/timetracker/blob/main/docs/ACTIVITY_DETECTION.md
