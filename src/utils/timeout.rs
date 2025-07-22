use anyhow::Result;
use std::future::Future;
use std::time::Duration;

/// 超时配置
#[derive(Debug, Clone)]
pub struct TimeoutConfig {
    pub permission_check: Duration,
    pub daemon_start: Duration,
    pub daemon_stop: Duration,
    pub monitor_init: Duration,
    pub system_info: Duration,
}

impl Default for TimeoutConfig {
    fn default() -> Self {
        Self {
            permission_check: Duration::from_secs(5),
            daemon_start: Duration::from_secs(10),
            daemon_stop: Duration::from_secs(5),
            monitor_init: Duration::from_secs(15),
            system_info: Duration::from_secs(3),
        }
    }
}

/// 带超时的异步操作执行器
pub async fn with_timeout<F, T>(future: F, timeout: Duration, operation_name: &str) -> Result<T>
where
    F: Future<Output = Result<T>>,
{
    match tokio::time::timeout(timeout, future).await {
        Ok(result) => result,
        Err(_) => {
            let error_msg = format!("操作 '{}' 超时 ({:?})", operation_name, timeout);
            log::error!("{}", error_msg);
            Err(anyhow::anyhow!(error_msg))
        }
    }
}

/// 带超时的同步操作执行器
pub fn with_sync_timeout<F, T>(operation: F, timeout: Duration, operation_name: &str) -> Result<T>
where
    F: FnOnce() -> Result<T> + Send + 'static,
    T: Send + 'static,
{
    use std::sync::mpsc;
    use std::thread;

    let (tx, rx) = mpsc::channel();

    // 在单独线程中执行操作
    let handle = thread::spawn(move || {
        let result = operation();
        let _ = tx.send(result);
    });

    // 等待结果或超时
    match rx.recv_timeout(timeout) {
        Ok(result) => {
            // 确保线程正常结束
            let _ = handle.join();
            result
        }
        Err(mpsc::RecvTimeoutError::Timeout) => {
            let error_msg = format!("同步操作 '{}' 超时 ({:?})", operation_name, timeout);
            log::error!("{}", error_msg);
            // 注意：线程可能仍在运行，但我们无法强制终止它
            Err(anyhow::anyhow!(error_msg))
        }
        Err(mpsc::RecvTimeoutError::Disconnected) => {
            let error_msg = format!("同步操作 '{}' 线程意外终止", operation_name);
            log::error!("{}", error_msg);
            Err(anyhow::anyhow!(error_msg))
        }
    }
}

/// 重试配置
#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub initial_delay: Duration,
    pub max_delay: Duration,
    pub backoff_multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(5),
            backoff_multiplier: 2.0,
        }
    }
}

/// 带重试的操作执行器
pub async fn with_retry<F, Fut, T>(
    mut operation: F,
    config: RetryConfig,
    operation_name: &str,
) -> Result<T>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T>>,
{
    let mut delay = config.initial_delay;
    let mut last_error = None;

    for attempt in 1..=config.max_attempts {
        match operation().await {
            Ok(result) => {
                if attempt > 1 {
                    log::info!("操作 '{}' 在第 {} 次尝试后成功", operation_name, attempt);
                }
                return Ok(result);
            }
            Err(e) => {
                last_error = Some(e);

                if attempt < config.max_attempts {
                    log::warn!(
                        "操作 '{}' 第 {} 次尝试失败，{:?} 后重试: {}",
                        operation_name,
                        attempt,
                        delay,
                        last_error.as_ref().unwrap()
                    );

                    tokio::time::sleep(delay).await;

                    // 指数退避
                    delay = std::cmp::min(
                        Duration::from_millis(
                            (delay.as_millis() as f64 * config.backoff_multiplier) as u64,
                        ),
                        config.max_delay,
                    );
                } else {
                    log::error!(
                        "操作 '{}' 在 {} 次尝试后仍然失败",
                        operation_name,
                        config.max_attempts
                    );
                }
            }
        }
    }

    Err(last_error.unwrap_or_else(|| anyhow::anyhow!("未知错误")))
}

/// 优雅的错误处理器
pub fn handle_error(error: &anyhow::Error, operation: &str) -> String {
    let error_msg = format!("操作 '{}' 失败: {}", operation, error);

    // 根据错误类型提供不同的建议
    let suggestion = if error.to_string().contains("permission")
        || error.to_string().contains("权限")
    {
        "建议运行 'timetracker permissions request' 来获取必要权限"
    } else if error.to_string().contains("timeout") || error.to_string().contains("超时") {
        "操作超时，请检查系统负载或网络连接"
    } else if error.to_string().contains("not found") || error.to_string().contains("找不到") {
        "请确保所有依赖项已正确安装"
    } else {
        "请查看日志文件获取详细信息"
    };

    log::error!("{}", error_msg);
    log::info!("建议: {}", suggestion);

    format!("{}\n建议: {}", error_msg, suggestion)
}

/// 安全的资源清理器
pub struct ResourceGuard<F>
where
    F: FnOnce(),
{
    cleanup: Option<F>,
}

impl<F> ResourceGuard<F>
where
    F: FnOnce(),
{
    pub fn new(cleanup: F) -> Self {
        Self {
            cleanup: Some(cleanup),
        }
    }

    pub fn disarm(mut self) {
        self.cleanup.take();
    }
}

impl<F> Drop for ResourceGuard<F>
where
    F: FnOnce(),
{
    fn drop(&mut self) {
        if let Some(cleanup) = self.cleanup.take() {
            cleanup();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    #[tokio::test]
    async fn test_timeout_success() {
        let result = with_timeout(
            async { Ok::<i32, anyhow::Error>(42) },
            Duration::from_secs(1),
            "test_operation",
        )
        .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_timeout_failure() {
        let result = with_timeout(
            async {
                tokio::time::sleep(Duration::from_secs(2)).await;
                Ok::<i32, anyhow::Error>(42)
            },
            Duration::from_millis(100),
            "test_operation",
        )
        .await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("超时"));
    }

    #[tokio::test]
    async fn test_retry_success_after_failure() {
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        let result = with_retry(
            move || {
                let count = counter_clone.fetch_add(1, Ordering::SeqCst);
                async move {
                    if count < 2 {
                        Err(anyhow::anyhow!("Simulated failure"))
                    } else {
                        Ok(42)
                    }
                }
            },
            RetryConfig::default(),
            "test_retry",
        )
        .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
        assert_eq!(counter.load(Ordering::SeqCst), 3);
    }
}
