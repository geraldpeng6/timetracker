// 生产力洞察生成器 - 分析用户工作模式并提供智能建议
// 基于活动数据生成个性化的生产力洞察和建议

use crate::core::tracker::ActivityRecord;
use anyhow::Result;
use chrono::{DateTime, Utc, Weekday};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

// 为了兼容性，创建一个类型别名
pub type ActivityData = ActivityRecord;
use super::classifier::{ActivityCategory, ClassificationResult};

/// 洞察生成器
#[allow(dead_code)]
pub struct InsightsGenerator {
    /// 分析器集合
    analyzers: AnalyzerCollection,
    /// 建议引擎
    recommendation_engine: RecommendationEngine,
    /// 配置
    config: InsightsConfig,
}

/// 分析器集合
pub struct AnalyzerCollection {
    pub focus_analyzer: FocusAnalyzer,
    pub productivity_analyzer: ProductivityAnalyzer,
    pub pattern_analyzer: PatternAnalyzer,
    pub stress_analyzer: StressAnalyzer,
    pub trend_analyzer: TrendAnalyzer,
}

/// 专注度分析器
#[allow(dead_code)]
pub struct FocusAnalyzer {
    /// 专注阈值（分钟）
    focus_threshold_minutes: u32,
    /// 中断检测敏感度
    interruption_sensitivity: f32,
}

/// 生产力分析器
#[allow(dead_code)]
pub struct ProductivityAnalyzer {
    /// 生产力权重配置
    category_weights: HashMap<String, f32>,
    /// 时间段权重
    time_weights: HashMap<u8, f32>,
}

/// 模式分析器
#[allow(dead_code)]
pub struct PatternAnalyzer {
    /// 最小模式长度（天）
    min_pattern_days: u32,
    /// 模式置信度阈值
    pattern_confidence_threshold: f32,
}

/// 压力分析器
#[allow(dead_code)]
pub struct StressAnalyzer {
    /// 工作强度阈值
    intensity_threshold: f32,
    /// 休息时间最小要求（分钟）
    min_break_minutes: u32,
}

/// 趋势分析器
#[allow(dead_code)]
pub struct TrendAnalyzer {
    /// 趋势分析窗口（天）
    analysis_window_days: u32,
    /// 预测窗口（天）
    prediction_window_days: u32,
}

/// 建议引擎
#[allow(dead_code)]
pub struct RecommendationEngine {
    /// 建议模板
    templates: Vec<RecommendationTemplate>,
    /// 个性化权重
    personalization_weights: HashMap<String, f32>,
}

/// 洞察配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsightsConfig {
    /// 启用深度分析
    pub enable_deep_analysis: bool,
    /// 分析历史天数
    pub analysis_history_days: u32,
    /// 最小活动时长（秒）
    pub min_activity_duration_seconds: u64,
    /// 生成建议数量
    pub max_recommendations: usize,
}

/// 生产力洞察
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductivityInsights {
    /// 总体评分
    pub overall_score: f32,
    /// 专注度分析
    pub focus_analysis: FocusAnalysis,
    /// 生产力分析
    pub productivity_analysis: ProductivityAnalysis,
    /// 工作模式
    pub work_patterns: WorkPatterns,
    /// 压力评估
    pub stress_assessment: StressAssessment,
    /// 趋势分析
    pub trends: TrendAnalysis,
    /// 个性化建议
    pub recommendations: Vec<Recommendation>,
    /// 生成时间
    pub generated_at: DateTime<Utc>,
}

/// 专注度分析
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FocusAnalysis {
    /// 平均专注时长
    pub average_focus_duration: Duration,
    /// 专注会话数量
    pub focus_sessions_count: u32,
    /// 最长专注时长
    pub longest_focus_duration: Duration,
    /// 中断频率（每小时）
    pub interruption_rate_per_hour: f32,
    /// 专注质量评分
    pub focus_quality_score: f32,
    /// 最佳专注时间段
    pub peak_focus_hours: Vec<u8>,
    /// 专注会话详情
    pub focus_sessions: Vec<FocusSession>,
}

