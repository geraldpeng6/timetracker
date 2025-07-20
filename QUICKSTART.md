# TimeTracker 快速开始指南

## 🚀 立即开始

### 1. 构建程序
```bash
cargo build --release
```

### 2. 开始追踪
```bash
./target/release/timetracker start
```

程序启动后会显示：
- 📁 数据文件位置
- ⏱️ 检查间隔设置
- 📋 使用说明
- 🔍 实时活动状态

### 3. 停止追踪
按 `Ctrl+C` 停止追踪，程序会自动保存数据并显示摘要。

### 4. 查看统计
```bash
./target/release/timetracker stats
```

### 5. 导出数据
```bash
./target/release/timetracker export --output my_report.csv
```

## 🔧 常用选项

### 自定义检查间隔
```bash
# 每3秒检查一次（平衡精度和性能）
./target/release/timetracker start --interval 3

# 每1秒检查一次（高精度）
./target/release/timetracker start --interval 1
```

### 自定义数据文件
```bash
# 使用自定义数据文件
./target/release/timetracker start --data-file work_tracking.json
./target/release/timetracker stats --data-file work_tracking.json
```

### 只查看今天的数据
```bash
./target/release/timetracker stats --today
```

## 💡 使用技巧

1. **间隔设置建议**：
   - 1-2秒：高精度，适合详细分析
   - 3-5秒：平衡精度和性能
   - 10秒+：省电模式，适合长期追踪

2. **多项目追踪**：
   ```bash
   # 工作项目
   ./target/release/timetracker start --data-file work.json
   
   # 个人项目
   ./target/release/timetracker start --data-file personal.json
   ```

3. **数据分析**：
   - JSON文件可以直接查看原始数据
   - CSV导出可以用Excel等工具分析
   - 使用 `--today` 选项查看当日统计

## ⚠️ 重要提示

### macOS 用户
首次运行时需要授予辅助功能权限：
1. 系统偏好设置 → 安全性与隐私 → 隐私 → 辅助功能
2. 添加终端应用或 TimeTracker
3. 如果看到 "Unknown Window"，说明需要权限设置

### Linux 用户
确保已安装 xdotool：
```bash
sudo apt install xdotool  # Ubuntu/Debian
sudo yum install xdotool   # CentOS/RHEL
```

## 🎯 实时显示功能

新版本的 `start` 命令现在提供：

✅ **启动信息**：
- 数据文件位置
- 检查间隔设置
- 使用说明

✅ **实时状态**：
- 当前活动窗口
- 活动持续时间
- 活动编号

✅ **停止摘要**：
- 总追踪时间
- 活动数量
- 数据保存确认

## 📚 更多信息

- `README.md` - 完整文档
- `EXAMPLES.md` - 详细使用示例
- `./demo.sh` - 功能演示脚本

## 🆘 常见问题

**Q: 程序显示 "Unknown Window" 怎么办？**
A: 这通常是权限问题，请按照上面的权限设置步骤操作。

**Q: 如何在后台运行？**
A: 可以使用 `nohup` 或 `screen`：
```bash
nohup ./target/release/timetracker start &
```

**Q: 数据文件在哪里？**
A: 默认在当前目录的 `timetracker_data.json`，可以用 `--data-file` 自定义。