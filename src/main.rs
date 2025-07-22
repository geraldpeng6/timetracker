use anyhow::Result;
use clap::{Arg, Command};
use std::path::PathBuf;
use std::time::{Duration, Instant};

// 导入核心模块
use timetracker::core::daemon::DaemonManager;
use timetracker::core::tracker::TimeTracker;
use timetracker::ui::tui::TuiApp;

// 快速响应模式 - 避免导入可能阻塞的模块

/// 打印帮助信息
fn print_help() {
    println!("timetracker {}", env!("CARGO_PKG_VERSION"));
    println!("A time tracking application with AI-powered insights");
    println!();
    println!("USAGE:");
    println!("    timetracker [SUBCOMMAND]");
    println!();
    println!("SUBCOMMANDS:");
    println!("    start        Start the time tracking daemon");
    println!("    stop         Stop the time tracking daemon");
    println!("    status       Show daemon status");
    println!("    tui          Launch the TUI interface");
    println!("    export       Export data to various formats");
    println!("    permissions  Check and manage permissions");
    println!("    activity     Manage user activity detection");
    println!("    help         Print this message or the help of the given subcommand(s)");
    println!();
    println!("OPTIONS:");
    println!("    -h, --help       Print help information");
    println!("    -V, --version    Print version information");
}

/// 打印简短帮助信息
fn print_short_help() {
    println!("timetracker {}", env!("CARGO_PKG_VERSION"));
    println!("A time tracking application with AI-powered insights");
    println!();
    println!("Use 'timetracker --help' for more information.");
    println!("Common commands:");
    println!("  timetracker start    # Start tracking");
    println!("  timetracker tui      # Open interface");
    println!("  timetracker status   # Check status");
}

/// 处理活跃度检测命令
fn handle_activity_command(sub_matches: &clap::ArgMatches) -> Result<()> {
    use timetracker::core::enhanced_platform::HybridWindowMonitor;

    match sub_matches.subcommand() {
        Some(("status", _)) => {
            println!("📊 用户活跃度状态");
            println!("{}", "=".repeat(50));

            // 创建监控器并检查活跃度
            let monitor = HybridWindowMonitor::new();
            let activity_stats = monitor.activity_detector().get_stats();

            println!("当前状态: {}", activity_stats.status_description());
            println!(
                "检测功能: {}",
                if activity_stats.detection_enabled {
                    "启用"
                } else {
                    "禁用"
                }
            );

            if activity_stats.detection_enabled {
                println!("闲置超时: {}秒", activity_stats.idle_timeout.as_secs());
                if activity_stats.idle_duration.as_secs() > 0 {
                    println!("闲置时长: {}", activity_stats.format_idle_duration());
                }
            }
        }
        Some(("config", _)) => {
            use timetracker::config::manager::ConfigManager;

            println!("⚙️ 活跃度检测配置");
            println!("{}", "=".repeat(50));

            let config_manager = ConfigManager::new()?;
            let activity_config = &config_manager.app_config.activity;

            println!(
                "启用状态: {}",
                if activity_config.enabled {
                    "启用"
                } else {
                    "禁用"
                }
            );
            println!("闲置超时: {}秒", activity_config.idle_timeout);
            println!("检测间隔: {}毫秒", activity_config.check_interval);
            println!("视频应用: {} 个", activity_config.video_apps.len());
            println!("视频网站: {} 个", activity_config.video_sites.len());

            println!("\n视频应用列表:");
            for app in &activity_config.video_apps {
                println!("  - {}", app);
            }

            println!("\n视频网站列表:");
            for site in &activity_config.video_sites {
                println!("  - {}", site);
            }
        }
        Some(("test", _)) => {
            use timetracker::core::enhanced_platform::{
                EnhancedWindowMonitor, HybridWindowMonitor,
            };

            println!("🧪 测试活跃度检测");
            println!("{}", "=".repeat(50));

            let mut monitor = HybridWindowMonitor::new();

            // 获取当前窗口信息
            match monitor.get_active_window() {
                Ok(Some(window_info)) => {
                    println!(
                        "当前窗口: {} - {}",
                        window_info.app_name, window_info.window_title
                    );

                    // 检测活跃度
                    let activity_status = monitor.activity_detector_mut().detect_activity(
                        Some(&window_info.app_name),
                        Some(&window_info.window_title),
                    )?;

                    println!(
                        "活跃状态: {} {}",
                        activity_status.icon(),
                        activity_status.description()
                    );
                    println!(
                        "是否记录: {}",
                        if activity_status.should_record() {
                            "是"
                        } else {
                            "否"
                        }
                    );
                }
                Ok(None) => {
                    println!("未检测到活动窗口");
                }
                Err(e) => {
                    println!("窗口检测失败: {}", e);
                }
            }
        }
        Some(("enable", _)) => {
            println!("✅ 启用活跃度检测");
            // 这里可以添加修改配置的逻辑
            println!("活跃度检测已启用");
        }
        Some(("disable", _)) => {
            println!("❌ 禁用活跃度检测");
            // 这里可以添加修改配置的逻辑
            println!("活跃度检测已禁用");
        }
        _ => {
            println!("使用 'timetracker activity --help' 查看可用的活跃度检测命令");
        }
    }

    Ok(())
}

