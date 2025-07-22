// 验证相关的工具函数
// 采用函数式编程风格

use anyhow::{anyhow, Result};

/// 验证监控间隔
///
/// # 参数
/// - `interval`: 间隔秒数
///
/// # 返回
/// - `Ok(interval)`: 验证通过，返回有效的间隔
/// - `Err`: 验证失败
pub fn validate_interval(interval: u64) -> Result<u64> {
    match interval {
        0 => Err(anyhow!("监控间隔不能为0")),
        i if i > 3600 => Err(anyhow!("监控间隔不能超过1小时(3600秒)")),
        i => Ok(i.max(1)), // 最小值为1秒
    }
}

/// 验证文件路径
pub fn validate_file_path(path: &str) -> Result<String> {
    if path.is_empty() {
        return Err(anyhow!("文件路径不能为空"));
    }

    // 检查路径是否包含非法字符
    let invalid_chars = ['<', '>', ':', '"', '|', '?', '*'];
    if path.chars().any(|c| invalid_chars.contains(&c)) {
        return Err(anyhow!("文件路径包含非法字符"));
    }

    Ok(path.to_string())
}

/// 验证 API 密钥格式
pub fn validate_api_key(key: &str) -> Result<String> {
    if key.is_empty() {
        return Err(anyhow!("API密钥不能为空"));
    }

    if key.len() < 10 {
        return Err(anyhow!("API密钥长度过短"));
    }

    // 检查是否包含空格
    if key.contains(' ') {
        return Err(anyhow!("API密钥不能包含空格"));
    }

    Ok(key.to_string())
}

/// 验证模型名称
pub fn validate_model_name(name: &str) -> Result<String> {
    if name.is_empty() {
        return Err(anyhow!("模型名称不能为空"));
    }

    // 检查模型名称格式
    if !name
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.')
    {
        return Err(anyhow!("模型名称只能包含字母、数字、连字符、下划线和点"));
    }

    Ok(name.to_string())
}

/// 验证 URL 格式
pub fn validate_url(url: &str) -> Result<String> {
    if url.is_empty() {
        return Err(anyhow!("URL不能为空"));
    }

    // 简单的 URL 格式检查
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err(anyhow!("URL必须以http://或https://开头"));
    }

    Ok(url.to_string())
}

/// 验证温度参数
pub fn validate_temperature(temp: f32) -> Result<f32> {
    if !(0.0..=2.0).contains(&temp) {
        return Err(anyhow!("温度参数必须在0.0到2.0之间"));
    }
    Ok(temp)
}

/// 验证最大 token 数
pub fn validate_max_tokens(tokens: u32) -> Result<u32> {
    if tokens == 0 {
        return Err(anyhow!("最大token数不能为0"));
    }

    if tokens > 100000 {
        return Err(anyhow!("最大token数不能超过100000"));
    }

    Ok(tokens)
}

/// 组合验证器 - 函数式编程风格
pub fn compose_validators<T, E>(
    validators: Vec<Box<dyn Fn(T) -> Result<T, E>>>,
) -> impl Fn(T) -> Result<T, E>
where
    T: Clone,
    E: std::fmt::Debug,
{
    move |input: T| -> Result<T, E> {
        validators
            .iter()
            .try_fold(input, |acc, validator| validator(acc))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_interval() {
        assert!(validate_interval(0).is_err());
        assert!(validate_interval(3601).is_err());
        assert_eq!(validate_interval(5).unwrap(), 5);
        assert_eq!(validate_interval(0).unwrap_or(1), 1);
    }

    #[test]
    fn test_validate_api_key() {
        assert!(validate_api_key("").is_err());
        assert!(validate_api_key("short").is_err());
        assert!(validate_api_key("key with space").is_err());
        assert!(validate_api_key("valid-api-key-123").is_ok());
    }

    #[test]
    fn test_validate_url() {
        assert!(validate_url("").is_err());
        assert!(validate_url("invalid-url").is_err());
        assert!(validate_url("http://example.com").is_ok());
        assert!(validate_url("https://api.example.com/v1").is_ok());
    }
}
