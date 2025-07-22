# TimeTracker

一个智能的跨平台时间追踪工具，用于监控和分析应用程序使用情况。

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.70+-blue.svg)](https://www.rust-lang.org)

## ✨ 功能特性

- 🖥️ **跨平台支持**: Windows、macOS、Linux
- ⏱️ **实时监控**: 后台守护进程，自动追踪活动窗口
- 🎯 **智能活跃度检测**: 自动识别用户闲置状态，观看视频时智能记录
- 📊 **智能分析**: 内置AI分析，深度洞察使用习惯
- 📈 **可视化界面**: 美观的终端界面，实时统计展示
- 📁 **数据导出**: 支持JSON、CSV格式导出
- 🔧 **简单易用**: 直观的命令行界面

## 🚀 快速开始

### 安装

#### 🎯 一键安装（推荐）

**Linux / macOS:**
```bash
curl -fsSL https://raw.githubusercontent.com/geraldpeng6/timetracker/main/install-remote.sh | bash
```

**Windows (PowerShell):**
```powershell
iwr -useb https://raw.githubusercontent.com/geraldpeng6/timetracker/main/install.ps1 | iex
```

#### 📦 包管理器安装

**Homebrew (macOS):**
```bash
brew tap geraldpeng6/timetracker
brew install timetracker
```

**APT (Ubuntu/Debian):**
```bash
wget https://github.com/geraldpeng6/timetracker/releases/latest/download/timetracker_0.2.2_amd64.deb
sudo dpkg -i timetracker_0.2.2_amd64.deb
```

**RPM (CentOS/RHEL/Fedora):**
```bash
wget https://github.com/geraldpeng6/timetracker/releases/latest/download/timetracker-0.2.2-1.x86_64.rpm
sudo rpm -i timetracker-0.2.2-1.x86_64.rpm
```

#### 🐳 Docker 部署
```bash
docker pull ghcr.io/geraldpeng6/timetracker:latest
docker run -it --rm ghcr.io/geraldpeng6/timetracker:latest
```

#### 📥 手动下载
从 [Releases](https://github.com/geraldpeng6/timetracker/releases) 页面下载预编译二进制文件。

#### 🔧 源码编译
```bash
git clone https://github.com/geraldpeng6/timetracker.git
cd timetracker
cargo build --release
sudo cp target/release/timetracker /usr/local/bin/
```

> 📖 详细安装指南请参考: [部署文档](docs/DEPLOYMENT.md)

### 基本使用

```bash
# 启动时间追踪（交互式）
timetracker start

# 启动守护进程
timetracker start --daemon

# 查看统计信息
timetracker stats

# 管理活跃度检测
timetracker activity status    # 查看活跃度状态
timetracker activity config    # 查看检测配置
timetracker activity test      # 测试检测功能

# 停止守护进程
timetracker stop

# 导出数据
timetracker export -o data.csv -f csv

# AI分析使用情况
timetracker analyze
```

## 📋 命令概览

| 命令 | 描述 |
|------|------|
| `start` | 开始时间追踪 |
| `stop` | 停止守护进程 |
| `status` | 查看运行状态 |
| `stats` | 显示使用统计 |
| `export` | 导出数据 |
| `analyze` | AI分析 |
| `permissions` | 检查权限 |
| `activity` | 管理活跃度检测 |

## 🎯 活跃度检测功能

TimeTracker 内置智能活跃度检测功能，可以：

### ✨ 核心特性
- **智能闲置检测**: 自动识别用户是否处于活跃状态
- **视频播放识别**: 观看视频时即使闲置也会继续记录
- **节省存储空间**: 闲置时不记录无意义的窗口活动
- **跨平台支持**: 支持 macOS、Windows、Linux 的系统级闲置检测

### 🎮 支持的视频场景
- **视频应用**: VLC, QuickTime Player, IINA, Netflix, YouTube 等
- **视频网站**: YouTube, Bilibili, Netflix, 爱奇艺, 腾讯视频等
- **关键词识别**: 自动识别窗口标题中的"播放"、"视频"、"直播"等关键词

### 📊 活跃状态类型
| 状态 | 图标 | 说明 | 是否记录 |
|------|------|------|----------|
| 活跃 | 🟢 | 用户正在使用计算机 | ✅ |
| 闲置 | 🟡 | 用户已闲置超过设定时间 | ❌ |
| 观看视频 | 📺 | 正在观看视频内容 | ✅ |
| 未知 | ❓ | 无法确定状态 | ❌ |

### 🔧 活跃度检测命令
```bash
timetracker activity status   # 查看当前活跃度状态
timetracker activity config   # 查看检测配置
timetracker activity test     # 测试检测功能
timetracker activity enable   # 启用活跃度检测
timetracker activity disable  # 禁用活跃度检测
```

> 📖 详细文档请参考: [活跃度检测功能说明](docs/ACTIVITY_DETECTION.md)

## 🔧 配置

### 权限设置

**macOS**: 需要授权辅助功能和屏幕录制权限
```bash
timetracker permissions
```

**Linux**: 需要安装 `xdotool` (X11)
```bash
sudo apt install xdotool  # Ubuntu/Debian
```

### AI 配置

```bash
# 配置AI服务
timetracker ai config

# 使用本地分析
timetracker analyze --local
```

## 📊 数据格式

数据以JSON格式存储，包含应用名称、窗口标题、使用时间等信息：

```json
{
  "activities": [
    {
      "app_name": "Visual Studio Code",
      "window_title": "main.rs - timetracker",
      "start_time": "2024-01-15T10:30:00Z",
      "duration_seconds": 1800
    }
  ]
}
```

## 🏗️ 项目结构

```
timetracker/
├── src/                   # 源代码
├── tests/                 # 测试文件
├── examples/              # 使用示例
├── docs/                  # 文档
├── docker/                # Docker配置
└── scripts/               # 构建脚本
```

## 🛠️ 开发工作流程

### 快速开始开发

```bash
# 克隆项目
git clone https://github.com/geraldpeng6/timetracker.git
cd timetracker

# 安装依赖并编译
cargo build

# 运行完整测试套件
./scripts/test.sh
# 或使用 Makefile
make all
```

### 代码质量检查

在每次提交前，请运行以下检查：

```bash
# 方式1: 使用测试脚本（推荐）
./scripts/test.sh

# 方式2: 使用 Makefile
make check          # 快速检查（格式 + Clippy）
make test           # 运行测试
make all            # 完整流程

# 方式3: 手动执行
cargo fmt --all -- --check    # 代码格式检查
cargo clippy --all-targets --all-features -- -D warnings  # 静态分析
cargo test --verbose          # 单元测试
cargo build --release         # 编译检查
```

### 自动化检查（推荐）

安装 Git pre-commit hooks，每次提交时自动进行代码检查：

```bash
# 安装 pre-commit hooks
make setup-hooks
# 或直接运行
./scripts/install-hooks.sh

# 现在每次 git commit 都会自动检查代码质量
# 如需跳过检查：git commit --no-verify
```

### 可用的 Make 命令

| 命令 | 描述 |
|------|------|
| `make help` | 显示帮助信息 |
| `make check` | 快速检查（格式 + Clippy） |
| `make test` | 运行测试 |
| `make build` | 编译项目 |
| `make release` | 编译 Release 版本 |
| `make dev` | 开发模式（格式化 + 检查 + 测试） |
| `make all` | 完整流程（检查 + 测试 + 构建） |
| `make clean` | 清理构建文件 |

### 发布流程

```bash
# 发布补丁版本
./scripts/release.sh patch

# 发布次版本
./scripts/release.sh minor

# 预览发布（不实际执行）
./scripts/release.sh --dry-run patch
```

## 📚 文档

- [快速开始](QUICKSTART.md) - 详细的入门指南
- [架构说明](docs/ARCHITECTURE.md) - 技术架构文档
- [使用示例](examples/basic_usage.md) - 基本使用示例
- [贡献指南](CONTRIBUTING.md) - 如何参与贡献

## 🤝 贡献

欢迎贡献代码！请查看 [贡献指南](CONTRIBUTING.md) 了解详情。

## 📄 许可证

本项目采用 [MIT 许可证](LICENSE)。

## 🙏 致谢

感谢所有贡献者和以下开源项目：
- [tokio](https://tokio.rs/) - 异步运行时
- [ratatui](https://ratatui.rs/) - 终端界面
- [serde](https://serde.rs/) - 序列化框架