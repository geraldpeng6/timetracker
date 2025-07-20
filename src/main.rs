mod ai_analysis;
mod ai_client;
mod ai_config;
mod ai_config_manager;
mod daemon;
mod permissions;
mod platform;
mod tracker;
mod tui;

use ai_analysis::AIAnalyzer;
use ai_config_manager::AIConfigManager;
use anyhow::Result;
use clap::{Arg, ArgMatches, Command};
use daemon::DaemonManager;
use permissions::auto_request_permissions;

use tokio::signal;
use tracker::TimeTracker;
use tui::TuiApp;

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    env_logger::init();

    let matches = Command::new("timetracker")
        .version("0.2.0")
        .author("TimeTracker Team")
        .about("智能时间追踪工具 - 监控应用程序使用情况")
        .subcommand(
            Command::new("start")
                .about("开始时间追踪")
                .arg(
                    Arg::new("interval")
                        .short('i')
                        .long("interval")
                        .value_name("SECONDS")
                        .help("监控间隔（秒），最小值为1，默认为5")
                        .default_value("5"),
                )
                .arg(
                    Arg::new("data-file")
                        .short('f')
                        .long("data-file")
                        .value_name("FILE")
                        .help("数据文件路径")
                        .default_value("timetracker_data.json"),
                )
                .arg(
                    Arg::new("daemon")
                        .short('d')
                        .long("daemon")
                        .help("以守护进程模式运行（默认）")
                        .action(clap::ArgAction::SetTrue),
                )
                .arg(
                    Arg::new("interactive")
                        .short('I')
                        .long("interactive")
                        .help("以交互式模式运行")
                        .action(clap::ArgAction::SetTrue),
                )
                .arg(
                    Arg::new("daemon-child")
                        .long("daemon-child")
                        .help("内部使用：守护进程子进程标志")
                        .action(clap::ArgAction::SetTrue)
                        .hide(true),
                ),
        )
        .subcommand(Command::new("stop").about("停止时间追踪守护进程"))
        .subcommand(Command::new("status").about("查看守护进程状态"))
        .subcommand(
            Command::new("restart")
                .about("重启时间追踪守护进程")
                .arg(
                    Arg::new("interval")
                        .short('i')
                        .long("interval")
                        .value_name("SECONDS")
                        .help("监控间隔（秒），最小值为1，默认为5")
                        .default_value("5"),
                )
                .arg(
                    Arg::new("data-file")
                        .short('f')
                        .long("data-file")
                        .value_name("FILE")
                        .help("数据文件路径")
                        .default_value("timetracker_data.json"),
                ),
        )
        .subcommand(
            Command::new("stats").about("显示交互式统计界面").arg(
                Arg::new("data-file")
                    .short('f')
                    .long("data-file")
                    .value_name("FILE")
                    .help("数据文件路径")
                    .default_value("timetracker_data.json"),
            ),
        )
        .subcommand(
            Command::new("export")
                .about("导出数据")
                .arg(
                    Arg::new("output")
                        .short('o')
                        .long("output")
                        .value_name("FILE")
                        .help("输出文件路径")
                        .required(true),
                )
                .arg(
                    Arg::new("data-file")
                        .short('d')
                        .long("data-file")
                        .value_name("FILE")
                        .help("数据文件路径")
                        .default_value("timetracker_data.json"),
                )
                .arg(
                    Arg::new("format")
                        .short('f')
                        .long("format")
                        .value_name("FORMAT")
                        .help("导出格式 (json, csv)")
                        .default_value("json"),
                ),
        )
        .subcommand(Command::new("permissions").about("检查和请求必要权限"))
        .subcommand(
            Command::new("analyze")
                .about("AI 分析使用情况")
                .arg(
                    Arg::new("data-file")
                        .short('f')
                        .long("data-file")
                        .value_name("FILE")
                        .help("数据文件路径")
                        .default_value("timetracker_data.json"),
                )
                .arg(
                    Arg::new("local")
                        .short('l')
                        .long("local")
                        .help("使用本地分析（不调用 AI API）")
                        .action(clap::ArgAction::SetTrue),
                )
                .arg(
                    Arg::new("output")
                        .short('o')
                        .long("output")
                        .value_name("FILE")
                        .help("保存分析结果到文件"),
                ),
        )
        .subcommand(
            Command::new("ai")
                .about("AI 配置管理")
                .subcommand(
                    Command::new("config")
                        .about("配置 AI 提供商")
                        .arg(
                            Arg::new("provider")
                                .short('p')
                                .long("provider")
                                .value_name("PROVIDER")
                                .help(
                                    "AI 提供商 (openai, anthropic, google, baidu, alibaba, local)",
                                )
                                .required(true),
                        )
                        .arg(
                            Arg::new("model")
                                .short('m')
                                .long("model")
                                .value_name("MODEL")
                                .help("模型名称"),
                        )
                        .arg(
                            Arg::new("api-key")
                                .short('k')
                                .long("api-key")
                                .value_name("KEY")
                                .help("API 密钥"),
                        )
                        .arg(
                            Arg::new("endpoint")
                                .short('e')
                                .long("endpoint")
                                .value_name("URL")
                                .help("自定义 API 端点"),
                        ),
                )
                .subcommand(Command::new("list").about("列出可用的 AI 提供商和模型"))
                .subcommand(Command::new("show").about("显示当前 AI 配置"))
                .subcommand(
                    Command::new("select").about("选择默认 AI 提供商").arg(
                        Arg::new("provider")
                            .short('p')
                            .long("provider")
                            .value_name("PROVIDER")
                            .help("AI 提供商")
                            .required(true),
                    ),
                )
                .subcommand(
                    Command::new("test").about("测试 AI 配置").arg(
                        Arg::new("provider")
                            .short('p')
                            .long("provider")
                            .value_name("PROVIDER")
                            .help("要测试的 AI 提供商"),
                    ),
                ),
        )
        .get_matches();

    match matches.subcommand() {
        Some(("start", sub_matches)) => {
            let interval_str = sub_matches.get_one::<String>("interval").unwrap();
            let interval = match interval_str.parse::<u64>() {
                Ok(val) => {
                    if val < 1 {
                        println!("❌ 错误：监控间隔不能小于1秒");
                        println!("💡 建议：使用1-60秒之间的值，推荐5秒");
                        return Ok(());
                    } else if val > 3600 {
                        println!("⚠️  警告：监控间隔过长（{}秒），可能影响数据准确性", val);
                        println!("💡 建议：使用1-60秒之间的值，推荐5秒");
                        println!("是否继续？(y/N): ");
                        let mut input = String::new();
                        std::io::stdin().read_line(&mut input)?;
                        if !input.trim().to_lowercase().starts_with('y') {
                            println!("操作已取消");
                            return Ok(());
                        }
                    }
                    val
                }
                Err(_) => {
                    println!("❌ 错误：无效的监控间隔 '{}'", interval_str);
                    println!("💡 请输入一个有效的数字（秒），例如：5");
                    return Ok(());
                }
            };

            let data_file = sub_matches
                .get_one::<String>("data-file")
                .unwrap()
                .to_string();

            // 验证数据文件路径
            if let Some(parent) = std::path::Path::new(&data_file).parent() {
                if !parent.as_os_str().is_empty() && !parent.exists() {
                    println!("❌ 错误：数据文件目录不存在: {}", parent.display());
                    println!("💡 请确保目录存在或使用默认路径");
                    return Ok(());
                }
            }

            let daemon_mode = sub_matches.get_flag("daemon");
            let interactive_mode = sub_matches.get_flag("interactive");
            let daemon_child = sub_matches.get_flag("daemon-child");

            // 检查冲突的参数
            if daemon_mode && interactive_mode {
                println!("❌ 错误：不能同时指定 --daemon 和 --interactive 参数");
                return Ok(());
            }

            if daemon_child {
                // 守护进程子进程模式 - 实际运行监控
                use crate::daemon::setup_signal_handlers;
                setup_signal_handlers()?;

                // 检查权限
                if !auto_request_permissions()? {
                    std::process::exit(1);
                }

                // 创建日志文件
                let log_file = std::path::Path::new("/tmp/timetracker.log");
                let mut log_handle = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(log_file)?;

                // 创建PID文件（在启动监控之前）
                let pid_file = std::path::Path::new("/tmp/timetracker.pid");
                let current_pid = std::process::id();

                // 直接写入PID文件
                std::fs::write(pid_file, current_pid.to_string())?;

                // 立即验证PID文件内容
                match std::fs::read_to_string(pid_file) {
                    Ok(content) => {
                        use std::io::Write;
                        writeln!(
                            log_handle,
                            "[{}] PID文件验证成功，内容: '{}'",
                            chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                            content
                        )?;
                    }
                    Err(e) => {
                        use std::io::Write;
                        writeln!(
                            log_handle,
                            "[{}] PID文件验证失败: {}",
                            chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                            e
                        )?;
                    }
                }

                // 启动监控（同步模式，不需要tokio）
                start_daemon_tracking(interval, data_file)?;
            } else if interactive_mode {
                // 交互式模式
                // 检查权限
                if !auto_request_permissions()? {
                    return Ok(());
                }

                start_interactive_tracking(interval, data_file).await?;
            } else {
                // 默认守护进程模式 - 启动子进程
                let mut daemon_manager = DaemonManager::new();
                daemon_manager.start_daemon(interval, &data_file)?;
            }
        }
        Some(("stop", _)) => {
            let daemon_manager = DaemonManager::new();
            daemon_manager.stop_daemon()?;
        }
        Some(("status", _)) => {
            let daemon_manager = DaemonManager::new();
            daemon_manager.status()?;
        }
        Some(("restart", sub_matches)) => {
            let interval_str = sub_matches.get_one::<String>("interval").unwrap();
            let interval = match interval_str.parse::<u64>() {
                Ok(val) => {
                    if val < 1 {
                        println!("❌ 错误：监控间隔不能小于1秒");
                        println!("💡 建议：使用1-60秒之间的值，推荐5秒");
                        return Ok(());
                    } else if val > 3600 {
                        println!("⚠️  警告：监控间隔过长（{}秒），可能影响数据准确性", val);
                        println!("💡 建议：使用1-60秒之间的值，推荐5秒");
                        println!("是否继续？(y/N): ");
                        let mut input = String::new();
                        std::io::stdin().read_line(&mut input)?;
                        if !input.trim().to_lowercase().starts_with('y') {
                            println!("操作已取消");
                            return Ok(());
                        }
                    }
                    val
                }
                Err(_) => {
                    println!("❌ 错误：无效的监控间隔 '{}'", interval_str);
                    println!("💡 请输入一个有效的数字（秒），例如：5");
                    return Ok(());
                }
            };

            let data_file = sub_matches
                .get_one::<String>("data-file")
                .unwrap()
                .to_string();

            // 验证数据文件路径
            if let Some(parent) = std::path::Path::new(&data_file).parent() {
                if !parent.as_os_str().is_empty() && !parent.exists() {
                    println!("❌ 错误：数据文件目录不存在: {}", parent.display());
                    println!("💡 请确保目录存在或使用默认路径");
                    return Ok(());
                }
            }

            let mut daemon_manager = DaemonManager::new();
            daemon_manager.restart_daemon(interval, &data_file)?;
        }
        Some(("stats", sub_matches)) => {
            let data_file = sub_matches
                .get_one::<String>("data-file")
                .unwrap()
                .to_string();

            // 验证数据文件是否存在
            if !std::path::Path::new(&data_file).exists() {
                println!("❌ 错误：数据文件不存在: {}", data_file);
                println!("💡 请先运行 'timetracker start' 收集数据");
                return Ok(());
            }

            show_interactive_stats(data_file)?;
        }
        Some(("export", sub_matches)) => {
            let output = sub_matches.get_one::<String>("output").unwrap();
            let data_file = sub_matches
                .get_one::<String>("data-file")
                .unwrap()
                .to_string();
            let format = sub_matches.get_one::<String>("format").unwrap();

            // 验证输出文件路径
            if let Some(parent) = std::path::Path::new(output).parent() {
                if !parent.as_os_str().is_empty() && !parent.exists() {
                    println!("❌ 错误：输出目录不存在: {}", parent.display());
                    println!("💡 请确保目录存在或选择其他路径");
                    return Ok(());
                }
            }

            // 验证数据文件是否存在
            if !std::path::Path::new(&data_file).exists() {
                println!("❌ 错误：数据文件不存在: {}", data_file);
                println!("💡 请先运行 'timetracker start' 收集数据");
                return Ok(());
            }

            // 验证导出格式
            if !matches!(format.as_str(), "json" | "csv") {
                println!("❌ 错误：不支持的导出格式 '{}'", format);
                println!("💡 支持的格式：json, csv");
                return Ok(());
            }

            export_data(&data_file, output, format)?;
        }
        Some(("permissions", _)) => {
            permissions::check_permissions()?;
        }
        Some(("analyze", sub_matches)) => {
            let data_file = sub_matches
                .get_one::<String>("data-file")
                .unwrap()
                .to_string();
            let use_local = sub_matches.get_flag("local");
            let output_file = sub_matches.get_one::<String>("output");

            // 验证数据文件是否存在
            if !std::path::Path::new(&data_file).exists() {
                println!("❌ 错误：数据文件不存在: {}", data_file);
                println!("💡 请先运行 'timetracker start' 收集数据");
                return Ok(());
            }

            // 验证输出文件路径（如果指定）
            if let Some(output_path) = output_file {
                if let Some(parent) = std::path::Path::new(output_path).parent() {
                    if !parent.exists() {
                        println!("❌ 错误：输出目录不存在: {}", parent.display());
                        println!("💡 请确保目录存在或选择其他路径");
                        return Ok(());
                    }
                }
            }

            analyze_usage(&data_file, use_local, output_file).await?;
        }
        Some(("ai", sub_matches)) => {
            handle_ai_command(sub_matches).await?;
        }
        _ => {
            println!("TimeTracker v0.2.0 - 智能时间追踪工具");
            println!();
            println!("使用方法:");
            println!("  timetracker start [选项]     - 开始时间追踪");
            println!("  timetracker stop             - 停止守护进程");
            println!("  timetracker status           - 查看状态");
            println!("  timetracker restart [选项]   - 重启守护进程");
            println!("  timetracker stats [选项]     - 显示交互式统计");
            println!("  timetracker export [选项]    - 导出数据");
            println!("  timetracker analyze [选项]   - AI 分析使用情况");
            println!("  timetracker ai [子命令]      - AI 配置管理");
            println!("  timetracker permissions      - 检查权限");
            println!();
            println!("使用 'timetracker <命令> --help' 查看具体命令的帮助信息");
        }
    }

    Ok(())
}

