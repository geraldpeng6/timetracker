use crate::ai::config::{AIConfig, AIModelConfig, AIProvider};
use anyhow::{anyhow, Result};
use reqwest::{
    header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE},
    Client,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::time::Duration;

/// 统一的AI消息格式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIMessage {
    pub role: String,
    pub content: String,
}

/// 统一的AI请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIRequest {
    pub messages: Vec<AIMessage>,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
    pub stream: Option<bool>,
}

/// 统一的AI响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIResponse {
    pub content: String,
    pub model: String,
    pub usage: Option<AIUsage>,
    pub finish_reason: Option<String>,
}

/// 使用统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// 统一AI客户端
pub struct UnifiedAIClient {
    config: AIConfig,
    http_client: Client,
}

impl UnifiedAIClient {
    /// 创建新的客户端
    pub fn new(config: AIConfig) -> Result<Self> {
        let http_client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .build()?;

        Ok(Self {
            config,
            http_client,
        })
    }

    /// 发送聊天请求
    pub async fn chat(&self, request: AIRequest) -> Result<AIResponse> {
        let model_config = self
            .config
            .get_current_model_config()
            .ok_or_else(|| anyhow!("未找到当前模型配置"))?;

        match model_config.provider {
            AIProvider::OpenAI => self.call_openai_api(request, model_config).await,
            AIProvider::Anthropic => self.call_anthropic_api(request, model_config).await,
            AIProvider::Google => self.call_google_api(request, model_config).await,
            AIProvider::Baidu => self.call_baidu_api(request, model_config).await,
            AIProvider::Alibaba => self.call_alibaba_api(request, model_config).await,
            AIProvider::SiliconFlow => self.call_siliconflow_api(request, model_config).await,
            AIProvider::Local => self.call_local_api(request, model_config).await,
        }
    }

    /// 调用OpenAI API
    async fn call_openai_api(
        &self,
        request: AIRequest,
        model_config: &AIModelConfig,
    ) -> Result<AIResponse> {
        let api_key = self
            .config
            .get_api_key(&AIProvider::OpenAI)
            .ok_or_else(|| anyhow!("未配置OpenAI API密钥"))?;

        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", api_key))?,
        );
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        let payload = json!({
            "model": model_config.model_name,
            "messages": request.messages,
            "max_tokens": request.max_tokens.unwrap_or(model_config.max_tokens),
            "temperature": request.temperature.unwrap_or(model_config.temperature),
            "stream": request.stream.unwrap_or(false)
        });

