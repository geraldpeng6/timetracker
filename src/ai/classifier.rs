// AI活动分类器 - 智能识别和分类用户活动
// 使用本地机器学习模型，保护用户隐私

use crate::core::tracker::ActivityRecord;
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// 为了兼容性，创建一个类型别名
pub type ActivityData = ActivityRecord;

/// 活动分类器
pub struct ActivityClassifier {
    /// 分类模型
    model: ClassificationModel,
    /// 特征提取器
    feature_extractor: FeatureExtractor,
    /// 分类规则
    rules: Vec<ClassificationRule>,
    /// 学习历史
    learning_history: Vec<LearningRecord>,
    /// 配置
    config: ClassifierConfig,
}

/// 分类模型（简化版本，实际可以集成TensorFlow Lite）
#[allow(dead_code)]
pub struct ClassificationModel {
    /// 权重矩阵
    weights: HashMap<String, f32>,
    /// 偏置
    bias: f32,
    /// 模型版本
    version: String,
    /// 最后更新时间
    last_updated: DateTime<Utc>,
}

/// 特征提取器
#[allow(dead_code)]
pub struct FeatureExtractor {
    /// 应用名称特征
    app_features: HashMap<String, Vec<String>>,
    /// 窗口标题特征
    title_features: HashMap<String, Vec<String>>,
    /// 时间特征
    time_features: TimeFeatures,
}

/// 时间特征
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct TimeFeatures {
    /// 工作时间模式
    work_hours: Vec<(u8, u8)>, // (开始小时, 结束小时)
    /// 周末模式
    weekend_pattern: bool,
    /// 深夜模式
    late_night_threshold: u8,
}

/// 分类规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassificationRule {
    /// 规则ID
    pub id: String,
    /// 规则名称
    pub name: String,
    /// 匹配条件
    pub conditions: Vec<RuleCondition>,
    /// 目标类别
    pub target_category: ActivityCategory,
    /// 置信度
    pub confidence: f32,
    /// 优先级
    pub priority: u8,
}

/// 规则条件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuleCondition {
    /// 应用名称包含
    AppNameContains(String),
    /// 窗口标题包含
    WindowTitleContains(String),
    /// 应用名称匹配正则
    AppNameRegex(String),
    /// 窗口标题匹配正则
    WindowTitleRegex(String),
    /// 时间范围
    TimeRange { start: u8, end: u8 },
    /// 工作日
    Weekday,
    /// 周末
    Weekend,
}

/// 活动类别
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ActivityCategory {
    /// 编程开发
    Development {
        language: Option<String>,
        framework: Option<String>,
        project_type: Option<String>,
    },
    /// 会议沟通
    Communication {
        platform: String,
        meeting_type: MeetingType,
        participants: Option<u32>,
    },
    /// 文档写作
    Documentation {
        doc_type: DocumentType,
        format: Option<String>,
    },
    /// 学习研究
    Learning {
        subject: Option<String>,
        resource_type: Option<String>,
    },
    /// 设计创作
    Design {
        tool: String,
        design_type: Option<String>,
    },
    /// 数据分析
    DataAnalysis {
        tool: String,
        data_type: Option<String>,
    },
    /// 项目管理
    ProjectManagement {
        tool: String,
        activity_type: Option<String>,
    },
    /// 娱乐休闲
    Entertainment {
        content_type: EntertainmentType,
        platform: Option<String>,
    },
    /// 系统管理
    SystemAdmin { task_type: String },
    /// 其他
    Other {
        description: String,
        custom_tags: Vec<String>,
    },
}

/// 会议类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum MeetingType {
    OneOnOne,
    TeamMeeting,
    AllHands,
    Interview,
    Presentation,
    Workshop,
    Training,
    Standup,
    Review,
}

/// 文档类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum DocumentType {
    TechnicalDoc,
    UserManual,
    Specification,
    Report,
    Email,
    Notes,
    Proposal,
    Contract,
    Blog,
}

/// 娱乐类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum EntertainmentType {
    Video,
    Music,
    Gaming,
    SocialMedia,
    News,
    Shopping,
    Reading,
    Podcast,
}

