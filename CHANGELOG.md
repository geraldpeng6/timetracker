# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- 🎯 **用户活跃度检测功能** - 智能检测用户是否处于活跃状态
  - 自动检测用户闲置时间，闲置时不记录窗口活动
  - 智能识别视频播放场景（看视频时即使闲置也记录）
  - 支持自定义视频应用和网站列表
  - 可配置的闲置超时时间和检测间隔
  - 新增 `timetracker activity` 命令管理活跃度检测
- 🚀 **完整部署系统** - 现代化的多平台部署解决方案
  - 一键安装脚本 (Linux/macOS: `install-remote.sh`, Windows: `install.ps1`)
  - 包管理器支持 (Homebrew, APT/DEB, RPM)
  - Docker 容器化部署 (多架构镜像)
  - Windows MSI 和 macOS PKG 安装器
  - GitHub Actions 完全自动化 CI/CD
  - 多平台二进制文件构建 (x86_64, ARM64)
  - 容器注册表集成 (GitHub Container Registry)
  - 自动化发布流程和版本管理
  - 部署配置验证和测试脚本

### Changed
- 更新依赖到最新版本
- 改进安装文档和说明
- 增强监控系统，集成活跃度检测功能
- 扩展配置系统，支持活跃度检测配置

### Fixed
- 修复 ratatui 升级后的兼容性问题
- 修复 sysinfo 库升级后的 API 变更
- 修复跨平台编译问题

## [0.2.0] - 2024-01-15

### Added
- 交互式 TUI 统计界面
- 实时数据刷新
- 多种排序选项
- 应用程序和窗口分类统计
- 最近活动记录
- 帮助界面

### Changed
- 重构代码架构，提高可维护性
- 改进错误处理和日志记录
- 优化性能和内存使用

### Fixed
- 修复 Windows 平台的窗口检测问题
- 修复 macOS 权限检查
- 修复数据持久化问题

## [0.1.0] - 2024-01-01

### Added
- 基本的时间追踪功能
- 跨平台支持 (Windows, macOS, Linux)
- CLI 命令行界面
- JSON 数据存储
- CSV 数据导出
- 守护进程模式
- 权限检查和请求

### Features
- 实时窗口监控
- 应用程序使用统计
- 数据导出功能
- 后台运行支持

[Unreleased]: https://github.com/geraldpeng6/timetracker/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/geraldpeng6/timetracker/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/geraldpeng6/timetracker/releases/tag/v0.1.0