        let response = self
            .http_client
            .post(&model_config.api_url)
            .headers(headers)
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("OpenAI API错误: {}", error_text));
        }

        let response_json: Value = response.json().await?;
        self.parse_openai_response(response_json, &model_config.model_name)
    }

    /// 调用Anthropic API
    async fn call_anthropic_api(
        &self,
        request: AIRequest,
        model_config: &AIModelConfig,
    ) -> Result<AIResponse> {
        let api_key = self
            .config
            .get_api_key(&AIProvider::Anthropic)
            .ok_or_else(|| anyhow!("未配置Anthropic API密钥"))?;

        let mut headers = HeaderMap::new();
        headers.insert("x-api-key", HeaderValue::from_str(api_key)?);
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert("anthropic-version", HeaderValue::from_static("2023-06-01"));

        // 转换消息格式 - Anthropic需要分离system和user消息
        let mut system_message = String::new();
        let mut messages = Vec::new();

        for msg in request.messages {
            if msg.role == "system" {
                system_message = msg.content;
            } else {
                messages.push(json!({
                    "role": msg.role,
                    "content": msg.content
                }));
            }
        }

        let mut payload = json!({
            "model": model_config.model_name,
            "max_tokens": request.max_tokens.unwrap_or(model_config.max_tokens),
            "messages": messages
        });

        if !system_message.is_empty() {
            payload["system"] = json!(system_message);
        }

        let response = self
            .http_client
            .post(&model_config.api_url)
            .headers(headers)
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("Anthropic API错误: {}", error_text));
        }

        let response_json: Value = response.json().await?;
        self.parse_anthropic_response(response_json, &model_config.model_name)
    }

    /// 调用Google API (使用OpenAI兼容接口)
    async fn call_google_api(
        &self,
        request: AIRequest,
        model_config: &AIModelConfig,
    ) -> Result<AIResponse> {
        let api_key = self
            .config
            .get_api_key(&AIProvider::Google)
            .ok_or_else(|| anyhow!("未配置Google API密钥"))?;

        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", api_key))?,
        );
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        let payload = json!({
            "model": model_config.model_name,
            "messages": request.messages,
            "max_tokens": request.max_tokens.unwrap_or(model_config.max_tokens),
            "temperature": request.temperature.unwrap_or(model_config.temperature)
        });

        let response = self
            .http_client
            .post(&model_config.api_url)
            .headers(headers)
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("Google API错误: {}", error_text));
        }

        let response_json: Value = response.json().await?;
        self.parse_openai_response(response_json, &model_config.model_name) // 使用OpenAI格式解析
    }

    /// 调用百度API
    async fn call_baidu_api(
        &self,
        request: AIRequest,
        model_config: &AIModelConfig,
    ) -> Result<AIResponse> {
        let api_key = self
            .config
            .get_api_key(&AIProvider::Baidu)
            .ok_or_else(|| anyhow!("未配置百度API密钥"))?;

        // 百度需要先获取access_token
        let access_token = self.get_baidu_access_token(api_key).await?;

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        let payload = json!({
            "messages": request.messages,
            "temperature": request.temperature.unwrap_or(model_config.temperature),
            "max_output_tokens": request.max_tokens.unwrap_or(model_config.max_tokens)
        });

        let url = format!("{}?access_token={}", model_config.api_url, access_token);

        let response = self
            .http_client
            .post(&url)
            .headers(headers)
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("百度API错误: {}", error_text));
        }

        let response_json: Value = response.json().await?;
        self.parse_baidu_response(response_json, &model_config.model_name)
    }

    /// 调用阿里云API
    async fn call_alibaba_api(
        &self,
        request: AIRequest,
        model_config: &AIModelConfig,
    ) -> Result<AIResponse> {
        let api_key = self
            .config
            .get_api_key(&AIProvider::Alibaba)
            .ok_or_else(|| anyhow!("未配置阿里云API密钥"))?;

        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", api_key))?,
        );
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        let payload = json!({
            "model": model_config.model_name,
            "input": {
                "messages": request.messages
            },
            "parameters": {
                "max_tokens": request.max_tokens.unwrap_or(model_config.max_tokens),
                "temperature": request.temperature.unwrap_or(model_config.temperature)
            }
        });

        let response = self
            .http_client
            .post(&model_config.api_url)
            .headers(headers)
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("阿里云API错误: {}", error_text));
        }

        let response_json: Value = response.json().await?;
        self.parse_alibaba_response(response_json, &model_config.model_name)
    }

    /// 调用硅基流动API (使用OpenAI兼容接口)
    async fn call_siliconflow_api(
        &self,
        request: AIRequest,
        model_config: &AIModelConfig,
    ) -> Result<AIResponse> {
        let api_key = self
            .config
            .get_api_key(&AIProvider::SiliconFlow)
            .ok_or_else(|| anyhow!("未配置硅基流动API密钥"))?;

        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", api_key))?,
        );
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        let payload = json!({
            "model": model_config.model_name,
            "messages": request.messages,
            "max_tokens": request.max_tokens.unwrap_or(model_config.max_tokens),
            "temperature": request.temperature.unwrap_or(model_config.temperature),
            "stream": request.stream.unwrap_or(false)
        });

        let response = self
            .http_client
            .post(&model_config.api_url)
            .headers(headers)
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("硅基流动API错误: {}", error_text));
        }

        let response_json: Value = response.json().await?;
        self.parse_openai_response(response_json, &model_config.model_name) // 使用OpenAI格式解析
    }

    /// 调用本地API (Ollama)
    async fn call_local_api(
        &self,
        request: AIRequest,
        model_config: &AIModelConfig,
    ) -> Result<AIResponse> {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        let payload = json!({
            "model": model_config.model_name,
            "messages": request.messages,
            "stream": false,
            "options": {
                "temperature": request.temperature.unwrap_or(model_config.temperature),
                "num_predict": request.max_tokens.unwrap_or(model_config.max_tokens)
            }
        });

        let response = self
            .http_client
            .post(&model_config.api_url)
            .headers(headers)
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("本地API错误: {}", error_text));
        }

        let response_json: Value = response.json().await?;
        self.parse_ollama_response(response_json, &model_config.model_name)
    }

    /// 获取百度access_token
    async fn get_baidu_access_token(&self, api_key: &str) -> Result<String> {
        // 这里需要实现百度的OAuth流程
        // 简化实现，实际应该缓存token
        let url = format!(
            "https://aip.baidubce.com/oauth/2.0/token?grant_type=client_credentials&client_id={}&client_secret={}",
            api_key, api_key // 实际应该分别是client_id和client_secret
        );

        let response = self.http_client.post(&url).send().await?;
        let response_json: Value = response.json().await?;

        response_json["access_token"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| anyhow!("无法获取百度access_token"))
    }

    /// 解析OpenAI格式响应
    fn parse_openai_response(&self, response: Value, model: &str) -> Result<AIResponse> {
        let content = response["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| anyhow!("无法解析响应内容"))?
            .to_string();

        let usage = response["usage"].as_object().map(|usage_obj| AIUsage {
            prompt_tokens: usage_obj["prompt_tokens"].as_u64().unwrap_or(0) as u32,
            completion_tokens: usage_obj["completion_tokens"].as_u64().unwrap_or(0) as u32,
            total_tokens: usage_obj["total_tokens"].as_u64().unwrap_or(0) as u32,
        });

        let finish_reason = response["choices"][0]["finish_reason"]
            .as_str()
            .map(|s| s.to_string());

        Ok(AIResponse {
            content,
            model: model.to_string(),
            usage,
            finish_reason,
        })
    }

    /// 解析Anthropic格式响应
    fn parse_anthropic_response(&self, response: Value, model: &str) -> Result<AIResponse> {
        let content = response["content"][0]["text"]
            .as_str()
            .ok_or_else(|| anyhow!("无法解析Anthropic响应内容"))?
            .to_string();

        let usage = response["usage"].as_object().map(|usage_obj| AIUsage {
            prompt_tokens: usage_obj["input_tokens"].as_u64().unwrap_or(0) as u32,
            completion_tokens: usage_obj["output_tokens"].as_u64().unwrap_or(0) as u32,
            total_tokens: (usage_obj["input_tokens"].as_u64().unwrap_or(0)
                + usage_obj["output_tokens"].as_u64().unwrap_or(0))
                as u32,
        });

        let finish_reason = response["stop_reason"].as_str().map(|s| s.to_string());

        Ok(AIResponse {
            content,
            model: model.to_string(),
            usage,
            finish_reason,
        })
    }

    /// 解析百度格式响应
    fn parse_baidu_response(&self, response: Value, model: &str) -> Result<AIResponse> {
        let content = response["result"]
            .as_str()
            .ok_or_else(|| anyhow!("无法解析百度响应内容"))?
            .to_string();

        let usage = response["usage"].as_object().map(|usage_obj| AIUsage {
            prompt_tokens: usage_obj["prompt_tokens"].as_u64().unwrap_or(0) as u32,
            completion_tokens: usage_obj["completion_tokens"].as_u64().unwrap_or(0) as u32,
            total_tokens: usage_obj["total_tokens"].as_u64().unwrap_or(0) as u32,
        });

        Ok(AIResponse {
            content,
            model: model.to_string(),
            usage,
            finish_reason: Some("stop".to_string()),
        })
    }

    /// 解析阿里云格式响应
    fn parse_alibaba_response(&self, response: Value, model: &str) -> Result<AIResponse> {
        let content = response["output"]["text"]
            .as_str()
            .ok_or_else(|| anyhow!("无法解析阿里云响应内容"))?
            .to_string();

        let usage = response["usage"].as_object().map(|usage_obj| AIUsage {
            prompt_tokens: usage_obj["input_tokens"].as_u64().unwrap_or(0) as u32,
            completion_tokens: usage_obj["output_tokens"].as_u64().unwrap_or(0) as u32,
            total_tokens: usage_obj["total_tokens"].as_u64().unwrap_or(0) as u32,
        });

        let finish_reason = response["output"]["finish_reason"]
            .as_str()
            .map(|s| s.to_string());

        Ok(AIResponse {
            content,
            model: model.to_string(),
            usage,
            finish_reason,
        })
    }

    /// 解析Ollama格式响应
    fn parse_ollama_response(&self, response: Value, model: &str) -> Result<AIResponse> {
        let content = response["message"]["content"]
            .as_str()
            .ok_or_else(|| anyhow!("无法解析Ollama响应内容"))?
            .to_string();

        Ok(AIResponse {
            content,
            model: model.to_string(),
            usage: None, // Ollama通常不返回token使用统计
            finish_reason: Some("stop".to_string()),
        })
    }

    /// 获取当前配置
    #[allow(dead_code)]
    pub fn get_config(&self) -> &AIConfig {
        &self.config
    }

    /// 更新配置
    #[allow(dead_code)]
    pub fn update_config(&mut self, config: AIConfig) -> Result<()> {
        self.config = config;

        // 重新创建HTTP客户端以应用新的超时设置
        self.http_client = Client::builder()
            .timeout(Duration::from_secs(self.config.timeout_seconds))
            .build()?;

        Ok(())
    }
}