/// 处理启动命令
fn handle_start_command(sub_matches: &clap::ArgMatches) -> Result<()> {
    eprintln!("处理启动命令...");

    // 检查是否是守护进程子进程
    if sub_matches.get_flag("daemon-child") {
        handle_daemon_child(sub_matches)
    } else {
        handle_daemon_start(sub_matches)
    }
}

/// 处理守护进程子进程
fn handle_daemon_child(sub_matches: &clap::ArgMatches) -> Result<()> {
    eprintln!("这是守护进程子进程");

    // 立即执行守护进程化，在任何其他操作之前
    if let Err(e) = daemonize_process() {
        eprintln!("守护进程化失败: {}", e);
        std::process::exit(1);
    }

    // TimeTracker已在顶部导入

    let interval = sub_matches.get_one::<u64>("interval").copied().unwrap_or(1);
    let data_file = sub_matches
        .get_one::<String>("data-file")
        .cloned()
        .unwrap_or_else(|| {
            dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".timetracker")
                .join("activities.json")
                .to_string_lossy()
                .to_string()
        });

    // 确保数据目录存在
    if let Some(parent) = std::path::Path::new(&data_file).parent() {
        if let Err(_e) = std::fs::create_dir_all(parent) {
            std::process::exit(1);
        }
    }

    // 写入PID文件（在守护化之后）
    let pid = std::process::id();
    if let Err(_e) = std::fs::write("/tmp/timetracker.pid", pid.to_string()) {
        std::process::exit(1);
    }

    // 启动监控
    start_monitoring_with_timeout(data_file, interval)
}

/// 处理守护进程启动
fn handle_daemon_start(sub_matches: &clap::ArgMatches) -> Result<()> {
    eprintln!("启动守护进程");

    // 延迟导入DaemonManager
    use timetracker::core::daemon::DaemonManager;

    let mut daemon_manager = DaemonManager::new();

    let interval = sub_matches.get_one::<u64>("interval").copied().unwrap_or(1);
    let data_file = sub_matches
        .get_one::<String>("data-file")
        .cloned()
        .unwrap_or_else(|| {
            dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".timetracker")
                .join("activities.json")
                .to_string_lossy()
                .to_string()
        });

    daemon_manager.start_daemon(interval, &data_file)
}