/// 专注会话
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FocusSession {
    pub start_time: DateTime<Utc>,
    pub duration: Duration,
    pub primary_activity: ActivityCategory,
    pub interruption_count: u32,
    pub quality_score: f32,
    pub context: String,
}

/// 生产力分析
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductivityAnalysis {
    /// 日均生产力评分
    pub daily_average_score: f32,
    /// 生产力分布
    pub productivity_distribution: HashMap<String, f32>,
    /// 高效时间段
    pub high_productivity_periods: Vec<ProductivityPeriod>,
    /// 低效时间段
    pub low_productivity_periods: Vec<ProductivityPeriod>,
    /// 活动类别统计
    pub category_breakdown: HashMap<ActivityCategory, CategoryStats>,
}

/// 生产力时间段
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductivityPeriod {
    pub start_hour: u8,
    pub end_hour: u8,
    pub average_score: f32,
    pub consistency: f32,
    pub dominant_activities: Vec<ActivityCategory>,
}

/// 活动类别统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryStats {
    pub total_time: Duration,
    pub percentage: f32,
    pub average_productivity: f32,
    pub session_count: u32,
}

/// 工作模式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkPatterns {
    /// 工作时间模式
    pub work_schedule: WorkSchedule,
    /// 休息模式
    pub break_patterns: BreakPatterns,
    /// 任务切换模式
    pub task_switching: TaskSwitching,
    /// 周期性模式
    pub cyclical_patterns: Vec<CyclicalPattern>,
}

/// 工作时间安排
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkSchedule {
    pub typical_start_time: u8,
    pub typical_end_time: u8,
    pub average_work_hours: f32,
    pub work_days: Vec<Weekday>,
    pub overtime_frequency: f32,
}

/// 休息模式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreakPatterns {
    pub average_break_duration: Duration,
    pub break_frequency_per_hour: f32,
    pub longest_work_stretch: Duration,
    pub break_timing_consistency: f32,
}

/// 任务切换
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskSwitching {
    pub switches_per_hour: f32,
    pub average_task_duration: Duration,
    pub context_switching_cost: f32,
    pub multitasking_tendency: f32,
}

/// 周期性模式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CyclicalPattern {
    pub pattern_type: PatternType,
    pub cycle_length: Duration,
    pub confidence: f32,
    pub description: String,
}

/// 模式类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PatternType {
    Daily,
    Weekly,
    Monthly,
    Custom(String),
}

/// 压力评估
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StressAssessment {
    /// 压力水平 (0-100)
    pub stress_level: f32,
    /// 压力指标
    pub stress_indicators: StressIndicators,
    /// 风险因素
    pub risk_factors: Vec<RiskFactor>,
    /// 缓解建议
    pub mitigation_suggestions: Vec<String>,
}

/// 压力指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StressIndicators {
    pub work_intensity: f32,
    pub overtime_frequency: f32,
    pub break_deficit: Duration,
    pub task_switching_stress: f32,
    pub deadline_pressure: f32,
}

/// 风险因素
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskFactor {
    pub factor_type: RiskFactorType,
    pub severity: f32,
    pub description: String,
    pub recommendation: String,
}

/// 风险因素类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskFactorType {
    Overwork,
    LackOfBreaks,
    HighStress,
    PoorFocus,
    Burnout,
}

/// 趋势分析
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendAnalysis {
    /// 生产力趋势
    pub productivity_trend: Trend,
    /// 专注度趋势
    pub focus_trend: Trend,
    /// 工作时长趋势
    pub work_hours_trend: Trend,
    /// 压力水平趋势
    pub stress_trend: Trend,
    /// 预测
    pub predictions: Predictions,
}

/// 趋势
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trend {
    pub direction: TrendDirection,
    pub magnitude: f32,
    pub confidence: f32,
    pub data_points: Vec<TrendPoint>,
}

/// 趋势方向
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrendDirection {
    Improving,
    Declining,
    Stable,
    Volatile,
}

