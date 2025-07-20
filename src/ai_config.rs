use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;
use anyhow::Result;

/// AI厂商枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum AIProvider {
    OpenAI,
    Anthropic,
    Google,
    Baidu,
    Alibaba,
    SiliconFlow,
    Local,
}

impl std::fmt::Display for AIProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AIProvider::OpenAI => write!(f, "OpenAI"),
            AIProvider::Anthropic => write!(f, "Anthropic"),
            AIProvider::Google => write!(f, "Google"),
            AIProvider::Baidu => write!(f, "百度"),
            AIProvider::Alibaba => write!(f, "阿里云"),
            AIProvider::SiliconFlow => write!(f, "硅基流动"),
            AIProvider::Local => write!(f, "本地模型"),
        }
    }
}

/// AI模型配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIModelConfig {
    pub provider: AIProvider,
    pub model_name: String,
    pub display_name: String,
    pub api_url: String,
    pub max_tokens: u32,
    pub temperature: f32,
    pub supports_streaming: bool,
    pub supports_function_calling: bool,
}

/// AI配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIConfig {
    pub current_provider: AIProvider,
    pub current_model: String,
    pub api_keys: HashMap<AIProvider, String>,
    pub models: HashMap<String, AIModelConfig>,
    pub custom_endpoints: HashMap<AIProvider, String>,
    pub timeout_seconds: u64,
    pub retry_attempts: u32,
}