/// 分类结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassificationResult {
    /// 主要类别
    pub primary_category: ActivityCategory,
    /// 置信度
    pub confidence: f32,
    /// 生产力评分 (0-100)
    pub productivity_score: f32,
    /// 替代类别（按置信度排序）
    pub alternative_categories: Vec<(ActivityCategory, f32)>,
    /// 提取的标签
    pub tags: Vec<String>,
    /// 分类时间
    pub timestamp: DateTime<Utc>,
    /// 使用的规则
    pub applied_rules: Vec<String>,
}

/// 学习记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningRecord {
    /// 原始活动数据
    pub activity: ActivityData,
    /// 预测结果
    pub predicted_category: ActivityCategory,
    /// 实际类别（用户反馈）
    pub actual_category: Option<ActivityCategory>,
    /// 学习时间
    pub timestamp: DateTime<Utc>,
    /// 改进效果
    pub improvement_score: Option<f32>,
}

/// 分类器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassifierConfig {
    /// 最小置信度阈值
    pub min_confidence_threshold: f32,
    /// 启用机器学习
    pub enable_ml: bool,
    /// 启用规则引擎
    pub enable_rules: bool,
    /// 学习率
    pub learning_rate: f32,
    /// 最大学习记录数
    pub max_learning_records: usize,
}

impl Default for ClassifierConfig {
    fn default() -> Self {
        Self {
            min_confidence_threshold: 0.7,
            enable_ml: true,
            enable_rules: true,
            learning_rate: 0.01,
            max_learning_records: 10000,
        }
    }
}

impl ActivityClassifier {
    /// 创建新的分类器
    pub fn new(config: ClassifierConfig) -> Result<Self> {
        let model = ClassificationModel::new()?;
        let feature_extractor = FeatureExtractor::new()?;
        let rules = Self::load_default_rules();

        Ok(Self {
            model,
            feature_extractor,
            rules,
            learning_history: Vec::new(),
            config,
        })
    }

    /// 分类活动
    pub async fn classify(&self, activity: &ActivityData) -> Result<ClassificationResult> {
        // 1. 提取特征
        let features = self.feature_extractor.extract_features(activity)?;

        // 2. 应用规则引擎
        let rule_results = if self.config.enable_rules {
            self.apply_rules(activity, &features)?
        } else {
            Vec::new()
        };

        // 3. 应用机器学习模型
        let ml_results = if self.config.enable_ml {
            self.model.predict(&features)?
        } else {
            Vec::new()
        };

        // 4. 合并结果
        let final_result = self.merge_results(rule_results, ml_results, &features)?;

        // 5. 计算生产力评分
        let productivity_score =
            self.calculate_productivity_score(&final_result.primary_category, activity)?;

        Ok(ClassificationResult {
            primary_category: final_result.primary_category,
            confidence: final_result.confidence,
            productivity_score,
            alternative_categories: final_result.alternatives,
            tags: self.extract_tags(activity, &features)?,
            timestamp: Utc::now(),
            applied_rules: final_result.applied_rules,
        })
    }

    /// 从用户反馈学习
    pub async fn learn_from_feedback(&mut self, feedback: UserFeedback) -> Result<()> {
        // 记录学习历史
        let learning_record = LearningRecord {
            activity: feedback.activity.clone(),
            predicted_category: feedback.predicted_category.clone(),
            actual_category: Some(feedback.correct_category.clone()),
            timestamp: Utc::now(),
            improvement_score: None,
        };

        self.learning_history.push(learning_record);

        // 限制历史记录数量
        if self.learning_history.len() > self.config.max_learning_records {
            self.learning_history.remove(0);
        }

        // 更新模型权重
        if self.config.enable_ml {
            self.update_model_weights(&feedback).await?;
        }

        // 更新规则
        if self.config.enable_rules {
            self.update_rules(&feedback).await?;
        }

        Ok(())
    }