fn start_daemon_tracking(interval: u64, data_file: String) -> Result<()> {
    use std::io::Write;

    let mut tracker = TimeTracker::new(data_file.clone(), interval);
    tracker.load_data()?;

    // 写入启动日志（使用append模式）
    let log_msg = format!(
        "[{}] TimeTracker 守护进程已启动，数据文件: {}, 检查间隔: {}秒\n",
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
        data_file,
        interval
    );
    let _ = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("/tmp/timetracker.log")
        .and_then(|mut f| f.write_all(log_msg.as_bytes()));

    // 启动同步监控循环
    let mut loop_count = 0;
    let mut last_app = String::new();
    let mut last_window = String::new();
    
    loop {
        loop_count += 1;
        
        // 每60次循环记录一次心跳日志（约5分钟）
        if loop_count % 60 == 1 {
            let log_msg = format!(
                "[{}] 守护进程运行正常，已完成 {} 次检查\n",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                loop_count
            );
            let _ = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open("/tmp/timetracker.log")
                .and_then(|mut f| f.write_all(log_msg.as_bytes()));
        }

        match platform::get_active_window() {
            Ok(window_info) => {
                // 只在应用或窗口发生变化时记录日志
                if window_info.app_name != last_app || window_info.window_title != last_window {
                    let log_msg = format!(
                        "[{}] 活动窗口变化: {} - {}\n",
                        chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                        window_info.app_name,
                        window_info.window_title
                    );
                    let _ = std::fs::OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open("/tmp/timetracker.log")
                        .and_then(|mut f| f.write_all(log_msg.as_bytes()));
                    
                    last_app = window_info.app_name.clone();
                    last_window = window_info.window_title.clone();
                }

                if let Err(e) = tracker.update_activity(window_info) {
                    let log_msg = format!(
                        "[{}] 更新活动错误: {}\n",
                        chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                        e
                    );
                    let _ = std::fs::OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open("/tmp/timetracker.log")
                        .and_then(|mut f| f.write_all(log_msg.as_bytes()));
                }
            }
            Err(e) => {
                let log_msg = format!(
                    "[{}] 获取活动窗口错误: {}\n",
                    chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                    e
                );
                let _ = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open("/tmp/timetracker.log")
                    .and_then(|mut f| f.write_all(log_msg.as_bytes()));
            }
        }

        std::thread::sleep(std::time::Duration::from_secs(interval));
    }
}

