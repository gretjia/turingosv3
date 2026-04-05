use std::time::Duration;
use serde_json::json;
use log::{error, warn, info};

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
    use_curl: bool, // true for cloud APIs, false for local llama.cpp
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
        let is_local = api_url.contains("127.0.0.1") || api_url.contains("localhost") || api_url.contains("192.168.");
        Self {
            api_url: api_url.to_string(),
            model_name: model_name.to_string(),
            api_key,
            use_curl: !is_local,
        }
    }

    pub fn with_key(api_url: &str, model_name: &str, api_key: &str) -> Self {
        let is_local = api_url.contains("127.0.0.1") || api_url.contains("localhost") || api_url.contains("192.168.");
        Self {
            api_url: api_url.to_string(),
            model_name: model_name.to_string(),
            api_key: Some(api_key.to_string()),
            use_curl: !is_local,
        }
    }

    pub fn model_name(&self) -> &str { &self.model_name }

    pub async fn resilient_generate(&self, prompt: &str, agent_id: usize, temperature: f32) -> Result<String, DriverError> {
        let is_qwen3 = self.model_name.contains("qwen3");
        let is_dashscope = self.api_url.contains("dashscope");

        let thinking_mode = std::env::var("THINKING_MODE").unwrap_or_else(|_| "off".to_string());

        let (system_prefix, max_tok) = if is_qwen3 {
            match thinking_mode.as_str() {
                "on" => ("", 3072_u32),
                s if s.starts_with("budget:") => {
                    let n: u32 = s[7..].parse().unwrap_or(1500);
                    ("", n)
                }
                _ => ("/no_think\n", 3072_u32),
            }
        } else {
            ("", 3072)
        };

        let system_msg = format!(
            "{}You are a reasoning agent. Follow all formatting instructions exactly.",
            system_prefix
        );

        // DashScope Qwen3 open-source: must stream. Others: non-streaming OK.
        let force_stream = is_qwen3 && is_dashscope;

        let mut payload = json!({
            "model": self.model_name,
            "messages": [
                { "role": "system", "content": system_msg },
                { "role": "user", "content": prompt }
            ],
            "temperature": temperature,
            "max_tokens": max_tok,
            "stream": force_stream
        });

        if is_qwen3 && is_dashscope && thinking_mode == "off" {
            payload["enable_thinking"] = json!(false);
        }

        if self.use_curl {
            self.call_via_curl(&payload, agent_id).await
        } else {
            self.call_via_reqwest(&payload, agent_id).await
        }
    }

    /// Cloud APIs: use curl subprocess (reqwest + rustls hangs on Chinese HTTPS endpoints)
    async fn call_via_curl(&self, payload: &serde_json::Value, agent_id: usize) -> Result<String, DriverError> {
        let payload_str = serde_json::to_string(payload).map_err(|e| DriverError::JsonParseError)?;

        let mut args = vec![
            "-s".to_string(),
            "--connect-timeout".to_string(), "30".to_string(),
            "-m".to_string(), "300".to_string(),
            "-X".to_string(), "POST".to_string(),
            self.api_url.clone(),
            "-H".to_string(), "Content-Type: application/json".to_string(),
        ];

        if let Some(ref key) = self.api_key {
            args.push("-H".to_string());
            args.push(format!("Authorization: Bearer {}", key));
        }

        args.push("-d".to_string());
        args.push(payload_str);

        let output = tokio::process::Command::new("curl")
            .args(&args)
            .output()
            .await
            .map_err(|e| DriverError::NetworkFracture(format!("curl exec failed: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            warn!("[Driver {}] curl failed (exit {}): {}", agent_id, output.status, stderr);
            return Err(DriverError::NetworkFracture(format!("curl exit {}", output.status)));
        }

        let body = String::from_utf8_lossy(&output.stdout);

        // Handle streaming SSE response (DashScope Qwen3)
        if payload.get("stream").and_then(|v| v.as_bool()) == Some(true) {
            let mut content = String::new();
            for line in body.lines() {
                if let Some(data) = line.strip_prefix("data: ") {
                    if data == "[DONE]" { break; }
                    if let Ok(chunk) = serde_json::from_str::<serde_json::Value>(data) {
                        if let Some(delta) = chunk.get("choices")
                            .and_then(|c| c.get(0))
                            .and_then(|c| c.get("delta")) {
                            if let Some(c) = delta.get("content").and_then(|v| v.as_str()) {
                                content.push_str(c);
                            }
                            if let Some(r) = delta.get("reasoning_content").and_then(|v| v.as_str()) {
                                content.push_str(r);
                            }
                        }
                    }
                }
            }
            if !content.is_empty() {
                return Ok(content);
            }
            error!("[Driver {}] SSE stream empty", agent_id);
            return Err(DriverError::JsonParseError);
        }

        // Non-streaming JSON response
        self.parse_json_response(&body, agent_id)
    }

    /// Local llama.cpp: use reqwest (works for local HTTP, no TLS issues)
    async fn call_via_reqwest(&self, payload: &serde_json::Value, agent_id: usize) -> Result<String, DriverError> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(1200))
            .build()
            .map_err(|e| DriverError::NetworkFracture(format!("reqwest build: {}", e)))?;

        let mut req = client.post(&self.api_url).json(payload);
        if let Some(ref key) = self.api_key {
            req = req.header("Authorization", format!("Bearer {}", key));
        }

        match req.send().await {
            Ok(response) if response.status().is_success() => {
                let body = response.text().await.unwrap_or_default();
                self.parse_json_response(&body, agent_id)
            }
            Ok(response) => {
                let status = response.status();
                let body = response.text().await.unwrap_or_default();
                warn!("[Driver {}] API Error (HTTP {}): {}", agent_id, status, &body[..body.len().min(300)]);
                Err(DriverError::BackendError(format!("HTTP {}", status)))
            }
            Err(e) => {
                if e.is_timeout() {
                    warn!("[Driver {}] Network Timeout.", agent_id);
                    Err(DriverError::Timeout)
                } else {
                    warn!("[Driver {}] Network I/O Fracture: {}", agent_id, e);
                    Err(DriverError::NetworkFracture(e.to_string()))
                }
            }
        }
    }

    fn parse_json_response(&self, body: &str, agent_id: usize) -> Result<String, DriverError> {
        let json_body: serde_json::Value = serde_json::from_str(body)
            .map_err(|_| {
                error!("[Driver {}] JSON parse fault: {}", agent_id, &body[..body.len().min(200)]);
                DriverError::JsonParseError
            })?;

        // Check for API error
        if let Some(err) = json_body.get("error") {
            let msg = err.get("message").and_then(|v| v.as_str()).unwrap_or("unknown");
            warn!("[Driver {}] API Error: {}", agent_id, msg);
            return Err(DriverError::BackendError(msg.to_string()));
        }

        let mut final_content = String::new();
        if let Some(choices) = json_body.get("choices") {
            let message = &choices[0]["message"];
            if let Some(reasoning) = message.get("reasoning_content").and_then(|v| v.as_str()) {
                final_content.push_str(reasoning);
                final_content.push('\n');
            }
            if let Some(content) = message.get("content").and_then(|v| v.as_str()) {
                final_content.push_str(content);
            }
        }

        if !final_content.is_empty() {
            Ok(final_content)
        } else {
            error!("[Driver {}] No content in response: {}", agent_id, &body[..body.len().min(200)]);
            Err(DriverError::JsonParseError)
        }
    }
}
