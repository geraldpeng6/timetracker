# Contributing to TimeTracker

感谢你对 TimeTracker 项目的关注！我们欢迎各种形式的贡献。

## 🤝 如何贡献

### 报告 Bug
1. 在 [Issues](https://github.com/yourusername/timetracker/issues) 中搜索是否已有相关问题
2. 如果没有，创建新的 Issue，包含：
   - 详细的问题描述
   - 重现步骤
   - 期望的行为
   - 实际的行为
   - 系统信息（操作系统、版本等）
   - 相关的日志或错误信息

### 功能请求
1. 在 Issues 中创建功能请求
2. 详细描述你希望的功能
3. 解释为什么这个功能有用
4. 如果可能，提供使用场景

### 代码贡献

#### 开发环境设置
```bash
# 1. Fork 项目到你的 GitHub 账户
# 2. 克隆你的 fork
git clone https://github.com/yourusername/timetracker.git
cd timetracker

# 3. 添加上游仓库
git remote add upstream https://github.com/originalowner/timetracker.git

# 4. 安装 Rust（如果还没有）
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 5. 安装开发依赖
cargo install cargo-watch cargo-audit cargo-deb

# 6. 运行测试确保环境正常
cargo test
```

#### 开发流程
1. **创建分支**
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **进行开发**
   - 遵循现有的代码风格
   - 添加必要的测试
   - 更新文档

3. **测试你的更改**
   ```bash
   # 运行所有测试
   cargo test
   
   # 检查代码格式
   cargo fmt --check
   
   # 运行 Clippy 检查
   cargo clippy -- -D warnings
   
   # 安全审计
   cargo audit
   ```

4. **提交更改**
   ```bash
   git add .
   git commit -m "feat: add your feature description"
   ```

5. **推送并创建 PR**
   ```bash
   git push origin feature/your-feature-name
   ```
   然后在 GitHub 上创建 Pull Request

#### 提交信息规范
我们使用 [Conventional Commits](https://www.conventionalcommits.org/) 规范：

- `feat:` 新功能
- `fix:` Bug 修复
- `docs:` 文档更新
- `style:` 代码格式（不影响功能）
- `refactor:` 重构
- `test:` 测试相关
- `chore:` 构建过程或辅助工具的变动

示例：
```
feat: add TUI interface for statistics
fix: resolve window detection issue on Windows
docs: update installation instructions
```

#### 代码风格
- 使用 `cargo fmt` 格式化代码
- 遵循 Rust 官方风格指南
- 使用有意义的变量和函数名
- 添加适当的注释和文档

#### 测试
- 为新功能添加单元测试
- 确保所有测试通过
- 测试跨平台兼容性（如果可能）

## 📋 开发指南

### 项目结构
```
src/
├── main.rs          # 主程序入口和 CLI
├── tracker.rs       # 时间追踪核心逻辑
├── tui.rs          # TUI 界面
├── daemon.rs       # 守护进程管理
├── platform.rs     # 跨平台实现
└── permissions.rs  # 权限检查
```

### 添加新平台支持
1. 在 `platform.rs` 中添加平台检测
2. 实现平台特定的窗口监控逻辑
3. 添加相应的依赖到 `Cargo.toml`
4. 更新文档和测试

### 调试技巧
```bash
# 启用详细日志
RUST_LOG=debug cargo run -- start

# 使用 cargo-watch 自动重新编译
cargo watch -x run

# 性能分析
cargo build --release
perf record target/release/timetracker start
```

## 🔄 发布流程

### 版本发布
1. 更新 `CHANGELOG.md`
2. 更新 `Cargo.toml` 中的版本号
3. 运行发布脚本：
   ```bash
   ./scripts/release.sh patch  # 或 minor, major
   ```
4. GitHub Actions 会自动构建和发布

### 文档更新
- README.md 更新
- API 文档更新
- 示例代码更新

## 📞 联系方式

- GitHub Issues: [项目 Issues](https://github.com/yourusername/timetracker/issues)
- 邮件: your.email@example.com

## 📄 许可证

通过贡献代码，你同意你的贡献将在 [MIT License](LICENSE) 下发布。

---

再次感谢你的贡献！🎉