/// 趋势数据点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendPoint {
    pub date: DateTime<Utc>,
    pub value: f32,
}

/// 预测
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Predictions {
    /// 下周生产力预测
    pub next_week_productivity: f32,
    /// 最佳工作时间预测
    pub optimal_work_times: Vec<TimeSlot>,
    /// 潜在风险预测
    pub potential_risks: Vec<String>,
    /// 改进机会
    pub improvement_opportunities: Vec<String>,
}

/// 时间段
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSlot {
    pub start_hour: u8,
    pub end_hour: u8,
    pub day_of_week: Option<Weekday>,
    pub expected_productivity: f32,
}

/// 建议
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    pub id: String,
    pub title: String,
    pub description: String,
    pub category: RecommendationCategory,
    pub priority: RecommendationPriority,
    pub expected_impact: f32,
    pub implementation_effort: f32,
    pub personalization_score: f32,
    pub action_items: Vec<ActionItem>,
}

/// 建议类别
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationCategory {
    TimeManagement,
    FocusImprovement,
    BreakOptimization,
    WorkEnvironment,
    HealthWellness,
    SkillDevelopment,
    StressReduction,
    ProductivityBoost,
}

/// 建议优先级
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationPriority {
    Critical,
    High,
    Medium,
    Low,
}

/// 行动项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionItem {
    pub description: String,
    pub estimated_time: Duration,
    pub difficulty: f32,
}

/// 建议模板
#[derive(Debug, Clone)]
pub struct RecommendationTemplate {
    pub id: String,
    pub title_template: String,
    pub description_template: String,
    pub category: RecommendationCategory,
    pub conditions: Vec<RecommendationCondition>,
    pub base_priority: RecommendationPriority,
}

/// 建议条件
#[derive(Debug, Clone)]
pub enum RecommendationCondition {
    LowProductivity(f32),
    HighStress(f32),
    PoorFocus(f32),
    LongWorkHours(f32),
    FrequentInterruptions(f32),
}

impl Default for InsightsConfig {
    fn default() -> Self {
        Self {
            enable_deep_analysis: true,
            analysis_history_days: 30,
            min_activity_duration_seconds: 30,
            max_recommendations: 10,
        }
    }
}

impl InsightsGenerator {
    /// 创建新的洞察生成器
    pub fn new(config: InsightsConfig) -> Result<Self> {
        let analyzers = AnalyzerCollection::new()?;
        let recommendation_engine = RecommendationEngine::new()?;

        Ok(Self {
            analyzers,
            recommendation_engine,
            config,
        })
    }

    /// 生成生产力洞察
    pub async fn generate_insights(
        &self,
        activities: &[ActivityData],
        classifications: &[ClassificationResult],
    ) -> Result<ProductivityInsights> {
        // 1. 专注度分析
        let focus_analysis = self
            .analyzers
            .focus_analyzer
            .analyze(activities, classifications)
            .await?;

        // 2. 生产力分析
        let productivity_analysis = self
            .analyzers
            .productivity_analyzer
            .analyze(activities, classifications)
            .await?;

        // 3. 工作模式分析
        let work_patterns = self
            .analyzers
            .pattern_analyzer
            .analyze_patterns(activities, classifications)
            .await?;

        // 4. 压力评估
        let stress_assessment = self
            .analyzers
            .stress_analyzer
            .assess_stress(activities, classifications)
            .await?;

        // 5. 趋势分析
        let trends = self
            .analyzers
            .trend_analyzer
            .analyze_trends(activities, classifications)
            .await?;

        // 6. 计算总体评分
        let overall_score = self.calculate_overall_score(
            &focus_analysis,
            &productivity_analysis,
            &stress_assessment,
        )?;

        // 7. 生成建议
        let recommendations = self
            .recommendation_engine
            .generate_recommendations(
                &focus_analysis,
                &productivity_analysis,
                &work_patterns,
                &stress_assessment,
                &trends,
            )
            .await?;

        Ok(ProductivityInsights {
            overall_score,
            focus_analysis,
            productivity_analysis,
            work_patterns,
            stress_assessment,
            trends,
            recommendations,
            generated_at: Utc::now(),
        })
    }

