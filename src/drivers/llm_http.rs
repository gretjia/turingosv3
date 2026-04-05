use std::time::Duration;
use serde_json::json;
use log::{error, warn};

#[derive(Debug)]
pub enum DriverError {
    NetworkFracture(String),
    Timeout,
    JsonParseError,
    BackendError(String),
}

pub struct ResilientLLMClient {
    api_url: String,
    model_name: String,
    api_key: Option<String>,
}

impl ResilientLLMClient {
    pub fn new(api_url: &str, model_name: &str, _timeout_secs: u64) -> Self {
        let api_key = if api_url.contains("volcengine") || api_url.contains("volces.com") {
            std::env::var("VOLCENGINE_API_KEY").or_else(|_| std::env::var("SILICONFLOW_API_KEY")).ok()
        } else if api_url.contains("deepseek.com") {
            std::env::var("DEEPSEEK_API_KEY").ok()
        } else {
            std::env::var("SILICONFLOW_API_KEY").or_else(|_| std::env::var("VOLCENGINE_API_KEY")).ok()
        };
        Self { api_url: api_url.to_string(), model_name: model_name.to_string(), api_key }
    }

    pub fn with_key(api_url: &str, model_name: &str, api_key: &str) -> Self {
        Self {
            api_url: api_url.to_string(),
            model_name: model_name.to_string(),
            api_key: Some(api_key.to_string()),
        }
    }

    pub fn model_name(&self) -> &str { &self.model_name }

    /// All calls go through HTTP (either local llama-server or local llm_proxy.py).
    /// No HTTPS, no TLS, no Chinese API issues — reqwest works perfectly over HTTP.
    pub async fn resilient_generate(&self, prompt: &str, agent_id: usize, temperature: f32) -> Result<String, DriverError> {
        let is_qwen3 = self.model_name.contains("qwen3");
        let is_dashscope = self.api_url.contains("dashscope");
        let thinking_mode = std::env::var("THINKING_MODE").unwrap_or_else(|_| "off".to_string());

        let (system_prefix, max_tok) = if is_qwen3 {
            match thinking_mode.as_str() {
                "on" => ("", 3072_u32),
                s if s.starts_with("budget:") => ("", s[7..].parse().unwrap_or(1500)),
                _ => ("/no_think\n", 3072_u32),
            }
        } else {
            ("", 3072)
        };

        let system_msg = format!("{}You are a reasoning agent. Follow all formatting instructions exactly.", system_prefix);

        let mut payload = json!({
            "model": self.model_name,
            "messages": [
                { "role": "system", "content": system_msg },
                { "role": "user", "content": prompt }
            ],
            "temperature": temperature,
            "max_tokens": max_tok,
            "stream": false
        });

        // DashScope Qwen3 via proxy: proxy handles streaming internally
        if is_qwen3 && is_dashscope && thinking_mode == "off" {
            payload["enable_thinking"] = json!(false);
        }

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(120))
            .build()
            .map_err(|e| DriverError::NetworkFracture(format!("client: {}", e)))?;

        let mut req = client.post(&self.api_url).json(&payload);
        if let Some(ref key) = self.api_key {
            req = req.header("Authorization", format!("Bearer {}", key));
        }

        match req.send().await {
            Ok(response) if response.status().is_success() => {
                match response.json::<serde_json::Value>().await {
                    Ok(body) => {
                        // Check for API error in body
                        if let Some(err) = body.get("error") {
                            let msg = err.get("message").and_then(|v| v.as_str()).unwrap_or("unknown");
                            warn!("[Driver {}] API error: {}", agent_id, msg);
                            return Err(DriverError::BackendError(msg.to_string()));
                        }

                        let mut content = String::new();
                        if let Some(choices) = body.get("choices") {
                            let msg = &choices[0]["message"];
                            if let Some(r) = msg.get("reasoning_content").and_then(|v| v.as_str()) {
                                if !r.is_empty() { content.push_str(r); content.push('\n'); }
                            }
                            if let Some(c) = msg.get("content").and_then(|v| v.as_str()) {
                                content.push_str(c);
                            }
                        }

                        if !content.is_empty() {
                            Ok(content)
                        } else {
                            error!("[Driver {}] Empty response: {}", agent_id, &body.to_string()[..body.to_string().len().min(200)]);
                            Err(DriverError::JsonParseError)
                        }
                    }
                    Err(e) => {
                        error!("[Driver {}] JSON parse: {}", agent_id, e);
                        Err(DriverError::JsonParseError)
                    }
                }
            }
            Ok(response) => {
                let status = response.status();
                let body = response.text().await.unwrap_or_default();
                warn!("[Driver {}] HTTP {}: {}", agent_id, status, &body[..body.len().min(300)]);
                Err(DriverError::BackendError(format!("HTTP {}", status)))
            }
            Err(e) => {
                if e.is_timeout() {
                    warn!("[Driver {}] Timeout", agent_id);
                    Err(DriverError::Timeout)
                } else {
                    warn!("[Driver {}] Network: {}", agent_id, e);
                    Err(DriverError::NetworkFracture(e.to_string()))
                }
            }
        }
    }
}