async fn start_interactive_tracking(interval: u64, data_file: String) -> Result<()> {
    let mut tracker = TimeTracker::new(data_file.clone(), interval);
    tracker.load_data()?;

    println!("🚀 TimeTracker 已启动");
    println!("📁 数据文件: {}", data_file);
    println!("⏱️  检查间隔: {}秒", interval);
    println!();
    println!("💡 使用说明:");
    println!("  • 程序将自动监控您的应用程序使用情况");
    println!("  • 按 Ctrl+C 停止追踪");
    println!("  • 使用 'timetracker stats' 查看统计信息");
    println!("  • 使用 'timetracker export -o data.json' 导出数据");
    println!();

    // 设置信号处理
    let mut tracker_clone = tracker;
    tokio::select! {
        result = tracker_clone.start_monitoring() => {
            if let Err(e) = result {
                eprintln!("监控过程中出错: {}", e);
            }
        }
        _ = signal::ctrl_c() => {
            println!("\n\n🛑 收到停止信号，正在保存数据...");
            tracker_clone.stop_monitoring()?;

            // 显示会话总结
            let total_time = tracker_clone.get_total_time();
            let hours = total_time / 3600;
            let minutes = (total_time % 3600) / 60;
            let seconds = total_time % 60;

            println!("📊 本次会话统计:");
            println!("  总追踪时间: {}h {}m {}s", hours, minutes, seconds);
            println!("  活动记录数: {}", tracker_clone.get_activities().len());
            println!("✅ 数据已保存到: {}", data_file);
            println!("👋 感谢使用 TimeTracker！");
        }
    }

    Ok(())
}