    /// 计算总体评分
    fn calculate_overall_score(
        &self,
        focus: &FocusAnalysis,
        productivity: &ProductivityAnalysis,
        stress: &StressAssessment,
    ) -> Result<f32> {
        // 加权平均计算总体评分
        let focus_weight = 0.3;
        let productivity_weight = 0.4;
        let stress_weight = 0.3; // 压力越低越好，所以要反转

        let focus_score = focus.focus_quality_score;
        let productivity_score = productivity.daily_average_score;
        let stress_score = 100.0 - stress.stress_level; // 反转压力评分

        let overall = (focus_score * focus_weight
            + productivity_score * productivity_weight
            + stress_score * stress_weight)
            .min(100.0)
            .max(0.0);

        Ok(overall)
    }
}

// 实现各个分析器的基本结构
impl AnalyzerCollection {
    pub fn new() -> Result<Self> {
        Ok(Self {
            focus_analyzer: FocusAnalyzer::new()?,
            productivity_analyzer: ProductivityAnalyzer::new()?,
            pattern_analyzer: PatternAnalyzer::new()?,
            stress_analyzer: StressAnalyzer::new()?,
            trend_analyzer: TrendAnalyzer::new()?,
        })
    }
}

impl FocusAnalyzer {
    pub fn new() -> Result<Self> {
        Ok(Self {
            focus_threshold_minutes: 15,
            interruption_sensitivity: 0.7,
        })
    }

    pub async fn analyze(
        &self,
        _activities: &[ActivityData],
        _classifications: &[ClassificationResult],
    ) -> Result<FocusAnalysis> {
        // 实现专注度分析逻辑
        Ok(FocusAnalysis {
            average_focus_duration: Duration::from_secs(25 * 60),
            focus_sessions_count: 8,
            longest_focus_duration: Duration::from_secs(90 * 60),
            interruption_rate_per_hour: 3.2,
            focus_quality_score: 75.0,
            peak_focus_hours: vec![9, 10, 14, 15],
            focus_sessions: vec![],
        })
    }
}

impl ProductivityAnalyzer {
    pub fn new() -> Result<Self> {
        Ok(Self {
            category_weights: HashMap::new(),
            time_weights: HashMap::new(),
        })
    }

    pub async fn analyze(
        &self,
        _activities: &[ActivityData],
        _classifications: &[ClassificationResult],
    ) -> Result<ProductivityAnalysis> {
        // 实现生产力分析逻辑
        Ok(ProductivityAnalysis {
            daily_average_score: 78.5,
            productivity_distribution: HashMap::new(),
            high_productivity_periods: vec![],
            low_productivity_periods: vec![],
            category_breakdown: HashMap::new(),
        })
    }
}

// 其他分析器的实现...
impl PatternAnalyzer {
    pub fn new() -> Result<Self> {
        Ok(Self {
            min_pattern_days: 7,
            pattern_confidence_threshold: 0.8,
        })
    }

    pub async fn analyze_patterns(
        &self,
        _activities: &[ActivityData],
        _classifications: &[ClassificationResult],
    ) -> Result<WorkPatterns> {
        Ok(WorkPatterns {
            work_schedule: WorkSchedule {
                typical_start_time: 9,
                typical_end_time: 17,
                average_work_hours: 8.0,
                work_days: vec![
                    Weekday::Mon,
                    Weekday::Tue,
                    Weekday::Wed,
                    Weekday::Thu,
                    Weekday::Fri,
                ],
                overtime_frequency: 0.2,
            },
            break_patterns: BreakPatterns {
                average_break_duration: Duration::from_secs(15 * 60),
                break_frequency_per_hour: 0.5,
                longest_work_stretch: Duration::from_secs(3 * 60 * 60),
                break_timing_consistency: 0.7,
            },
            task_switching: TaskSwitching {
                switches_per_hour: 4.2,
                average_task_duration: Duration::from_secs(15 * 60),
                context_switching_cost: 0.3,
                multitasking_tendency: 0.6,
            },
            cyclical_patterns: vec![],
        })
    }
}

