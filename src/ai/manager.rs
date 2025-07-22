use crate::ai::client::{AIMessage, AIRequest, UnifiedAIClient};
use crate::ai::config::{AIConfig, AIModelConfig, AIProvider};
use anyhow::Result;
use std::io::{self, Write};

/// AI配置管理器
pub struct AIConfigManager {
    config: AIConfig,
}

impl AIConfigManager {
    /// 创建新的配置管理器
    pub fn new() -> Result<Self> {
        let config = AIConfig::load()?;
        Ok(Self { config })
    }

    /// 显示当前配置
    pub fn show_config(&self) {
        println!("=== AI配置信息 ===");
        println!("当前厂商: {}", self.config.current_provider);
        println!("当前模型: {}", self.config.current_model);

        if let Some(model_config) = self.config.get_current_model_config() {
            println!("模型显示名: {}", model_config.display_name);
            println!("API端点: {}", model_config.api_url);
            println!("最大Token数: {}", model_config.max_tokens);
            println!("温度参数: {}", model_config.temperature);
            println!(
                "支持流式输出: {}",
                if model_config.supports_streaming {
                    "是"
                } else {
                    "否"
                }
            );
            println!(
                "支持函数调用: {}",
                if model_config.supports_function_calling {
                    "是"
                } else {
                    "否"
                }
            );
        }

        println!("\n=== 已配置的厂商 ===");
        for provider in self.config.get_available_providers() {
            let status = if self.config.is_provider_configured(&provider) {
                "✓ 已配置"
            } else {
                "✗ 未配置"
            };
            println!("{}: {}", provider, status);
        }

        println!("\n=== 配置参数 ===");
        println!("请求超时: {}秒", self.config.timeout_seconds);
        println!("重试次数: {}", self.config.retry_attempts);
    }

    /// 列出所有可用模型
    pub fn list_models(&self) {
        println!("=== 可用模型列表 ===");

        for provider in self.config.get_available_providers() {
            println!("\n【{}】", provider);
            let models = self.config.get_models_by_provider(&provider);

            for model in models {
                let current_marker = if self.config.current_model == model.model_name {
                    " (当前)"
                } else {
                    ""
                };

                let configured_marker = if self.config.is_provider_configured(&provider) {
                    " ✓"
                } else {
                    " ✗"
                };

                println!(
                    "  - {}{}{}",
                    model.display_name, current_marker, configured_marker
                );
                println!("    模型ID: {}", model.model_name);
                println!("    最大Token: {}", model.max_tokens);
            }
        }
    }

    /// 配置指定厂商
    pub async fn configure_provider(
        &mut self,
        provider: &str,
        model: Option<&String>,
        api_key: Option<&String>,
        endpoint: Option<&String>,
    ) -> Result<()> {
        // 解析厂商
        let ai_provider = match provider.to_lowercase().as_str() {
            "openai" => AIProvider::OpenAI,
            "anthropic" => AIProvider::Anthropic,
            "google" => AIProvider::Google,
            "baidu" | "百度" => AIProvider::Baidu,
            "alibaba" | "阿里云" => AIProvider::Alibaba,
            "siliconflow" | "硅基流动" => AIProvider::SiliconFlow,
            "local" | "本地" => AIProvider::Local,
            _ => return Err(anyhow::anyhow!("不支持的厂商: {}", provider)),
        };

        // 设置API密钥
        if let Some(key) = api_key {
            if !key.is_empty() {
                self.config
                    .set_api_key(ai_provider.clone(), key.to_string());
                println!("✓ {}的API密钥已设置", ai_provider);
            }
        }

        // 设置自定义端点
        if let Some(ep) = endpoint {
            if !ep.is_empty() {
                self.config
                    .set_custom_endpoint(ai_provider.clone(), ep.to_string());
                println!("✓ {}的自定义端点已设置", ai_provider);
            }
        }

        // 设置模型
        if let Some(model_name) = model {
            if !model_name.is_empty() {
                // 检查模型是否存在
                if self.config.models.contains_key(model_name) {
                    self.config.current_provider = ai_provider.clone();
                    self.config.current_model = model_name.clone();
                    println!("✓ 已切换到模型: {}", model_name);
                } else {
                    println!(
                        "警告: 模型 {} 不存在，请使用 'timetracker ai list' 查看可用模型",
                        model_name
                    );
                }
            }
        } else {
            // 如果没有指定模型，切换到该厂商的默认模型
            let model_entries = self.config.get_model_entries_by_provider(&ai_provider);
            if let Some((default_model_key, default_model_config)) = model_entries.first() {
                let default_model_key = default_model_key.clone();
                let default_model_display_name = default_model_config.display_name.clone();

                self.config.current_provider = ai_provider.clone();
                self.config.current_model = default_model_key;
                println!("✓ 已切换到默认模型: {}", default_model_display_name);
            }
        }

        // 保存配置
        self.save_config()?;

        // 如果配置了API密钥，测试连接
        if api_key.is_some() && self.config.is_provider_configured(&ai_provider) {
            println!("\n正在测试配置...");
            match self.test_current_config().await {
                Ok(_) => println!("✓ 配置测试成功!"),
                Err(e) => println!("⚠ 配置测试失败: {}", e),
            }
        }

        Ok(())
    }