fn show_interactive_stats(data_file: String) -> Result<()> {
    let tracker = TimeTracker::new(data_file, 5); // 间隔在这里不重要
    let mut app = TuiApp::new(tracker)?;
    app.run()?;
    Ok(())
}

fn export_data(data_file: &str, output: &str, format: &str) -> Result<()> {
    let mut tracker = TimeTracker::new(data_file.to_string(), 5);
    tracker.load_data()?;

    if tracker.get_activities().is_empty() {
        println!("⚠️  没有找到活动数据，请先运行 'timetracker start' 收集数据");
        return Ok(());
    }

    match format {
        "json" => {
            let json = serde_json::to_string_pretty(tracker.get_activities())?;
            std::fs::write(output, json)?;
            println!("✅ 数据已导出到: {} (JSON格式)", output);
            println!("📊 导出了 {} 条活动记录", tracker.get_activities().len());
        }
        "csv" => {
            export_to_csv(&tracker, output)?;
            println!("✅ 数据已导出到: {} (CSV格式)", output);
            println!("📊 导出了 {} 条活动记录", tracker.get_activities().len());
        }
        _ => {
            return Err(anyhow::anyhow!("不支持的导出格式: {}", format));
        }
    }

    Ok(())
}

fn export_to_csv(tracker: &TimeTracker, output: &str) -> Result<()> {
    use std::fs::File;
    use std::io::Write;

    let mut file = File::create(output)?;

    // 写入CSV头部
    writeln!(
        file,
        "App Name,Window Title,Process ID,Start Time,End Time,Duration (seconds)"
    )?;

    // 写入数据
    for activity in tracker.get_activities() {
        writeln!(
            file,
            "\"{}\",\"{}\",{},\"{}\",\"{}\",{}",
            activity.app_name,
            activity.window_title,
            activity.process_id,
            activity.start_time.format("%Y-%m-%d %H:%M:%S"),
            activity
                .end_time
                .as_ref()
                .map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string())
                .unwrap_or_else(|| "Still Active".to_string()),
            activity.duration
        )?;
    }

    Ok(())
}

