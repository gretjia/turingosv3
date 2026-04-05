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
            self.call_via_python(&payload, agent_id).await
        } else {
            self.call_via_reqwest(&payload, agent_id).await
        }
    }

    /// Cloud APIs: use Python OpenAI SDK (reqwest hangs on Chinese HTTPS, curl SSE unreliable)
    async fn call_via_python(&self, payload: &serde_json::Value, agent_id: usize) -> Result<String, DriverError> {
        // Determine provider from URL
        let provider = if self.api_url.contains("dashscope") { "aliyun" }
            else if self.api_url.contains("siliconflow") { "siliconflow" }
            else if self.api_url.contains("deepseek") { "deepseek" }
            else if self.api_url.contains("volces") || self.api_url.contains("volcengine") { "volcengine" }
            else if self.api_url.contains("nvidia") { "nvidia" }
            else { "aliyun" };

        let thinking_mode = std::env::var("THINKING_MODE").unwrap_or_else(|_| "off".to_string());

        let py_input = json!({
            "provider": provider,
            "model": self.model_name,
            "prompt": payload["messages"][1]["content"],
            "system": payload["messages"][0]["content"],
            "temperature": payload["temperature"],
            "max_tokens": payload["max_tokens"],
            "enable_thinking": thinking_mode == "on",
        });

        // Find llm_call.py relative to binary or in known locations
        let script = std::env::var("LLM_CALL_PY").unwrap_or_else(|_| {
            let candidates = [
                "src/drivers/llm_call.py",
                "../src/drivers/llm_call.py",
                "llm_call.py",
            ];
            for c in &candidates {
                if std::path::Path::new(c).exists() { return c.to_string(); }
            }
            "src/drivers/llm_call.py".to_string()
        });

        info!("[Driver {}] python → {} via {} (model={})", agent_id, provider, script, self.model_name);

        // Synchronous try_wait polling — avoids tokio pipe/async deadlocks
        let tmp_path = format!("/tmp/llm_call_{}.json", agent_id);
        std::fs::write(&tmp_path, serde_json::to_string(&py_input).unwrap_or_default())
            .map_err(|e| DriverError::NetworkFracture(format!("write: {}", e)))?;

        use std::io::Read;
        // Use absolute path — Mac's homebrew python3 may not be in PATH for child processes
        let python = std::env::var("PYTHON3").unwrap_or_else(|_| "python3".to_string());
        let mut child = std::process::Command::new(&python)
            .arg(&script)
            .arg(&tmp_path)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| DriverError::NetworkFracture(format!("python3 spawn: {}", e)))?;

        let start = std::time::Instant::now();
        let timeout = std::time::Duration::from_secs(120);

        let result = loop {
            match child.try_wait() {
                Ok(Some(status)) => {
                    let mut stdout_buf = Vec::new();
                    let mut stderr_buf = Vec::new();
                    if let Some(mut out) = child.stdout.take() { let _ = out.read_to_end(&mut stdout_buf); }
                    if let Some(mut err) = child.stderr.take() { let _ = err.read_to_end(&mut stderr_buf); }

                    if !status.success() {
                        let stderr = String::from_utf8_lossy(&stderr_buf);
                        warn!("[Driver {}] python failed (exit {}): {}", agent_id, status, stderr.trim());
                        break Err(DriverError::BackendError(stderr.trim().to_string()));
                    }

                    let content = String::from_utf8_lossy(&stdout_buf).trim().to_string();
                    if content.is_empty() {
                        let stderr = String::from_utf8_lossy(&stderr_buf);
                        error!("[Driver {}] python empty. stderr: {}", agent_id, stderr.trim());
                        break Err(DriverError::JsonParseError);
                    }
                    break Ok(content);
                }
                Ok(None) => {
                    if start.elapsed() > timeout {
                        let _ = child.kill();
                        warn!("[Driver {}] Python timeout (120s)", agent_id);
                        break Err(DriverError::Timeout);
                    }
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
                Err(e) => {
                    break Err(DriverError::NetworkFracture(format!("poll: {}", e)));
                }
            }
        };

        let _ = std::fs::remove_file(&tmp_path);
        result
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
