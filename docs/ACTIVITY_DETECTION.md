# 用户活跃度检测功能

TimeTracker 的用户活跃度检测功能可以智能地判断用户是否处于活跃状态，在用户闲置时自动停止记录窗口活动，同时对视频播放等特殊场景进行智能识别。

## 🎯 功能特点

### 智能闲置检测
- **自动检测用户闲置状态**：监控键盘、鼠标活动，判断用户是否处于活跃状态
- **节省存储空间**：闲置时不记录窗口活动，避免产生无意义的数据
- **可配置超时时间**：默认5分钟闲置超时，可根据需要调整

### 视频播放智能识别
- **视频应用识别**：自动识别常见的视频播放应用
- **视频网站识别**：检测浏览器中的视频网站
- **关键词识别**：通过窗口标题中的关键词识别视频内容
- **特殊处理**：观看视频时即使用户闲置也会继续记录

### 跨平台支持
- **macOS**：使用 IOHIDSystem 获取系统闲置时间
- **Windows**：使用 GetLastInputInfo API 检测用户输入
- **Linux**：支持 xprintidle 和 xssstate 工具

## 📋 活跃状态类型

| 状态 | 图标 | 描述 | 是否记录 |
|------|------|------|----------|
| 活跃 | 🟢 | 用户正在活跃使用计算机 | ✅ 是 |
| 闲置 | 🟡 | 用户已闲置超过设定时间 | ❌ 否 |
| 观看视频 | 📺 | 正在观看视频内容 | ✅ 是 |
| 未知 | ❓ | 无法确定用户状态 | ❌ 否 |

## ⚙️ 配置选项

### 基本配置
```toml
[activity]
enabled = true           # 是否启用活跃度检测
idle_timeout = 300       # 闲置超时时间（秒）
check_interval = 1000    # 检测间隔（毫秒）
```

### 视频应用列表
默认支持的视频应用：
- **媒体播放器**：VLC, QuickTime Player, IINA, PotPlayer, MPC-HC, Windows Media Player
- **流媒体应用**：Netflix, YouTube, Bilibili, 爱奇艺, 腾讯视频, 优酷

### 视频网站列表
默认支持的视频网站：
- **国际平台**：youtube.com, netflix.com, twitch.tv, vimeo.com
- **国内平台**：bilibili.com, iqiyi.com, v.qq.com, youku.com

## 🔧 命令行接口

### 查看活跃度状态
```bash
timetracker activity status
```
显示当前用户活跃度状态、检测功能状态和相关统计信息。

### 查看配置
```bash
timetracker activity config
```
显示活跃度检测的详细配置，包括支持的视频应用和网站列表。

### 测试检测功能
```bash
timetracker activity test
```
测试活跃度检测功能，显示当前窗口信息和检测结果。

### 启用/禁用检测
```bash
timetracker activity enable   # 启用活跃度检测
timetracker activity disable  # 禁用活跃度检测
```

## 🛠️ 技术实现

### 系统闲置时间检测

#### macOS 实现
```rust
// 使用 IOHIDSystem 获取系统闲置时间
ioreg -c IOHIDSystem | grep HIDIdleTime
```

#### Windows 实现
```rust
// 使用 GetLastInputInfo API
let mut last_input_info = LASTINPUTINFO { ... };
GetLastInputInfo(&mut last_input_info);
```

#### Linux 实现
```bash
# 优先使用 xprintidle
xprintidle

# 备用方案使用 xssstate
xssstate -i
```

### 视频内容识别

#### 应用名称匹配
检查当前活跃应用是否在预定义的视频应用列表中。

#### 窗口标题分析
- **网站域名检测**：检查浏览器窗口标题中是否包含视频网站域名
- **关键词识别**：识别"播放"、"视频"、"电影"、"直播"等关键词
- **多语言支持**：支持中英文关键词识别

## 📊 使用示例

### 基本使用
```bash
# 启动带活跃度检测的监控
timetracker start

# 查看当前状态
timetracker activity status
```

### 自定义配置
```toml
# ~/.timetracker/config.toml
[activity]
enabled = true
idle_timeout = 600        # 10分钟闲置超时
check_interval = 2000     # 2秒检测间隔

# 添加自定义视频应用
video_apps = [
    "VLC",
    "Custom Video Player",
    "My Media App"
]

# 添加自定义视频网站
video_sites = [
    "youtube.com",
    "myvideo.com",
    "customstream.tv"
]
```

## 🔍 故障排除

### 常见问题

#### 1. 系统闲置时间检测不工作
**症状**：活跃度始终显示为"未知"
**解决方案**：
- **macOS**：确保应用有辅助功能权限
- **Linux**：安装 `xprintidle` 或 `xssstate` 工具
- **Windows**：确保应用有足够的系统权限

#### 2. 视频检测不准确
**症状**：观看视频时仍被标记为闲置
**解决方案**：
- 检查视频应用是否在支持列表中
- 添加自定义视频应用到配置文件
- 确保窗口标题包含可识别的关键词

#### 3. 检测过于敏感
**症状**：短暂停止操作就被标记为闲置
**解决方案**：
- 增加 `idle_timeout` 值
- 调整 `check_interval` 减少检测频率

### 调试模式
```bash
# 启用详细日志
RUST_LOG=debug timetracker activity test
```

## 🚀 最佳实践

### 推荐配置
- **办公环境**：闲置超时 5-10 分钟
- **个人使用**：闲置超时 3-5 分钟
- **服务器监控**：闲置超时 15-30 分钟

### 性能优化
- 适当调整检测间隔，避免过于频繁的检测
- 定期清理不需要的视频应用和网站配置
- 在不需要时禁用活跃度检测功能

## 📈 未来计划

- [ ] 支持更多视频平台和应用
- [ ] 添加机器学习算法提高检测准确性
- [ ] 支持用户行为模式学习
- [ ] 添加活跃度统计和报告功能
- [ ] 支持远程配置和管理