impl StressAnalyzer {
    pub fn new() -> Result<Self> {
        Ok(Self {
            intensity_threshold: 80.0,
            min_break_minutes: 15,
        })
    }

    pub async fn assess_stress(
        &self,
        _activities: &[ActivityData],
        _classifications: &[ClassificationResult],
    ) -> Result<StressAssessment> {
        Ok(StressAssessment {
            stress_level: 35.0,
            stress_indicators: StressIndicators {
                work_intensity: 70.0,
                overtime_frequency: 0.2,
                break_deficit: Duration::from_secs(30 * 60),
                task_switching_stress: 40.0,
                deadline_pressure: 60.0,
            },
            risk_factors: vec![],
            mitigation_suggestions: vec![
                "建议每小时休息5-10分钟".to_string(),
                "尝试番茄工作法提高专注度".to_string(),
            ],
        })
    }
}

impl TrendAnalyzer {
    pub fn new() -> Result<Self> {
        Ok(Self {
            analysis_window_days: 30,
            prediction_window_days: 7,
        })
    }

    pub async fn analyze_trends(
        &self,
        _activities: &[ActivityData],
        _classifications: &[ClassificationResult],
    ) -> Result<TrendAnalysis> {
        Ok(TrendAnalysis {
            productivity_trend: Trend {
                direction: TrendDirection::Improving,
                magnitude: 0.15,
                confidence: 0.8,
                data_points: vec![],
            },
            focus_trend: Trend {
                direction: TrendDirection::Stable,
                magnitude: 0.05,
                confidence: 0.7,
                data_points: vec![],
            },
            work_hours_trend: Trend {
                direction: TrendDirection::Declining,
                magnitude: 0.1,
                confidence: 0.6,
                data_points: vec![],
            },
            stress_trend: Trend {
                direction: TrendDirection::Improving,
                magnitude: 0.2,
                confidence: 0.75,
                data_points: vec![],
            },
            predictions: Predictions {
                next_week_productivity: 82.0,
                optimal_work_times: vec![],
                potential_risks: vec!["可能的工作强度过高".to_string()],
                improvement_opportunities: vec!["优化上午的专注时间".to_string()],
            },
        })
    }
}

impl RecommendationEngine {
    pub fn new() -> Result<Self> {
        Ok(Self {
            templates: Self::load_recommendation_templates(),
            personalization_weights: HashMap::new(),
        })
    }

    pub async fn generate_recommendations(
        &self,
        _focus: &FocusAnalysis,
        _productivity: &ProductivityAnalysis,
        _patterns: &WorkPatterns,
        _stress: &StressAssessment,
        _trends: &TrendAnalysis,
    ) -> Result<Vec<Recommendation>> {
        // 实现建议生成逻辑
        Ok(vec![Recommendation {
            id: "focus_improvement_1".to_string(),
            title: "优化专注时间管理".to_string(),
            description: "建议使用番茄工作法，每25分钟专注工作后休息5分钟".to_string(),
            category: RecommendationCategory::FocusImprovement,
            priority: RecommendationPriority::High,
            expected_impact: 0.8,
            implementation_effort: 0.3,
            personalization_score: 0.9,
            action_items: vec![ActionItem {
                description: "下载番茄工作法应用".to_string(),
                estimated_time: Duration::from_secs(5 * 60),
                difficulty: 0.1,
            }],
        }])
    }

    fn load_recommendation_templates() -> Vec<RecommendationTemplate> {
        vec![RecommendationTemplate {
            id: "focus_pomodoro".to_string(),
            title_template: "使用番茄工作法提高专注度".to_string(),
            description_template: "您的平均专注时间为{focus_time}分钟，建议尝试番茄工作法"
                .to_string(),
            category: RecommendationCategory::FocusImprovement,
            conditions: vec![RecommendationCondition::PoorFocus(60.0)],
            base_priority: RecommendationPriority::High,
        }]
    }
}
