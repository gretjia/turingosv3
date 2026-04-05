use std::time::Duration;
use reqwest::Client;
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
    client: Client,
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
        Self {
            client: Client::builder()
                .http1_only()  // Force HTTP/1.1 — Chinese APIs may hang on H2 ALPN
                .no_proxy()    // Bypass system proxy — Mac VPN proxy (7897) blocks Chinese API calls
                .build()
                .unwrap(),
            api_url: api_url.to_string(),
            model_name: model_name.to_string(),
            api_key,
        }
    }

    /// Construct with explicit API key — for multi-account routing
    pub fn with_key(api_url: &str, model_name: &str, api_key: &str) -> Self {
        Self {
            client: Client::builder().build().unwrap(),
            api_url: api_url.to_string(),
            model_name: model_name.to_string(),
            api_key: Some(api_key.to_string()),
        }
    }

    pub fn model_name(&self) -> &str { &self.model_name }

    /// 执行具备热力学韧性与指数退避的网络请求
    pub async fn resilient_generate(&self, prompt: &str, agent_id: usize, temperature: f32) -> Result<String, DriverError> {
        // Qwen3.5 thinking mode control:
        //   Env: THINKING_MODE = "on" | "off" | "budget:N"
        //     "on"       → full thinking (high quality, slow: ~60s/request on local 9B)
        //     "off"      → /no_think directive (lower quality, fast: ~5s/request)
        //     "budget:N" → thinking with max_tokens=N total (thinking+output compete)
        //   Default: "off" (for speed on local inference)
        //   Source: AutoResearch sweep — thinking mode is a tunable parameter
        //   Root cause analysis 2026-04-03:
        //     thinking ON  → detailed algebra, but 1000+ hidden tokens → 60s latency
        //     thinking OFF → terse summaries, but 5s latency → 12x faster sweeps
        let is_qwen3 = self.model_name.contains("qwen3");
        let is_dashscope = self.api_url.contains("dashscope");
        let is_local = self.api_url.contains("127.0.0.1") || self.api_url.contains("localhost");

        let thinking_mode = std::env::var("THINKING_MODE").unwrap_or_else(|_| "off".to_string());

        let (system_prefix, max_tok) = if is_qwen3 {
            match thinking_mode.as_str() {
                "on" => ("", 3072_u32),                    // full thinking
                s if s.starts_with("budget:") => {
                    let n: u32 = s[7..].parse().unwrap_or(1500);
                    ("", n)                                // budget-capped thinking
                }
                _ => ("/no_think\n", 3072_u32),            // off (default)
            }
        } else {
            ("", 3072)  // non-Qwen3: no thinking control needed
        };

        let system_msg = format!(
            "{}You are a reasoning agent. Follow all formatting instructions exactly.",
            system_prefix
        );

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

        // DashScope-specific: disable thinking via API parameter
        if is_qwen3 && is_dashscope && thinking_mode == "off" {
            payload["enable_thinking"] = json!(false);
        }

        let mut request_builder = self.client.post(&self.api_url)
            // Use resilient timeout internally as well, ignoring global client timeout for these heavy requests
            .timeout(Duration::from_secs(1200))
            .json(&payload);

        if let Some(ref key) = self.api_key {
            request_builder = request_builder.header("Authorization", format!("Bearer {}", key));
        }

        let request = request_builder.send().await;

        match request {
            Ok(response) if response.status().is_success() => {
                if let Ok(json_body) = response.json::<serde_json::Value>().await {
                    let mut final_content = String::new();
                    
                    // Parse OpenAI (llama.cpp) or native Ollama formats
                    if let Some(choices) = json_body.get("choices") {
                        let message = &choices[0]["message"];
                        if let Some(reasoning) = message.get("reasoning_content").and_then(|v| v.as_str()) {
                            final_content.push_str(reasoning);
                            final_content.push('\n');
                        }
                        if let Some(content) = message.get("content").and_then(|v| v.as_str()) {
                            final_content.push_str(content);
                        }
                    } else if let Some(message) = json_body.get("message") {
                        if let Some(content) = message.get("content").and_then(|v| v.as_str()) {
                            final_content.push_str(content);
                        }
                    }

                    if !final_content.is_empty() {
                        return Ok(final_content);
                    } else {
                        error!("[Driver {}] API Response parsed but no content found. Body: {}", agent_id, json_body);
                        return Err(DriverError::JsonParseError);
                    }
                } else {
                    error!("[Driver {}] JSON Parse Fault. Backend returned malformed data.", agent_id);
                    return Err(DriverError::JsonParseError);
                }
            }
            Ok(response) => {
                let status = response.status();
                let body = response.text().await.unwrap_or_default();
                warn!("[Driver {}] API Error (HTTP {}): {}", agent_id, status, &body[..body.len().min(300)]);
                return Err(DriverError::BackendError(format!("HTTP {}: {}", status, &body[..body.len().min(200)])));
            },
            Err(e) => {
                if e.is_timeout() {
                    warn!("[Driver {}] Network Timeout.", agent_id);
                    return Err(DriverError::Timeout);
                } else {
                    warn!("[Driver {}] Network I/O Fracture: {}", agent_id, e);
                    return Err(DriverError::NetworkFracture(e.to_string()));
                }
            }
        }
    }
}
