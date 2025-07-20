use crate::tracker::{TimeTracker};
use crate::ai_config::{AIConfig, AIProvider};
use crate::ai_client::{UnifiedAIClient, AIRequest, AIMessage};
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct AIAnalysisRequest {
    pub activities: Vec<ActivitySummary>,
    pub sessions: Vec<SessionSummary>,
    pub time_range: TimeRange,
    pub total_time: u64,
    pub app_distribution: HashMap<String, u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ActivitySummary {
    pub app_name: String,
    pub window_title: String,
    pub duration: u64,
    pub start_time: DateTime<Utc>,
    pub category: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionSummary {
    pub app_name: String,
    pub window_title: String,
    pub total_duration: u64,
    pub activity_count: usize,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TimeRange {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AIAnalysisResult {
    pub summary: String,
    pub productivity_score: Option<f32>,
    pub insights: Vec<String>,
    pub recommendations: Vec<String>,
    pub time_distribution: HashMap<String, String>,
    pub focus_periods: Vec<FocusPeriod>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FocusPeriod {
    pub app_name: String,
    pub duration: u64,
    pub start_time: DateTime<Utc>,
    pub focus_score: f32,
}

pub struct AIAnalyzer {
    config: AIConfig,
    client: UnifiedAIClient,
}

impl AIAnalyzer {
    pub fn new() -> Result<Self> {
        let config = AIConfig::load().unwrap_or_default();
        let client = UnifiedAIClient::new(config.clone())?;

        Ok(Self {
            config,
            client,
        })
    }

    pub fn is_configured(&self) -> bool {
        // 检查当前提供商是否已配置
        self.config.is_provider_configured(&self.config.current_provider)
    }

    pub async fn analyze_usage(&self, tracker: &TimeTracker) -> Result<AIAnalysisResult> {
        if !self.is_configured() {
            return Err(anyhow::anyhow!(
                "AI 分析未配置。请使用 'timetracker ai config' 配置 AI 提供商。"
            ));
        }

        // 准备分析数据
        let analysis_request = self.prepare_analysis_data(tracker)?;
        
        // 调用 AI API
        let ai_response = self.call_ai_api(&analysis_request).await?;
        
        // 解析响应
        let analysis_result = self.parse_ai_response(&ai_response, &analysis_request)?;
        
        Ok(analysis_result)
    }

    fn prepare_analysis_data(&self, tracker: &TimeTracker) -> Result<AIAnalysisRequest> {
        let activities = tracker.get_activities();
        let sessions = tracker.get_activity_sessions();
        
        // 计算时间范围
        let time_range = if let (Some(first), Some(last)) = (
            activities.first(),
            activities.last()
        ) {
            TimeRange {
                start: first.start_time,
                end: last.end_time.unwrap_or(Utc::now()),
            }
        } else {
            TimeRange {
                start: Utc::now(),
                end: Utc::now(),
            }
        };

        // 转换活动数据
        let activity_summaries: Vec<ActivitySummary> = activities
            .iter()
            .map(|activity| ActivitySummary {
                app_name: activity.app_name.clone(),
                window_title: activity.window_title.clone(),
                duration: activity.duration,
                start_time: activity.start_time,
                category: self.categorize_app(&activity.app_name),
            })
            .collect();

        // 转换会话数据
        let session_summaries: Vec<SessionSummary> = sessions
            .iter()
            .map(|session| SessionSummary {
                app_name: session.app_name.clone(),
                window_title: session.window_title.clone(),
                total_duration: session.total_duration,
                activity_count: session.activity_count,
                start_time: session.start_time,
                end_time: session.end_time,
            })
            .collect();

        // 计算应用分布
        let app_distribution = tracker.get_statistics();

        Ok(AIAnalysisRequest {
            activities: activity_summaries,
            sessions: session_summaries,
            time_range,
            total_time: tracker.get_total_time(),
            app_distribution,
        })
    }

    fn categorize_app(&self, app_name: &str) -> Option<String> {
        let app_lower = app_name.to_lowercase();
        
        if app_lower.contains("code") || app_lower.contains("vim") || app_lower.contains("ide") {
            Some("开发工具".to_string())
        } else if app_lower.contains("browser") || app_lower.contains("chrome") || app_lower.contains("firefox") || app_lower.contains("safari") {
            Some("浏览器".to_string())
        } else if app_lower.contains("slack") || app_lower.contains("teams") || app_lower.contains("discord") {
            Some("沟通工具".to_string())
        } else if app_lower.contains("word") || app_lower.contains("excel") || app_lower.contains("powerpoint") {
            Some("办公软件".to_string())
        } else if app_lower.contains("music") || app_lower.contains("video") || app_lower.contains("player") {
            Some("娱乐".to_string())
        } else {
            Some("其他".to_string())
        }
    }

    async fn call_ai_api(&self, data: &AIAnalysisRequest) -> Result<String> {
        // 构建提示词
        let prompt = self.build_analysis_prompt(data);
        
        // 构建AI请求
        let request = AIRequest {
            messages: vec![
                AIMessage {
                    role: "system".to_string(),
                    content: "你是一个专业的时间管理和生产力分析师。请分析用户的应用使用数据，提供有价值的洞察和建议。请用中文回复，并以JSON格式返回结构化的分析结果。".to_string(),
                },
                AIMessage {
                    role: "user".to_string(),
                    content: prompt,
                }
            ],
            max_tokens: None,
            temperature: None,
            stream: Some(false),
        };

        // 使用统一客户端调用AI API
        let response = self.client.chat(request).await?;

        Ok(response.content)
    }

    fn build_analysis_prompt(&self, data: &AIAnalysisRequest) -> String {
        let total_hours = data.total_time as f64 / 3600.0;
        let app_count = data.app_distribution.len();
        
        // 获取前5个最常用的应用
        let mut sorted_apps: Vec<_> = data.app_distribution.iter().collect();
        sorted_apps.sort_by(|a, b| b.1.cmp(a.1));
        let top_apps: Vec<String> = sorted_apps
            .iter()
            .take(5)
            .map(|(name, duration)| format!("{}: {:.1}小时", name, **duration as f64 / 3600.0))
            .collect();

        format!(
            r#"请分析以下时间追踪数据：

总使用时间: {:.1} 小时
使用的应用数量: {} 个
时间范围: {} 到 {}

前5个最常用的应用:
{}

会话数据: {} 个会话
活动记录: {} 条记录

请提供以下分析（以JSON格式返回）：
1. 总体使用情况摘要
2. 生产力评分（0-100）
3. 关键洞察（数组）
4. 改进建议（数组）
5. 时间分布分析
6. 专注时段识别

JSON格式示例：
{{
  "summary": "用户主要使用...",
  "productivity_score": 75.5,
  "insights": ["洞察1", "洞察2"],
  "recommendations": ["建议1", "建议2"],
  "time_distribution": {{"工作": "60%", "娱乐": "40%"}},
  "focus_periods": []
}}"#,
            total_hours,
            app_count,
            data.time_range.start.format("%Y-%m-%d %H:%M"),
            data.time_range.end.format("%Y-%m-%d %H:%M"),
            top_apps.join("\n"),
            data.sessions.len(),
            data.activities.len()
        )
    }

    fn parse_ai_response(&self, response: &str, _data: &AIAnalysisRequest) -> Result<AIAnalysisResult> {
        // 尝试从响应中提取JSON
        let json_start = response.find('{');
        let json_end = response.rfind('}');
        
        if let (Some(start), Some(end)) = (json_start, json_end) {
            let json_str = &response[start..=end];
            match serde_json::from_str::<AIAnalysisResult>(json_str) {
                Ok(result) => Ok(result),
                Err(_) => {
                    // 如果解析失败，创建一个基本的结果
                    Ok(AIAnalysisResult {
                        summary: response.to_string(),
                        productivity_score: None,
                        insights: vec!["AI 分析完成，但响应格式需要调整".to_string()],
                        recommendations: vec!["请检查 AI API 配置".to_string()],
                        time_distribution: HashMap::new(),
                        focus_periods: vec![],
                    })
                }
            }
        } else {
            Ok(AIAnalysisResult {
                summary: response.to_string(),
                productivity_score: None,
                insights: vec![],
                recommendations: vec![],
                time_distribution: HashMap::new(),
                focus_periods: vec![],
            })
        }
    }

    // 本地分析功能（不需要 AI API）
    pub fn local_analysis(&self, tracker: &TimeTracker) -> Result<AIAnalysisResult> {
        let _activities = tracker.get_activities();
        let sessions = tracker.get_activity_sessions();
        let total_time = tracker.get_total_time();
        let app_stats = tracker.get_statistics();

        // 计算基本统计
        let total_hours = total_time as f64 / 3600.0;
        let app_count = app_stats.len();
        
        // 分析应用类别分布
        let mut category_time: HashMap<String, u64> = HashMap::new();
        for (app_name, duration) in &app_stats {
            let category = self.categorize_app(app_name).unwrap_or_else(|| "其他".to_string());
            *category_time.entry(category).or_insert(0) += duration;
        }

        // 计算时间分布百分比
        let time_distribution: HashMap<String, String> = category_time
            .iter()
            .map(|(category, duration)| {
                let percentage = (*duration as f64 / total_time as f64) * 100.0;
                (category.clone(), format!("{:.1}%", percentage))
            })
            .collect();

        // 生成洞察
        let mut insights = vec![];
        let mut recommendations = vec![];

        if total_hours > 8.0 {
            insights.push(format!("今日使用电脑时间较长：{:.1}小时", total_hours));
            recommendations.push("建议适当休息，保护视力健康".to_string());
        }

        if app_count > 10 {
            insights.push(format!("使用了{}个不同的应用，任务切换较频繁", app_count));
            recommendations.push("尝试减少应用切换，提高专注度".to_string());
        }

        // 找出最长的专注会话
        let mut focus_periods = vec![];
        for session in &sessions {
            if session.total_duration > 1800 { // 超过30分钟
                focus_periods.push(FocusPeriod {
                    app_name: session.app_name.clone(),
                    duration: session.total_duration,
                    start_time: session.start_time,
                    focus_score: (session.total_duration as f32 / 3600.0).min(10.0), // 最高10分
                });
            }
        }

        // 计算简单的生产力评分
        let productivity_score = self.calculate_productivity_score(&category_time, total_time);

        let summary = format!(
            "今日共使用电脑{:.1}小时，涉及{}个应用。主要时间分配：{}",
            total_hours,
            app_count,
            time_distribution.iter()
                .take(3)
                .map(|(k, v)| format!("{}: {}", k, v))
                .collect::<Vec<_>>()
                .join(", ")
        );

        Ok(AIAnalysisResult {
            summary,
            productivity_score: Some(productivity_score),
            insights,
            recommendations,
            time_distribution,
            focus_periods,
        })
    }

    fn calculate_productivity_score(&self, category_time: &HashMap<String, u64>, total_time: u64) -> f32 {
        let productive_categories = ["开发工具", "办公软件"];
        let neutral_categories = ["浏览器", "沟通工具"];
        
        let mut productive_time = 0u64;
        let mut neutral_time = 0u64;
        let mut entertainment_time = 0u64;

        for (category, duration) in category_time {
            if productive_categories.contains(&category.as_str()) {
                productive_time += duration;
            } else if neutral_categories.contains(&category.as_str()) {
                neutral_time += duration;
            } else if category == "娱乐" {
                entertainment_time += duration;
            }
        }

        if total_time == 0 {
            return 0.0;
        }

        let productive_ratio = productive_time as f32 / total_time as f32;
        let neutral_ratio = neutral_time as f32 / total_time as f32;
        let entertainment_ratio = entertainment_time as f32 / total_time as f32;

        // 计算评分：生产力时间权重最高，中性时间权重中等，娱乐时间扣分
        let score = (productive_ratio * 100.0) + (neutral_ratio * 50.0) - (entertainment_ratio * 20.0);
        score.max(0.0).min(100.0)
    }
}

impl Default for AIAnalyzer {
    fn default() -> Self {
        let config = AIConfig::load().unwrap_or_default();
        let client = UnifiedAIClient::new(config.clone()).unwrap_or_else(|_| {
            // 如果创建失败，使用默认配置
            UnifiedAIClient::new(AIConfig::default()).unwrap()
        });
        
        Self { config, client }
    }
}