use anyhow::Result;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use sysinfo::{Pid, System};

const PID_FILE: &str = "/tmp/timetracker.pid";
const LOG_FILE: &str = "/tmp/timetracker.log";

pub struct DaemonManager {
    pid_file: PathBuf,
    log_file: PathBuf,
    should_cleanup: bool,
}

impl DaemonManager {
    pub fn new() -> Self {
        Self {
            pid_file: PathBuf::from(PID_FILE),
            log_file: PathBuf::from(LOG_FILE),
            should_cleanup: true,
        }
    }

    pub fn is_running(&self) -> bool {
        if let Ok(pid_str) = fs::read_to_string(&self.pid_file) {
            if let Ok(pid) = pid_str.trim().parse::<u32>() {
                let mut system = System::new_all();
                system.refresh_all();
                return system.process(Pid::from_u32(pid)).is_some();
            }
        }
        false
    }

    pub fn get_pid(&self) -> Option<u32> {
        if let Ok(pid_str) = fs::read_to_string(&self.pid_file) {
            if let Ok(pid) = pid_str.trim().parse::<u32>() {
                let mut system = System::new_all();
                system.refresh_all();
                if system.process(Pid::from_u32(pid)).is_some() {
                    return Some(pid);
                }
            }
        }
        None
    }

    pub fn start_daemon(&mut self, interval: u64, data_file: &str) -> Result<()> {
        if self.is_running() {
            return Err(anyhow::anyhow!("TimeTracker 守护进程已在运行"));
        }

        // 获取当前可执行文件路径
        let current_exe = std::env::current_exe()?;

        // 启动守护进程，使用 --daemon-child 标志来避免无限递归
        let child = Command::new(&current_exe)
            .args([
                "start",
                "--interval",
                &interval.to_string(),
                "--data-file",
                data_file,
                "--daemon-child",
            ])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .stdin(Stdio::null())
            .spawn()?;

        let child_pid = child.id();

        println!("TimeTracker 守护进程已启动 (PID: {})", child_pid);
        println!("日志文件: {}", self.log_file.display());
        println!("使用 'timetracker stop' 停止守护进程");

        // 分离子进程，让它独立运行
        std::mem::forget(child);

        // 设置不要在Drop时清理PID文件
        self.should_cleanup = false;

        Ok(())
    }

    pub fn stop_daemon(&self) -> Result<()> {
        if let Some(pid) = self.get_pid() {
            // 发送 SIGTERM 信号
            #[cfg(unix)]
            {
                use nix::sys::signal::{self, Signal};
                use nix::unistd::Pid as NixPid;

                let nix_pid = NixPid::from_raw(pid as i32);
                signal::kill(nix_pid, Signal::SIGTERM)?;
            }

            #[cfg(windows)]
            {
                Command::new("taskkill")
                    .args(["/PID", &pid.to_string(), "/F"])
                    .output()?;
            }

            // 清理 PID 文件
            if self.pid_file.exists() {
                fs::remove_file(&self.pid_file)?;
            }

            println!("TimeTracker 守护进程已停止 (PID: {})", pid);
        } else {
            println!("TimeTracker 守护进程未运行");
        }

        Ok(())
    }

    pub fn status(&self) -> Result<()> {
        if let Some(pid) = self.get_pid() {
            let mut system = System::new_all();
            system.refresh_all();
            if let Some(process) = system.process(Pid::from_u32(pid)) {
                println!("TimeTracker 守护进程状态:");
                println!("  PID: {}", pid);
                println!("  状态: 运行中");
                println!("  CPU 使用率: {:.1}%", process.cpu_usage());
                println!("  内存使用: {} KB", process.memory());
                println!("  启动时间: {}", process.start_time());
                println!("  日志文件: {}", self.log_file.display());

                // 显示最近的日志
                if self.log_file.exists() {
                    if let Ok(log_content) = fs::read_to_string(&self.log_file) {
                        let lines: Vec<&str> = log_content.lines().collect();
                        let recent_lines = lines.iter().rev().take(5).rev();

                        println!("\n最近日志:");
                        for line in recent_lines {
                            println!("  {}", line);
                        }
                    }
                }
            } else {
                println!("TimeTracker 守护进程 PID 文件存在但进程未运行");
                // 清理无效的 PID 文件
                if self.pid_file.exists() {
                    fs::remove_file(&self.pid_file)?;
                }
            }
        } else {
            println!("TimeTracker 守护进程未运行");
        }

        Ok(())
    }

    pub fn restart_daemon(&mut self, interval: u64, data_file: &str) -> Result<()> {
        println!("重启 TimeTracker 守护进程...");

        // 停止现有守护进程
        if self.is_running() {
            self.stop_daemon()?;

            // 等待进程完全停止
            std::thread::sleep(std::time::Duration::from_secs(1));
        }

        // 启动新的守护进程
        self.start_daemon(interval, data_file)?;

        Ok(())
    }

    #[allow(dead_code)]
    pub fn get_log_file(&self) -> &PathBuf {
        &self.log_file
    }

    pub fn cleanup(&self) -> Result<()> {
        // 清理 PID 文件
        if self.pid_file.exists() {
            fs::remove_file(&self.pid_file)?;
        }

        // 可选：清理日志文件
        // if self.log_file.exists() {
        //     fs::remove_file(&self.log_file)?;
        // }

        Ok(())
    }
}

impl Drop for DaemonManager {
    fn drop(&mut self) {
        // 只有在should_cleanup为true时才清理资源
        if self.should_cleanup {
            let _ = self.cleanup();
        }
    }
}

#[cfg(unix)]
pub fn setup_signal_handlers() -> Result<()> {
    use nix::sys::signal::{self, SigHandler, Signal};

    extern "C" fn handle_sigterm(_: i32) {
        eprintln!("收到 SIGTERM 信号，正在优雅关闭...");
        std::process::exit(0);
    }

    extern "C" fn handle_sigint(_: i32) {
        eprintln!("收到 SIGINT 信号，正在优雅关闭...");
        std::process::exit(0);
    }

    unsafe {
        signal::signal(Signal::SIGTERM, SigHandler::Handler(handle_sigterm))?;
        signal::signal(Signal::SIGINT, SigHandler::Handler(handle_sigint))?;
    }

    Ok(())
}

#[cfg(windows)]
pub fn setup_signal_handlers() -> Result<()> {
    // Windows 信号处理实现
    Ok(())
}

#[allow(dead_code)]
pub fn daemonize() -> Result<()> {
    #[cfg(unix)]
    {
        use daemonize::Daemonize;

        let daemonize = Daemonize::new()
            .pid_file(PID_FILE)
            .chown_pid_file(true)
            .working_directory("/tmp")
            .user("nobody")
            .group("daemon")
            .umask(0o027);

        match daemonize.start() {
            Ok(_) => {
                setup_signal_handlers()?;
                Ok(())
            }
            Err(e) => Err(anyhow::anyhow!("守护进程启动失败: {}", e)),
        }
    }

    #[cfg(windows)]
    {
        // Windows 下的守护进程实现
        setup_signal_handlers()?;
        Ok(())
    }
}