async fn analyze_usage(
    data_file: &str,
    use_local: bool,
    output_file: Option<&String>,
) -> Result<()> {
    let mut tracker = TimeTracker::new(data_file.to_string(), 5);
    tracker.load_data()?;

    if tracker.get_activities().is_empty() {
        println!("❌ 没有找到活动数据，请先运行 'timetracker start' 收集数据");
        return Ok(());
    }

    println!("🔍 正在分析使用情况...");
    println!("📁 数据文件: {}", data_file);
    println!("📊 活动记录数: {}", tracker.get_activities().len());
    println!();

    let analyzer = match AIAnalyzer::new() {
        Ok(analyzer) => analyzer,
        Err(e) => {
            println!("❌ 初始化AI分析器失败: {}", e);
            return Ok(());
        }
    };

    let analysis_result = if use_local || !analyzer.is_configured() {
        if !use_local && !analyzer.is_configured() {
            println!("⚠️  未配置 AI API，使用本地分析");
            println!("💡 要使用 AI 分析，请设置环境变量: export OPENAI_API_KEY=your_api_key");
            println!();
        }
        analyzer.local_analysis(&tracker)?
    } else {
        println!("🤖 正在调用 AI API 进行分析...");
        match analyzer.analyze_usage(&tracker).await {
            Ok(result) => result,
            Err(e) => {
                println!("❌ AI 分析失败: {}", e);
                println!("🔄 回退到本地分析...");
                analyzer.local_analysis(&tracker)?
            }
        }
    };

    // 显示分析结果
    display_analysis_result(&analysis_result);

    // 保存到文件（如果指定）
    if let Some(output_path) = output_file {
        let json_result = serde_json::to_string_pretty(&analysis_result)?;
        std::fs::write(output_path, json_result)?;
        println!("\n💾 分析结果已保存到: {}", output_path);
    }

    Ok(())
}

