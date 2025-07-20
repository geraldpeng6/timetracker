# TimeTracker 基本使用示例

## 安装

```bash
# 使用安装脚本
curl -sSL https://raw.githubusercontent.com/your-repo/timetracker/main/install.sh | bash

# 或者手动编译
git clone https://github.com/your-repo/timetracker.git
cd timetracker
cargo build --release
```

## 基本命令

### 启动时间追踪

```bash
# 交互式模式
timetracker start

# 守护进程模式
timetracker start --daemon

# 自定义检查间隔（秒）
timetracker start --interval 10

# 自定义数据文件
timetracker start --data-file my_data.json
```

### 查看状态和统计

```bash
# 查看守护进程状态
timetracker status

# 查看交互式统计
timetracker stats

# 停止守护进程
timetracker stop
```

### 数据导出

```bash
# 导出为JSON格式
timetracker export -o data.json -f json

# 导出为CSV格式
timetracker export -o data.csv -f csv
```

### AI分析

```bash
# 使用本地分析
timetracker analyze --local

# 使用在线AI分析
timetracker analyze

# 配置AI服务
timetracker ai config
```

## 权限设置

在macOS上，TimeTracker需要以下权限：
- 辅助功能权限（用于监控活动窗口）
- 屏幕录制权限（用于获取窗口信息）

运行以下命令检查权限：
```bash
timetracker permissions
```