impl Default for AIConfig {
    fn default() -> Self {
        let mut models = HashMap::new();
        
        // OpenAI 模型
        models.insert("gpt-4o".to_string(), AIModelConfig {
            provider: AIProvider::OpenAI,
            model_name: "gpt-4o".to_string(),
            display_name: "GPT-4o".to_string(),
            api_url: "https://api.openai.com/v1/chat/completions".to_string(),
            max_tokens: 4096,
            temperature: 0.7,
            supports_streaming: true,
            supports_function_calling: true,
        });
        
        models.insert("gpt-4o-mini".to_string(), AIModelConfig {
            provider: AIProvider::OpenAI,
            model_name: "gpt-4o-mini".to_string(),
            display_name: "GPT-4o Mini".to_string(),
            api_url: "https://api.openai.com/v1/chat/completions".to_string(),
            max_tokens: 4096,
            temperature: 0.7,
            supports_streaming: true,
            supports_function_calling: true,
        });

        models.insert("gpt-3.5-turbo".to_string(), AIModelConfig {
            provider: AIProvider::OpenAI,
            model_name: "gpt-3.5-turbo".to_string(),
            display_name: "GPT-3.5 Turbo".to_string(),
            api_url: "https://api.openai.com/v1/chat/completions".to_string(),
            max_tokens: 4096,
            temperature: 0.7,
            supports_streaming: true,
            supports_function_calling: true,
        });

        // Anthropic 模型
        models.insert("claude-3-5-sonnet".to_string(), AIModelConfig {
            provider: AIProvider::Anthropic,
            model_name: "claude-3-5-sonnet-20241022".to_string(),
            display_name: "Claude 3.5 Sonnet".to_string(),
            api_url: "https://api.anthropic.com/v1/messages".to_string(),
            max_tokens: 4096,
            temperature: 0.7,
            supports_streaming: true,
            supports_function_calling: true,
        });

        models.insert("claude-3-haiku".to_string(), AIModelConfig {
            provider: AIProvider::Anthropic,
            model_name: "claude-3-haiku-20240307".to_string(),
            display_name: "Claude 3 Haiku".to_string(),
            api_url: "https://api.anthropic.com/v1/messages".to_string(),
            max_tokens: 4096,
            temperature: 0.7,
            supports_streaming: true,
            supports_function_calling: true,
        });

        // Google 模型
        models.insert("gemini-pro".to_string(), AIModelConfig {
            provider: AIProvider::Google,
            model_name: "gemini-pro".to_string(),
            display_name: "Gemini Pro".to_string(),
            api_url: "https://generativelanguage.googleapis.com/v1beta/openai/chat/completions".to_string(),
            max_tokens: 4096,
            temperature: 0.7,
            supports_streaming: true,
            supports_function_calling: true,
        });

        models.insert("gemini-flash".to_string(), AIModelConfig {
            provider: AIProvider::Google,
            model_name: "gemini-1.5-flash".to_string(),
            display_name: "Gemini 1.5 Flash".to_string(),
            api_url: "https://generativelanguage.googleapis.com/v1beta/openai/chat/completions".to_string(),
            max_tokens: 4096,
            temperature: 0.7,
            supports_streaming: true,
            supports_function_calling: true,
        });

        // 百度模型
        models.insert("ernie-bot".to_string(), AIModelConfig {
            provider: AIProvider::Baidu,
            model_name: "ERNIE-Bot".to_string(),
            display_name: "文心一言".to_string(),
            api_url: "https://aip.baidubce.com/rpc/2.0/ai_custom/v1/wenxinworkshop/chat/completions".to_string(),
            max_tokens: 4096,
            temperature: 0.7,
            supports_streaming: true,
            supports_function_calling: false,
        });

        // 阿里云模型
        models.insert("qwen-turbo".to_string(), AIModelConfig {
            provider: AIProvider::Alibaba,
            model_name: "qwen-turbo".to_string(),
            display_name: "通义千问 Turbo".to_string(),
            api_url: "https://dashscope.aliyuncs.com/api/v1/services/aigc/text-generation/generation".to_string(),
            max_tokens: 4096,
            temperature: 0.7,
            supports_streaming: true,
            supports_function_calling: true,
        });

        models.insert("qwen-plus".to_string(), AIModelConfig {
            provider: AIProvider::Alibaba,
            model_name: "qwen-plus".to_string(),
            display_name: "通义千问 Plus".to_string(),
            api_url: "https://dashscope.aliyuncs.com/api/v1/services/aigc/text-generation/generation".to_string(),
            max_tokens: 4096,
            temperature: 0.7,
            supports_streaming: true,
            supports_function_calling: true,
        });

        // 硅基流动模型
        models.insert("deepseek-v3".to_string(), AIModelConfig {
            provider: AIProvider::SiliconFlow,
            model_name: "deepseek-ai/DeepSeek-V3".to_string(),
            display_name: "DeepSeek V3".to_string(),
            api_url: "https://api.siliconflow.cn/v1/chat/completions".to_string(),
            max_tokens: 4096,
            temperature: 0.7,
            supports_streaming: true,
            supports_function_calling: true,
        });

        models.insert("qwen2.5-72b".to_string(), AIModelConfig {
            provider: AIProvider::SiliconFlow,
            model_name: "Qwen/Qwen2.5-72B-Instruct".to_string(),
            display_name: "Qwen2.5 72B".to_string(),
            api_url: "https://api.siliconflow.cn/v1/chat/completions".to_string(),
            max_tokens: 4096,
            temperature: 0.7,
            supports_streaming: true,
            supports_function_calling: true,
        });

        models.insert("qwen2.5-32b".to_string(), AIModelConfig {
            provider: AIProvider::SiliconFlow,
            model_name: "Qwen/Qwen2.5-32B-Instruct".to_string(),
            display_name: "Qwen2.5 32B".to_string(),
            api_url: "https://api.siliconflow.cn/v1/chat/completions".to_string(),
            max_tokens: 4096,
            temperature: 0.7,
            supports_streaming: true,
            supports_function_calling: true,
        });

        models.insert("qwen2.5-7b".to_string(), AIModelConfig {
            provider: AIProvider::SiliconFlow,
            model_name: "Qwen/Qwen2.5-7B-Instruct".to_string(),
            display_name: "Qwen2.5 7B".to_string(),
            api_url: "https://api.siliconflow.cn/v1/chat/completions".to_string(),
            max_tokens: 4096,
            temperature: 0.7,
            supports_streaming: true,
            supports_function_calling: true,
        });

        models.insert("glm-4-9b".to_string(), AIModelConfig {
            provider: AIProvider::SiliconFlow,
            model_name: "THUDM/glm-4-9b-chat".to_string(),
            display_name: "GLM-4 9B".to_string(),
            api_url: "https://api.siliconflow.cn/v1/chat/completions".to_string(),
            max_tokens: 4096,
            temperature: 0.7,
            supports_streaming: true,
            supports_function_calling: true,
        });

        // 本地模型
        models.insert("llama3:8b".to_string(), AIModelConfig {
            provider: AIProvider::Local,
            model_name: "llama3:8b".to_string(),
            display_name: "Llama 3 8B (本地)".to_string(),
            api_url: "http://localhost:11434/api/chat".to_string(),
            max_tokens: 4096,
            temperature: 0.7,
            supports_streaming: true,
            supports_function_calling: false,
        });

        Self {
            current_provider: AIProvider::OpenAI,
            current_model: "gpt-3.5-turbo".to_string(),
            api_keys: HashMap::new(),
            models,
            custom_endpoints: HashMap::new(),
            timeout_seconds: 30,
            retry_attempts: 3,
        }
    }
}

