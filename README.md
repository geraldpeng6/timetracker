# TimeTracker

一个智能的跨平台时间追踪工具，用于监控和分析应用程序使用情况。

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.70+-blue.svg)](https://www.rust-lang.org)

## ✨ 功能特性

- 🖥️ **跨平台支持**: Windows、macOS、Linux
- ⏱️ **实时监控**: 后台守护进程，自动追踪活动窗口
- 📊 **智能分析**: 内置AI分析，深度洞察使用习惯
- 📈 **可视化界面**: 美观的终端界面，实时统计展示
- 📁 **数据导出**: 支持JSON、CSV格式导出
- 🔧 **简单易用**: 直观的命令行界面

## 🚀 快速开始

### 安装

```bash
# 一键安装脚本
curl -fsSL https://raw.githubusercontent.com/geraldpeng6/timetracker/main/install.sh | bash

# 或使用 Cargo
cargo install timetracker

# 或从源码编译
git clone https://github.com/geraldpeng6/timetracker.git
cd timetracker
cargo build --release
```

### 基本使用

```bash
# 启动时间追踪（交互式）
timetracker start

# 启动守护进程
timetracker start --daemon

# 查看统计信息
timetracker stats

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