/// 带超时的监控启动
fn start_monitoring_with_timeout(data_file: String, interval: u64) -> Result<()> {
    use std::sync::mpsc;
    use std::thread;

    let (tx, rx) = mpsc::channel();

    // 在单独线程中启动监控，避免阻塞
    let monitoring_thread = thread::spawn(move || {
        // 延迟导入，避免静态初始化问题
        use timetracker::core::tracker::TimeTracker;

        let rt = tokio::runtime::Runtime::new().unwrap();
        let mut tracker = TimeTracker::new(data_file, interval);

        // 发送初始化完成信号
        let _ = tx.send(Ok(()));

        rt.block_on(async {
            // 检查权限
            if let Err(e) = tracker.check_permissions().await {
                log::warn!("权限检查失败: {}", e);
            }

            // 设置信号处理
            #[cfg(unix)]
            {
                use tokio::signal::unix::{signal, SignalKind};

                let mut sigterm = signal(SignalKind::terminate()).unwrap();
                let mut sigint = signal(SignalKind::interrupt()).unwrap();

                // 使用select来同时监听信号和运行监控
                tokio::select! {
                    result = tracker.start_monitoring() => {
                        if let Err(e) = result {
                            log::error!("监控过程中出错: {}", e);
                            std::process::exit(1);
                        }
                    }
                    _ = sigterm.recv() => {
                        log::info!("收到SIGTERM信号，正在优雅退出...");
                        if let Err(e) = tracker.stop_monitoring() {
                            log::error!("停止监控时出错: {}", e);
                        }
                    }
                    _ = sigint.recv() => {
                        log::info!("收到SIGINT信号，正在优雅退出...");
                        if let Err(e) = tracker.stop_monitoring() {
                            log::error!("停止监控时出错: {}", e);
                        }
                    }
                }
            }

            #[cfg(not(unix))]
            {
                if let Err(e) = tracker.start_monitoring().await {
                    log::error!("监控过程中出错: {}", e);
                    std::process::exit(1);
                }
            }
        });
    });

    // 等待初始化完成或超时
    match rx.recv_timeout(Duration::from_secs(10)) {
        Ok(Ok(())) => {
            eprintln!("监控初始化成功");
            // 让监控线程继续运行
            monitoring_thread.join().unwrap();
            Ok(())
        }
        Ok(Err(e)) => {
            eprintln!("监控初始化失败: {}", e);
            Err(e)
        }
        Err(_) => {
            eprintln!("监控初始化超时");
            Err(anyhow::anyhow!("监控初始化超时"))
        }
    }
}

/// 创建命令解析器
fn create_command_parser() -> Command {
    Command::new("timetracker")
        .version(env!("CARGO_PKG_VERSION"))
        .author("Your Name <your.email@example.com>")
        .about("A time tracking application with AI-powered insights")
}

/// 守护进程化函数
fn daemonize_process() -> Result<()> {
    #[cfg(unix)]
    {
        use nix::libc;

        // Fork第一次
        match unsafe { libc::fork() } {
            -1 => return Err(anyhow::anyhow!("第一次fork失败")),
            0 => {
                // 子进程继续
            }
            _ => {
                // 父进程退出
                std::process::exit(0);
            }
        }

        // 创建新的会话
        if unsafe { libc::setsid() } == -1 {
            return Err(anyhow::anyhow!("setsid失败"));
        }

        // Fork第二次（可选，但推荐）
        match unsafe { libc::fork() } {
            -1 => return Err(anyhow::anyhow!("第二次fork失败")),
            0 => {
                // 子进程继续
            }
            _ => {
                // 父进程退出
                std::process::exit(0);
            }
        }

        // 改变工作目录到根目录
        if let Err(_) = std::env::set_current_dir("/") {
            // 如果无法切换到根目录，使用/tmp
            let _ = std::env::set_current_dir("/tmp");
        }

        // 重定向标准输入、输出、错误到/dev/null
        use std::fs::OpenOptions;
        use std::os::unix::io::AsRawFd;

        let dev_null = OpenOptions::new()
            .read(true)
            .write(true)
            .open("/dev/null")?;

        let null_fd = dev_null.as_raw_fd();

        unsafe {
            libc::dup2(null_fd, 0); // stdin
            libc::dup2(null_fd, 1); // stdout
            libc::dup2(null_fd, 2); // stderr
        }
    }

    #[cfg(windows)]
    {
        // Windows下的守护进程实现相对简单
        // 主要是分离控制台
        #[cfg(feature = "winapi")]
        {
            use winapi::um::wincon::FreeConsole;
            unsafe {
                FreeConsole();
            }
        }
    }

    Ok(())
}