    /// 加载默认分类规则
    fn load_default_rules() -> Vec<ClassificationRule> {
        vec![
            // 开发相关规则
            ClassificationRule {
                id: "dev_vscode".to_string(),
                name: "Visual Studio Code 开发".to_string(),
                conditions: vec![RuleCondition::AppNameContains(
                    "Visual Studio Code".to_string(),
                )],
                target_category: ActivityCategory::Development {
                    language: None,
                    framework: None,
                    project_type: None,
                },
                confidence: 0.9,
                priority: 1,
            },
            // 会议相关规则
            ClassificationRule {
                id: "meeting_zoom".to_string(),
                name: "Zoom 会议".to_string(),
                conditions: vec![RuleCondition::AppNameContains("zoom".to_string())],
                target_category: ActivityCategory::Communication {
                    platform: "Zoom".to_string(),
                    meeting_type: MeetingType::TeamMeeting,
                    participants: None,
                },
                confidence: 0.85,
                priority: 1,
            },
            // 娱乐相关规则
            ClassificationRule {
                id: "entertainment_youtube".to_string(),
                name: "YouTube 视频".to_string(),
                conditions: vec![RuleCondition::WindowTitleContains("YouTube".to_string())],
                target_category: ActivityCategory::Entertainment {
                    content_type: EntertainmentType::Video,
                    platform: Some("YouTube".to_string()),
                },
                confidence: 0.8,
                priority: 2,
            },
        ]
    }

    /// 应用分类规则
    fn apply_rules(
        &self,
        activity: &ActivityData,
        features: &ActivityFeatures,
    ) -> Result<Vec<RuleResult>> {
        let mut results = Vec::new();

        for rule in &self.rules {
            if self.rule_matches(rule, activity, features)? {
                results.push(RuleResult {
                    rule_id: rule.id.clone(),
                    category: rule.target_category.clone(),
                    confidence: rule.confidence,
                    priority: rule.priority,
                });
            }
        }

        // 按优先级和置信度排序
        results.sort_by(|a, b| {
            b.priority.cmp(&a.priority).then(
                b.confidence
                    .partial_cmp(&a.confidence)
                    .unwrap_or(std::cmp::Ordering::Equal),
            )
        });

        Ok(results)
    }

    /// 检查规则是否匹配
    fn rule_matches(
        &self,
        rule: &ClassificationRule,
        activity: &ActivityData,
        _features: &ActivityFeatures,
    ) -> Result<bool> {
        for condition in &rule.conditions {
            match condition {
                RuleCondition::AppNameContains(text) => {
                    if !activity
                        .app_name
                        .to_lowercase()
                        .contains(&text.to_lowercase())
                    {
                        return Ok(false);
                    }
                }
                RuleCondition::WindowTitleContains(text) => {
                    if !activity
                        .window_title
                        .to_lowercase()
                        .contains(&text.to_lowercase())
                    {
                        return Ok(false);
                    }
                }
                // 其他条件的实现...
                _ => {}
            }
        }
        Ok(true)
    }

    /// 计算生产力评分
    fn calculate_productivity_score(
        &self,
        category: &ActivityCategory,
        _activity: &ActivityData,
    ) -> Result<f32> {
        let base_score = match category {
            ActivityCategory::Development { .. } => 90.0,
            ActivityCategory::Documentation { .. } => 85.0,
            ActivityCategory::Learning { .. } => 80.0,
            ActivityCategory::Communication { .. } => 70.0,
            ActivityCategory::ProjectManagement { .. } => 75.0,
            ActivityCategory::DataAnalysis { .. } => 85.0,
            ActivityCategory::Design { .. } => 80.0,
            ActivityCategory::SystemAdmin { .. } => 70.0,
            ActivityCategory::Entertainment { .. } => 20.0,
            ActivityCategory::Other { .. } => 50.0,
        };

        // 可以根据时间、上下文等因素调整评分
        Ok(base_score)
    }

    /// 提取标签
    fn extract_tags(
        &self,
        activity: &ActivityData,
        _features: &ActivityFeatures,
    ) -> Result<Vec<String>> {
        let mut tags = Vec::new();

        // 从应用名称提取标签
        if activity.app_name.to_lowercase().contains("code") {
            tags.push("编程".to_string());
        }

        if activity.app_name.to_lowercase().contains("browser")
            || activity.app_name.to_lowercase().contains("chrome")
            || activity.app_name.to_lowercase().contains("firefox")
        {
            tags.push("浏览器".to_string());
        }

        // 从窗口标题提取标签
        if activity.window_title.to_lowercase().contains("meeting") {
            tags.push("会议".to_string());
        }

        Ok(tags)
    }