fn display_analysis_result(result: &ai_analysis::AIAnalysisResult) {
    println!("📋 === 使用情况分析报告 ===");
    println!();

    // 总结
    println!("📝 总结:");
    println!("   {}", result.summary);
    println!();

    // 生产力评分
    if let Some(score) = result.productivity_score {
        println!("🎯 生产力评分: {:.1}/100", score);
        let emoji = if score >= 80.0 {
            "🔥"
        } else if score >= 60.0 {
            "👍"
        } else if score >= 40.0 {
            "⚠️"
        } else {
            "🔴"
        };
        println!("   {} {}", emoji, get_productivity_comment(score));
        println!();
    }

    // 时间分布
    if !result.time_distribution.is_empty() {
        println!("⏰ 时间分布:");
        for (category, percentage) in &result.time_distribution {
            println!("   • {}: {}", category, percentage);
        }
        println!();
    }

    // 关键洞察
    if !result.insights.is_empty() {
        println!("💡 关键洞察:");
        for insight in &result.insights {
            println!("   • {}", insight);
        }
        println!();
    }

    // 改进建议
    if !result.recommendations.is_empty() {
        println!("🚀 改进建议:");
        for recommendation in &result.recommendations {
            println!("   • {}", recommendation);
        }
        println!();
    }

    // 专注时段
    if !result.focus_periods.is_empty() {
        println!("🎯 专注时段 (超过30分钟):");
        for period in &result.focus_periods {
            let hours = period.duration / 3600;
            let minutes = (period.duration % 3600) / 60;
            println!(
                "   • {}: {}h{}m ({})",
                period.app_name,
                hours,
                minutes,
                period.start_time.format("%H:%M")
            );
        }
        println!();
    }

    println!("✨ 分析完成！");
}

fn get_productivity_comment(score: f32) -> &'static str {
    if score >= 90.0 {
        "极高生产力！保持这种状态"
    } else if score >= 80.0 {
        "高生产力，表现优秀"
    } else if score >= 70.0 {
        "良好的生产力水平"
    } else if score >= 60.0 {
        "中等生产力，有提升空间"
    } else if score >= 40.0 {
        "生产力偏低，建议优化时间分配"
    } else {
        "生产力较低，需要重新规划时间使用"
    }
}

async fn handle_ai_command(matches: &ArgMatches) -> Result<()> {
    let mut config_manager = match AIConfigManager::new() {
        Ok(manager) => manager,
        Err(e) => {
            println!("❌ 初始化AI配置管理器失败: {}", e);
            return Ok(());
        }
    };

    match matches.subcommand() {
        Some(("config", sub_matches)) => {
            let provider = sub_matches.get_one::<String>("provider").unwrap();
            let model = sub_matches.get_one::<String>("model");
            let api_key = sub_matches.get_one::<String>("api-key");
            let endpoint = sub_matches.get_one::<String>("endpoint");

            config_manager
                .configure_provider(provider, model, api_key, endpoint)
                .await?;
        }
        Some(("list", _)) => {
            config_manager.list_models();
        }
        Some(("show", _)) => {
            config_manager.show_config();
        }
        Some(("select", sub_matches)) => {
            let _provider = sub_matches.get_one::<String>("provider").unwrap();
            config_manager.select_model()?;
        }
        Some(("test", _sub_matches)) => {
            config_manager.test_current_config().await?;
        }
        _ => {
            println!("AI 配置管理");
            println!();
            println!("使用方法:");
            println!("  timetracker ai config -p <provider>  - 配置 AI 提供商");
            println!("  timetracker ai list                  - 列出可用提供商");
            println!("  timetracker ai show                  - 显示当前配置");
            println!("  timetracker ai select -p <provider> - 选择默认提供商");
            println!("  timetracker ai test                  - 测试配置");
        }
    }

    Ok(())
}