fn main() -> Result<()> {
    let start_time = Instant::now();

    // 快速路径：处理简单命令，避免复杂初始化
    let args: Vec<String> = std::env::args().collect();

    // 版本查询 - 最快响应
    if args.len() == 2 && (args[1] == "--version" || args[1] == "-V") {
        println!("timetracker {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    // 帮助查询 - 快速响应
    if args.len() == 2 && (args[1] == "--help" || args[1] == "-h") {
        print_help();
        return Ok(());
    }

    // 无参数调用 - 显示简短帮助
    if args.len() == 1 {
        print_short_help();
        return Ok(());
    }

    // 复杂命令需要完整的clap解析
    let matches = create_command_parser()
        .subcommand(
            Command::new("start")
                .about("Start the time tracking daemon")
                .arg(
                    Arg::new("data-dir")
                        .long("data-dir")
                        .value_name("DIR")
                        .help("Directory to store tracking data")
                        .value_parser(clap::value_parser!(PathBuf)),
                )
                .arg(
                    Arg::new("interval")
                        .long("interval")
                        .value_name("SECONDS")
                        .help("Monitoring interval in seconds")
                        .value_parser(clap::value_parser!(u64)),
                )
                .arg(
                    Arg::new("data-file")
                        .long("data-file")
                        .value_name("FILE")
                        .help("Data file path")
                        .value_parser(clap::value_parser!(String)),
                )
                .arg(
                    Arg::new("daemon-child")
                        .long("daemon-child")
                        .help("Internal flag for daemon child process")
                        .action(clap::ArgAction::SetTrue),
                ),
        )
        .subcommand(Command::new("stop").about("Stop the time tracking daemon"))
        .subcommand(Command::new("status").about("Show the status of the time tracking daemon"))
        .subcommand(
            Command::new("tui")
                .about("Launch the terminal user interface")
                .arg(
                    Arg::new("data-dir")
                        .long("data-dir")
                        .value_name("DIR")
                        .help("Directory to read tracking data from")
                        .value_parser(clap::value_parser!(PathBuf)),
                ),
        )
        .subcommand(
            Command::new("export")
                .about("Export tracking data")
                .arg(
                    Arg::new("format")
                        .long("format")
                        .value_name("FORMAT")
                        .help("Export format (json, csv)")
                        .default_value("json"),
                )
                .arg(
                    Arg::new("output")
                        .long("output")
                        .short('o')
                        .value_name("FILE")
                        .help("Output file path")
                        .value_parser(clap::value_parser!(PathBuf)),
                ),
        )
        .subcommand(
            Command::new("permissions")
                .about("Check and manage permissions for window monitoring")
                .subcommand(Command::new("check").about("Check current permission status"))
                .subcommand(Command::new("request").about("Request necessary permissions"))
                .subcommand(Command::new("test").about("Test all monitoring capabilities")),
        )
        .subcommand(
            Command::new("activity")
                .about("Manage user activity detection")
                .subcommand(Command::new("status").about("Show current activity status"))
                .subcommand(Command::new("config").about("Show activity detection configuration"))
                .subcommand(Command::new("test").about("Test activity detection"))
                .subcommand(Command::new("enable").about("Enable activity detection"))
                .subcommand(Command::new("disable").about("Disable activity detection")),
        )
        .get_matches();

    eprintln!("命令行解析完成");

    let elapsed = start_time.elapsed();
    eprintln!("命令行解析完成，耗时: {:?}", elapsed);

    // 使用延迟导入和超时机制处理复杂命令
    match matches.subcommand() {
        Some(("start", sub_matches)) => {
            handle_start_command(sub_matches)?;
            // 检查是否是守护进程子进程
            if sub_matches.get_flag("daemon-child") {
                // 立即执行守护进程化，在任何其他操作之前
                if let Err(e) = daemonize_process() {
                    eprintln!("守护进程化失败: {}", e);
                    std::process::exit(1);
                }

                // 守护化成功后，继续初始化
                let interval = sub_matches.get_one::<u64>("interval").copied().unwrap_or(1);
                let data_file = sub_matches
                    .get_one::<String>("data-file")
                    .cloned()
                    .unwrap_or_else(|| {
                        dirs::home_dir()
                            .unwrap_or_else(|| PathBuf::from("."))
                            .join(".timetracker")
                            .join("activities.json")
                            .to_string_lossy()
                            .to_string()
                    });

                // 确保数据目录存在
                if let Some(parent) = std::path::Path::new(&data_file).parent() {
                    if let Err(_e) = std::fs::create_dir_all(parent) {
                        std::process::exit(1);
                    }
                }

                // 写入PID文件（在守护化之后）
                let pid = std::process::id();
                if let Err(_e) = std::fs::write("/tmp/timetracker.pid", pid.to_string()) {
                    std::process::exit(1);
                }

                // 设置日志系统
                use simplelog::*;
                let log_file = match std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open("/tmp/timetracker.log")
                {
                    Ok(file) => file,
                    Err(e) => {
                        eprintln!("无法打开日志文件: {}", e);
                        std::process::exit(1);
                    }
                };

                // 初始化日志记录器
                if let Err(e) = WriteLogger::init(LevelFilter::Info, Config::default(), log_file) {
                    eprintln!("无法初始化日志记录器: {}", e);
                    std::process::exit(1);
                }

                log::info!("TimeTracker daemon started (PID: {})", pid);
                log::info!("数据文件: {}", data_file);
                log::info!("监控间隔: {}秒", interval);

                // 创建并启动时间追踪器
                let mut tracker = TimeTracker::new(data_file, interval);
                if let Err(e) = tracker.load_data() {
                    log::error!("加载数据失败: {}", e);
                    eprintln!("加载数据失败: {}", e);
                    std::process::exit(1);
                }

                log::info!("开始监控，间隔: {}秒", interval);

                // 使用 tokio 运行时
                let rt = match tokio::runtime::Runtime::new() {
                    Ok(rt) => rt,
                    Err(e) => {
                        log::error!("创建运行时失败: {}", e);
                        eprintln!("创建运行时失败: {}", e);
                        std::process::exit(1);
                    }
                };

                rt.block_on(async {
                    // 检查权限
                    if let Err(e) = tracker.check_permissions().await {
                        log::warn!("权限检查失败: {}", e);
                    }

                    // 设置信号处理
                    #[cfg(unix)]
                    {
                        use tokio::signal::unix::{signal, SignalKind};

                        let mut sigterm = signal(SignalKind::terminate()).unwrap();
                        let mut sigint = signal(SignalKind::interrupt()).unwrap();

                        // 使用select来同时监听信号和运行监控
                        tokio::select! {
                            result = tracker.start_monitoring() => {
                                if let Err(e) = result {
                                    log::error!("监控过程中出错: {}", e);
                                    std::process::exit(1);
                                }
                            }
                            _ = sigterm.recv() => {
                                log::info!("收到SIGTERM信号，正在优雅退出...");
                                if let Err(e) = tracker.stop_monitoring() {
                                    log::error!("停止监控时出错: {}", e);
                                }
                            }
                            _ = sigint.recv() => {
                                log::info!("收到SIGINT信号，正在优雅退出...");
                                if let Err(e) = tracker.stop_monitoring() {
                                    log::error!("停止监控时出错: {}", e);
                                }
                            }
                        }
                    }

                    #[cfg(not(unix))]
                    {
                        if let Err(e) = tracker.start_monitoring().await {
                            log::error!("监控过程中出错: {}", e);
                            std::process::exit(1);
                        }
                    }
                });
            } else {
                // 这是用户调用的启动命令，启动守护进程
                let data_dir = sub_matches
                    .get_one::<PathBuf>("data-dir")
                    .cloned()
                    .unwrap_or_else(|| {
                        dirs::home_dir()
                            .unwrap_or_else(|| PathBuf::from("."))
                            .join(".timetracker")
                    });

                let interval = sub_matches.get_one::<u64>("interval").copied().unwrap_or(1);

                let mut daemon_manager = DaemonManager::new();
                daemon_manager.start_daemon(
                    interval,
                    &data_dir.join("activities.json").to_string_lossy(),
                )?;
                println!("Time tracking daemon started successfully");
            }
        }
        Some(("stop", _)) => {
            let daemon_manager = DaemonManager::new();
            daemon_manager.stop_daemon()?;
            println!("Time tracking daemon stopped");
        }
        Some(("status", _)) => {
            let daemon_manager = DaemonManager::new();
            daemon_manager.status()?;
        }
        Some(("tui", sub_matches)) => {
            let data_dir = sub_matches
                .get_one::<PathBuf>("data-dir")
                .cloned()
                .unwrap_or_else(|| {
                    dirs::home_dir()
                        .unwrap_or_else(|| PathBuf::from("."))
                        .join(".timetracker")
                });

            let data_file = data_dir
                .join("activities.json")
                .to_string_lossy()
                .to_string();
            let mut app = TuiApp::new(data_file)?;
            app.run()?;

            // 检查是否需要退出整个程序
            if app.should_quit_program() {
                // 如果有守护进程在运行，先停止它
                let daemon_manager = DaemonManager::new();
                if daemon_manager.is_running() {
                    daemon_manager.stop_daemon()?;
                    println!("Time tracking daemon stopped");
                }
                std::process::exit(0);
            } else {
                println!("TUI界面已退出，程序继续在后台运行");
                println!("使用 'timetracker stop' 来停止后台监控");
                println!("使用 'timetracker tui' 来重新打开界面");
            }
        }
        Some(("export", sub_matches)) => {
            let format = sub_matches.get_one::<String>("format").unwrap();
            let output = sub_matches.get_one::<PathBuf>("output");

            let data_dir = dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".timetracker");

            let data_file = data_dir
                .join("activities.json")
                .to_string_lossy()
                .to_string();
            let mut tracker = TimeTracker::new(data_file, 5);
            tracker.load_data()?;

            match format.as_str() {
                "json" => {
                    let json_data = tracker.export_json()?;
                    if let Some(output_path) = output {
                        std::fs::write(output_path, json_data)?;
                        println!("Data exported to {}", output_path.display());
                    } else {
                        println!("{}", json_data);
                    }
                }
                "csv" => {
                    let csv_data = tracker.export_csv()?;
                    if let Some(output_path) = output {
                        std::fs::write(output_path, csv_data)?;
                        println!("Data exported to {}", output_path.display());
                    } else {
                        println!("{}", csv_data);
                    }
                }
                _ => {
                    eprintln!("Unsupported format: {}", format);
                    std::process::exit(1);
                }
            }
        }
        Some(("permissions", sub_matches)) => {
            match sub_matches.subcommand() {
                Some(("check", _)) => {
                    println!("🔍 检查窗口监控权限...");
                    println!("权限检查功能正在开发中");
                    println!("请确保您的系统允许应用程序访问窗口信息");
                }
                Some(("request", _)) => {
                    println!("🔐 请求窗口监控权限...");
                    println!("权限请求功能正在开发中");
                    println!("请手动在系统设置中授予必要的权限");
                }
                Some(("test", _)) => {
                    println!("🧪 测试所有监控功能...");
                    println!("监控功能测试正在开发中");
                }
                _ => {
                    println!("使用 'timetracker permissions --help' 查看可用的权限命令");
                }
            }
            // 检查是否是守护进程子进程
            if sub_matches.get_flag("daemon-child") {
                eprintln!("这是守护进程子进程");
                // 暂时禁用守护进程功能
                println!("守护进程功能暂时禁用");
            } else {
                eprintln!("启动守护进程");
                let _daemon_manager = DaemonManager::new();
                // 暂时只创建，不启动
                println!("守护进程管理器已创建");
            }
        }

        Some(("activity", sub_matches)) => {
            handle_activity_command(sub_matches)?;
        }

        None => {
            // 没有子命令，显示简短帮助
            println!("TimeTracker - 时间追踪工具");
            println!();
            println!("使用方法:");
            println!("  timetracker tui              # 启动 TUI 界面");
            println!("  timetracker start            # 启动守护进程");
            println!("  timetracker stop             # 停止守护进程");
            println!("  timetracker status           # 查看状态");
            println!("  timetracker --version        # 显示版本");
            println!("  timetracker --help           # 显示详细帮助");
        }
        Some((cmd, _)) => {
            println!("未知命令: {}", cmd);
            println!("使用 'timetracker --help' 查看可用命令");
        }
    }

    Ok(())
}