    /// 合并规则和机器学习结果
    fn merge_results(
        &self,
        rule_results: Vec<RuleResult>,
        _ml_results: Vec<(ActivityCategory, f32)>,
        _features: &ActivityFeatures,
    ) -> Result<MergedResult> {
        // 简化实现：优先使用规则结果
        if let Some(rule_result) = rule_results.first() {
            Ok(MergedResult {
                primary_category: rule_result.category.clone(),
                confidence: rule_result.confidence,
                alternatives: vec![],
                applied_rules: vec![rule_result.rule_id.clone()],
            })
        } else {
            // 默认分类
            Ok(MergedResult {
                primary_category: ActivityCategory::Other {
                    description: "未分类".to_string(),
                    custom_tags: vec![],
                },
                confidence: 0.5,
                alternatives: vec![],
                applied_rules: vec![],
            })
        }
    }

    /// 更新模型权重
    async fn update_model_weights(&mut self, _feedback: &UserFeedback) -> Result<()> {
        // 简化实现：记录反馈用于未来的模型训练
        // 实际应用中会更新神经网络权重
        Ok(())
    }

    /// 更新规则
    async fn update_rules(&mut self, _feedback: &UserFeedback) -> Result<()> {
        // 简化实现：基于反馈调整规则权重
        // 实际应用中会动态调整规则优先级和置信度
        Ok(())
    }
}

/// 活动特征
#[derive(Debug, Clone)]
pub struct ActivityFeatures {
    pub app_name_tokens: Vec<String>,
    pub window_title_tokens: Vec<String>,
    pub time_features: TimeFeatures,
    pub context_features: Vec<String>,
}

/// 规则结果
#[derive(Debug, Clone)]
pub struct RuleResult {
    pub rule_id: String,
    pub category: ActivityCategory,
    pub confidence: f32,
    pub priority: u8,
}

/// 合并结果
#[derive(Debug, Clone)]
pub struct MergedResult {
    pub primary_category: ActivityCategory,
    pub confidence: f32,
    pub alternatives: Vec<(ActivityCategory, f32)>,
    pub applied_rules: Vec<String>,
}

/// 用户反馈
#[derive(Debug, Clone)]
pub struct UserFeedback {
    pub activity: ActivityData,
    pub predicted_category: ActivityCategory,
    pub correct_category: ActivityCategory,
    pub feedback_type: FeedbackType,
}

/// 反馈类型
#[derive(Debug, Clone)]
pub enum FeedbackType {
    CategoryCorrection,
    ProductivityScoreAdjustment,
    TagSuggestion,
}

// 实现其他必要的结构体和方法...
impl ClassificationModel {
    pub fn new() -> Result<Self> {
        Ok(Self {
            weights: HashMap::new(),
            bias: 0.0,
            version: "1.0.0".to_string(),
            last_updated: Utc::now(),
        })
    }

    pub fn predict(&self, _features: &ActivityFeatures) -> Result<Vec<(ActivityCategory, f32)>> {
        // 简化的预测实现
        // 实际应用中会使用训练好的模型
        Ok(vec![])
    }
}

impl FeatureExtractor {
    pub fn new() -> Result<Self> {
        Ok(Self {
            app_features: HashMap::new(),
            title_features: HashMap::new(),
            time_features: TimeFeatures {
                work_hours: vec![(9, 17)],
                weekend_pattern: false,
                late_night_threshold: 22,
            },
        })
    }

    pub fn extract_features(&self, activity: &ActivityData) -> Result<ActivityFeatures> {
        Ok(ActivityFeatures {
            app_name_tokens: activity
                .app_name
                .split_whitespace()
                .map(|s| s.to_lowercase())
                .collect(),
            window_title_tokens: activity
                .window_title
                .split_whitespace()
                .map(|s| s.to_lowercase())
                .collect(),
            time_features: self.time_features.clone(),
            context_features: vec![],
        })
    }
}