    /// 交互式配置API密钥
    #[allow(dead_code)]
    pub fn configure_api_keys(&mut self) -> Result<()> {
        println!("=== 配置API密钥 ===");
        println!("提示: 直接按回车跳过该厂商的配置");

        for provider in self.config.get_available_providers() {
            if provider == AIProvider::Local {
                continue; // 本地模型不需要API密钥
            }

            let current_status = if self.config.is_provider_configured(&provider) {
                "已配置"
            } else {
                "未配置"
            };

            print!(
                "\n{} (当前状态: {})\n请输入API密钥: ",
                provider, current_status
            );
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let api_key = input.trim();

            if !api_key.is_empty() {
                self.config
                    .set_api_key(provider.clone(), api_key.to_string());
                println!("✓ {}的API密钥已设置", provider);
            } else {
                println!("跳过{}的配置", provider);
            }
        }

        self.save_config()?;
        println!("\n配置已保存!");
        Ok(())
    }

    /// 选择厂商和模型
    pub fn select_model(&mut self) -> Result<()> {
        println!("=== 选择AI模型 ===");

        // 显示可用厂商
        let providers = self.config.get_available_providers();
        println!("可用厂商:");
        for (i, provider) in providers.iter().enumerate() {
            let status = if self.config.is_provider_configured(provider) {
                "✓"
            } else {
                "✗"
            };
            println!("  {}. {} {}", i + 1, provider, status);
        }

        print!("\n请选择厂商 (输入数字): ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let provider_index: usize = input
            .trim()
            .parse()
            .map_err(|_| anyhow::anyhow!("无效的输入"))?;

        if provider_index == 0 || provider_index > providers.len() {
            return Err(anyhow::anyhow!("无效的厂商选择"));
        }

        let selected_provider = providers[provider_index - 1].clone();

        // 检查是否已配置
        if !self.config.is_provider_configured(&selected_provider)
            && selected_provider != AIProvider::Local
        {
            println!("警告: 该厂商尚未配置API密钥，请先运行 'timetracker ai config' 进行配置");
        }

        // 显示该厂商的可用模型
        let model_entries = self
            .config
            .get_model_entries_by_provider(&selected_provider);
        println!("\n{}的可用模型:", selected_provider);
        for (i, (_, model)) in model_entries.iter().enumerate() {
            println!("  {}. {}", i + 1, model.display_name);
            println!("     模型ID: {}", model.model_name);
            println!("     最大Token: {}", model.max_tokens);
        }

        print!("\n请选择模型 (输入数字): ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let model_index: usize = input
            .trim()
            .parse()
            .map_err(|_| anyhow::anyhow!("无效的输入"))?;

        if model_index == 0 || model_index > model_entries.len() {
            return Err(anyhow::anyhow!("无效的模型选择"));
        }

        let (selected_model_key, selected_model_config) = &model_entries[model_index - 1];
        let selected_model_key = selected_model_key.clone();
        let selected_model_display_name = selected_model_config.display_name.clone();

        // 更新配置
        self.config.current_provider = selected_provider.clone();
        self.config.current_model = selected_model_key;

        self.save_config()?;

        println!(
            "\n✓ 已切换到: {} - {}",
            selected_provider, selected_model_display_name
        );
        Ok(())
    }

    /// 添加自定义模型
    #[allow(dead_code)]
    pub fn add_custom_model(&mut self) -> Result<()> {
        println!("=== 添加自定义模型 ===");

        // 选择厂商
        let providers = self.config.get_available_providers();
        println!("选择厂商:");
        for (i, provider) in providers.iter().enumerate() {
            println!("  {}. {}", i + 1, provider);
        }

        print!("请选择厂商 (输入数字): ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let provider_index: usize = input
            .trim()
            .parse()
            .map_err(|_| anyhow::anyhow!("无效的输入"))?;

        if provider_index == 0 || provider_index > providers.len() {
            return Err(anyhow::anyhow!("无效的厂商选择"));
        }

        let provider = providers[provider_index - 1].clone();

        // 输入模型信息
        print!("模型ID: ");
        io::stdout().flush()?;
        let mut model_id = String::new();
        io::stdin().read_line(&mut model_id)?;
        let model_id = model_id.trim().to_string();

        print!("显示名称: ");
        io::stdout().flush()?;
        let mut display_name = String::new();
        io::stdin().read_line(&mut display_name)?;
        let display_name = display_name.trim().to_string();

        print!("API端点 (可选，按回车使用默认): ");
        io::stdout().flush()?;
        let mut api_url = String::new();
        io::stdin().read_line(&mut api_url)?;
        let api_url = api_url.trim();

        // 使用默认端点或用户输入
        let final_api_url = if api_url.is_empty() {
            self.config
                .get_api_endpoint(&provider)
                .unwrap_or_else(|| "http://localhost:11434/api/chat".to_string())
        } else {
            api_url.to_string()
        };

        print!("最大Token数 (默认4096): ");
        io::stdout().flush()?;
        let mut max_tokens_input = String::new();
        io::stdin().read_line(&mut max_tokens_input)?;
        let max_tokens = if max_tokens_input.trim().is_empty() {
            4096
        } else {
            max_tokens_input.trim().parse().unwrap_or(4096)
        };

        print!("温度参数 (默认0.7): ");
        io::stdout().flush()?;
        let mut temperature_input = String::new();
        io::stdin().read_line(&mut temperature_input)?;
        let temperature = if temperature_input.trim().is_empty() {
            0.7
        } else {
            temperature_input.trim().parse().unwrap_or(0.7)
        };

        // 创建模型配置
        let model_config = AIModelConfig {
            provider: provider.clone(),
            model_name: model_id.clone(),
            display_name,
            api_url: final_api_url,
            max_tokens,
            temperature,
            supports_streaming: true,
            supports_function_calling: provider != AIProvider::Local,
        };

        // 添加到配置
        let config_key = format!("custom-{}", model_id);
        self.config.add_custom_model(config_key, model_config);

        self.save_config()?;

        println!("✓ 自定义模型已添加: {}", model_id);
        Ok(())
    }

    /// 设置自定义端点
    #[allow(dead_code)]
    pub fn set_custom_endpoint(&mut self) -> Result<()> {
        println!("=== 设置自定义端点 ===");

        let providers = self.config.get_available_providers();
        println!("选择厂商:");
        for (i, provider) in providers.iter().enumerate() {
            println!("  {}. {}", i + 1, provider);
        }

        print!("请选择厂商 (输入数字): ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let provider_index: usize = input
            .trim()
            .parse()
            .map_err(|_| anyhow::anyhow!("无效的输入"))?;

        if provider_index == 0 || provider_index > providers.len() {
            return Err(anyhow::anyhow!("无效的厂商选择"));
        }

        let provider = providers[provider_index - 1].clone();

        print!("请输入自定义端点URL: ");
        io::stdout().flush()?;

        let mut endpoint = String::new();
        io::stdin().read_line(&mut endpoint)?;
        let endpoint = endpoint.trim().to_string();

        if endpoint.is_empty() {
            return Err(anyhow::anyhow!("端点URL不能为空"));
        }

        self.config
            .set_custom_endpoint(provider.clone(), endpoint.clone());
        self.save_config()?;

        println!("✓ {}的自定义端点已设置为: {}", provider, endpoint);
        Ok(())
    }

    /// 测试当前配置
    pub async fn test_current_config(&self) -> Result<()> {
        println!("=== 测试当前AI配置 ===");

        let model_config = self
            .config
            .get_current_model_config()
            .ok_or_else(|| anyhow::anyhow!("未找到当前模型配置"))?;

        println!(
            "正在测试: {} - {}",
            self.config.current_provider, model_config.display_name
        );

        // 检查API密钥
        if !self
            .config
            .is_provider_configured(&self.config.current_provider)
            && self.config.current_provider != AIProvider::Local
        {
            return Err(anyhow::anyhow!("当前厂商未配置API密钥"));
        }

        // 创建测试客户端
        let client = UnifiedAIClient::new(self.config.clone())?;

        // 发送测试请求
        let request = AIRequest {
            messages: vec![AIMessage {
                role: "user".to_string(),
                content: "请回复'测试成功'".to_string(),
            }],
            max_tokens: Some(50),
            temperature: Some(0.1),
            stream: Some(false),
        };

        match client.chat(request).await {
            Ok(response) => {
                println!("✓ 测试成功!");
                println!("响应内容: {}", response.content);
                if let Some(usage) = response.usage {
                    println!(
                        "Token使用: 输入{} + 输出{} = 总计{}",
                        usage.prompt_tokens, usage.completion_tokens, usage.total_tokens
                    );
                }
            }
            Err(e) => {
                println!("✗ 测试失败: {}", e);
                return Err(e);
            }
        }

        Ok(())
    }

    /// 保存配置
    fn save_config(&self) -> Result<()> {
        self.config.save()
    }

    /// 保存配置（公共接口）
    pub fn save(&self) -> Result<()> {
        self.save_config()
    }

    /// 获取配置
    #[allow(dead_code)]
    pub fn get_config(&self) -> &AIConfig {
        &self.config
    }

    /// 获取可变配置引用（用于TUI）
    pub fn get_config_mut(&mut self) -> &mut AIConfig {
        &mut self.config
    }

    /// 设置API密钥（简化接口）
    pub fn set_api_key(&mut self, provider: AIProvider, key: String) -> Result<()> {
        self.config.set_api_key(provider, key);
        self.save_config()
    }

    /// 设置自定义端点（简化接口）
    pub fn set_endpoint(&mut self, provider: AIProvider, endpoint: String) -> Result<()> {
        self.config.set_custom_endpoint(provider, endpoint);
        self.save_config()
    }

    /// 设置当前提供商
    pub fn set_current_provider(&mut self, provider: AIProvider) -> Result<()> {
        self.config.current_provider = provider;
        self.save_config()
    }

    /// 设置当前模型
    pub fn set_current_model(&mut self, model: String) -> Result<()> {
        self.config.current_model = model;
        self.save_config()
    }
}
