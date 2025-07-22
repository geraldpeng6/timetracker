// 时间相关的工具函数
// 采用函数式编程风格

use chrono::{DateTime, Datelike, Duration as ChronoDuration, Utc};

/// 格式化持续时间为可读字符串
///
/// # 示例
/// ```
/// use timetracker::utils::time::format_duration;
///
/// assert_eq!(format_duration(3661), "1h 1m 1s");
/// assert_eq!(format_duration(61), "1m 1s");
/// assert_eq!(format_duration(30), "30s");
/// ```
pub fn format_duration(duration: u64) -> String {
    let hours = duration / 3600;
    let minutes = (duration % 3600) / 60;
    let seconds = duration % 60;

    match (hours, minutes, seconds) {
        (0, 0, s) => format!("{s}s"),
        (0, m, s) => format!("{m}m {s}s"),
        (h, m, s) => format!("{h}h {m}m {s}s"),
    }
}

/// 格式化持续时间为简短字符串
pub fn format_duration_short(duration: u64) -> String {
    let hours = duration / 3600;
    let minutes = (duration % 3600) / 60;

    match (hours, minutes) {
        (0, m) if m < 1 => "<1m".to_string(),
        (0, m) => format!("{m}m"),
        (h, m) => format!("{h}h{m}m"),
    }
}

/// 计算两个时间点之间的持续时间（秒）
pub fn duration_between(start: DateTime<Utc>, end: DateTime<Utc>) -> u64 {
    (end - start).num_seconds().max(0) as u64
}

/// 获取今天的开始时间
pub fn today_start() -> DateTime<Utc> {
    let now = Utc::now();
    now.date_naive()
        .and_hms_opt(0, 0, 0)
        .map(|dt| dt.and_utc())
        .unwrap_or(now)
}

/// 获取本周的开始时间（周一）
pub fn week_start() -> DateTime<Utc> {
    let now = Utc::now();
    let days_since_monday = now.weekday().num_days_from_monday();
    now - ChronoDuration::days(days_since_monday as i64)
}

/// 获取本月的开始时间
pub fn month_start() -> DateTime<Utc> {
    let now = Utc::now();
    now.date_naive()
        .with_day(1)
        .and_then(|date| date.and_hms_opt(0, 0, 0))
        .map(|dt| dt.and_utc())
        .unwrap_or(now)
}

/// 时间范围枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeRange {
    Today,
    Yesterday,
    ThisWeek,
    LastWeek,
    ThisMonth,
    LastMonth,
    Custom(DateTime<Utc>, DateTime<Utc>),
}

impl TimeRange {
    /// 获取时间范围的开始和结束时间
    pub fn bounds(self) -> (DateTime<Utc>, DateTime<Utc>) {
        let now = Utc::now();

        match self {
            TimeRange::Today => {
                let start = today_start();
                (start, now)
            }
            TimeRange::Yesterday => {
                let today = today_start();
                let yesterday = today - ChronoDuration::days(1);
                (yesterday, today)
            }
            TimeRange::ThisWeek => {
                let start = week_start();
                (start, now)
            }
            TimeRange::LastWeek => {
                let this_week = week_start();
                let last_week = this_week - ChronoDuration::days(7);
                (last_week, this_week)
            }
            TimeRange::ThisMonth => {
                let start = month_start();
                (start, now)
            }
            TimeRange::LastMonth => {
                let this_month = month_start();
                let last_month = this_month - ChronoDuration::days(1);
                let last_month_start = last_month
                    .date_naive()
                    .with_day(1)
                    .and_then(|date| date.and_hms_opt(0, 0, 0))
                    .map(|dt| dt.and_utc())
                    .unwrap_or(last_month);
                (last_month_start, this_month)
            }
            TimeRange::Custom(start, end) => (start, end),
        }
    }

    /// 检查给定时间是否在范围内
    pub fn contains(self, time: DateTime<Utc>) -> bool {
        let (start, end) = self.bounds();
        time >= start && time <= end
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(0), "0s");
        assert_eq!(format_duration(30), "30s");
        assert_eq!(format_duration(60), "1m 0s");
        assert_eq!(format_duration(61), "1m 1s");
        assert_eq!(format_duration(3600), "1h 0m 0s");
        assert_eq!(format_duration(3661), "1h 1m 1s");
    }

    #[test]
    fn test_format_duration_short() {
        assert_eq!(format_duration_short(30), "<1m");
        assert_eq!(format_duration_short(60), "1m");
        assert_eq!(format_duration_short(3600), "1h0m");
        assert_eq!(format_duration_short(3661), "1h1m");
    }
}