impl AIConfig {
    /// 从配置文件加载
    pub fn load() -> Result<Self> {
        let config_path = Self::get_config_path()?;
        
        if config_path.exists() {
            let content = fs::read_to_string(&config_path)?;
            let mut config: AIConfig = serde_json::from_str(&content)?;
            
            // 从环境变量加载API密钥
            config.load_api_keys_from_env();
            
            Ok(config)
        } else {
            let mut config = Self::default();
            config.load_api_keys_from_env();
            config.save()?;
            Ok(config)
        }
    }

    /// 保存配置文件
    pub fn save(&self) -> Result<()> {
        let config_path = Self::get_config_path()?;
        
        // 创建目录
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // 保存时不包含API密钥（安全考虑）
        let mut config_to_save = self.clone();
        config_to_save.api_keys.clear();
        
        let content = serde_json::to_string_pretty(&config_to_save)?;
        fs::write(&config_path, content)?;
        
        Ok(())
    }

    /// 获取配置文件路径
    fn get_config_path() -> Result<PathBuf> {
        let home_dir = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("无法获取用户主目录"))?;
        
        Ok(home_dir.join(".timetracker").join("ai_config.json"))
    }

    /// 从环境变量加载API密钥
    fn load_api_keys_from_env(&mut self) {
        // OpenAI
        if let Ok(key) = env::var("OPENAI_API_KEY") {
            self.api_keys.insert(AIProvider::OpenAI, key);
        }

        // Anthropic
        if let Ok(key) = env::var("ANTHROPIC_API_KEY") {
            self.api_keys.insert(AIProvider::Anthropic, key);
        }

        // Google
        if let Ok(key) = env::var("GOOGLE_API_KEY") {
            self.api_keys.insert(AIProvider::Google, key);
        }

        // 百度
        if let Ok(key) = env::var("BAIDU_API_KEY") {
            self.api_keys.insert(AIProvider::Baidu, key);
        }

        // 阿里云
        if let Ok(key) = env::var("ALIBABA_API_KEY") {
            self.api_keys.insert(AIProvider::Alibaba, key);
        }

        // 硅基流动
        if let Ok(key) = env::var("SILICONFLOW_API_KEY") {
            self.api_keys.insert(AIProvider::SiliconFlow, key);
        }
    }

    /// 获取当前模型配置
    pub fn get_current_model_config(&self) -> Option<&AIModelConfig> {
        self.models.get(&self.current_model)
    }

    /// 获取API密钥
    pub fn get_api_key(&self, provider: &AIProvider) -> Option<&String> {
        self.api_keys.get(provider)
    }

    /// 设置API密钥
    pub fn set_api_key(&mut self, provider: AIProvider, key: String) {
        self.api_keys.insert(provider, key);
    }

    /// 获取指定厂商的所有模型
    pub fn get_models_by_provider(&self, provider: &AIProvider) -> Vec<&AIModelConfig> {
        self.models
            .values()
            .filter(|model| &model.provider == provider)
            .collect()
    }

    /// 获取指定厂商的所有模型键和配置对
    pub fn get_model_entries_by_provider(&self, provider: &AIProvider) -> Vec<(String, &AIModelConfig)> {
        self.models
            .iter()
            .filter(|(_, model)| &model.provider == provider)
            .map(|(key, model)| (key.clone(), model))
            .collect()
    }

    /// 获取所有可用的厂商
    pub fn get_available_providers(&self) -> Vec<AIProvider> {
        let mut providers: Vec<AIProvider> = self.models
            .values()
            .map(|model| model.provider.clone())
            .collect();
        
        providers.sort_by_key(|p| format!("{:?}", p));
        providers.dedup();
        providers
    }

    /// 检查厂商是否已配置
    pub fn is_provider_configured(&self, provider: &AIProvider) -> bool {
        match provider {
            AIProvider::Local => true, // 本地模型不需要API密钥
            _ => self.api_keys.contains_key(provider),
        }
    }

    /// 添加自定义模型
    pub fn add_custom_model(&mut self, id: String, config: AIModelConfig) {
        self.models.insert(id, config);
    }

    /// 设置自定义端点
    pub fn set_custom_endpoint(&mut self, provider: AIProvider, endpoint: String) {
        self.custom_endpoints.insert(provider, endpoint);
    }

    /// 获取有效的API端点
    pub fn get_api_endpoint(&self, provider: &AIProvider) -> Option<String> {
        // 优先使用自定义端点
        if let Some(custom_endpoint) = self.custom_endpoints.get(provider) {
            return Some(custom_endpoint.clone());
        }

        // 使用当前模型的默认端点
        if let Some(model_config) = self.get_current_model_config() {
            if &model_config.provider == provider {
                return Some(model_config.api_url.clone());
            }
        }

        None
    }
}