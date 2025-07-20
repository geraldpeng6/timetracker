# TimeTracker 架构说明

## 项目结构

```
timetracker/
├── src/                    # 源代码目录
│   ├── main.rs            # 主程序入口
│   ├── tracker.rs         # 时间追踪核心逻辑
│   ├── platform.rs        # 平台相关功能（窗口监控）
│   ├── daemon.rs          # 守护进程管理
│   ├── permissions.rs     # 权限检查和请求
│   ├── tui.rs            # 终端用户界面
│   ├── ai_analysis.rs    # AI分析功能
│   ├── ai_client.rs      # AI客户端
│   ├── ai_config.rs      # AI配置
│   └── ai_config_manager.rs # AI配置管理
├── tests/                 # 测试文件
├── examples/              # 使用示例
├── docs/                  # 文档
├── docker/                # Docker相关文件
├── scripts/               # 构建和发布脚本
├── .github/               # GitHub相关配置
└── Formula/               # Homebrew公式
```

## 核心模块

### 1. 时间追踪器 (tracker.rs)
- `TimeTracker` 结构体：核心时间追踪逻辑
- 活动记录和数据持久化
- 统计信息计算

### 2. 平台接口 (platform.rs)
- 跨平台窗口信息获取
- macOS 特定的窗口监控实现
- 活动窗口检测

### 3. 守护进程管理 (daemon.rs)
- `DaemonManager` 结构体：守护进程生命周期管理
- PID文件管理
- 进程启动、停止、重启

### 4. 权限管理 (permissions.rs)
- macOS 权限检查和请求
- 辅助功能权限
- 屏幕录制权限

### 5. 用户界面 (tui.rs)
- 基于 `ratatui` 的终端界面
- 交互式统计显示
- 实时数据可视化

### 6. AI 功能
- `ai_analysis.rs`: AI分析逻辑
- `ai_client.rs`: 统一AI客户端接口
- `ai_config.rs`: AI配置数据结构
- `ai_config_manager.rs`: AI配置管理

## 数据流

1. **启动阶段**：
   - 检查权限
   - 初始化配置
   - 启动监控循环

2. **监控阶段**：
   - 定期获取活动窗口信息
   - 记录应用使用时间
   - 保存数据到JSON文件

3. **分析阶段**：
   - 读取历史数据
   - 生成统计报告
   - 可选AI分析

## 配置文件

- 数据文件：JSON格式，存储时间追踪记录
- AI配置：存储API密钥和模型配置
- PID文件：`/tmp/timetracker.pid`
- 日志文件：`/tmp/timetracker.log`

## 依赖关系

主要外部依赖：
- `clap`: 命令行参数解析
- `serde`: 序列化/反序列化
- `tokio`: 异步运行时
- `ratatui`: 终端用户界面
- `chrono`: 时间处理
- `reqwest`: HTTP客户端（AI功能）