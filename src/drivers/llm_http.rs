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

        let python = std::env::var("PYTHON3").unwrap_or_else(|_| "python3".to_string());

        // tokio::process + wait_with_output() — auto-drains pipes, no macOS deadlock
        let mut child = tokio::process::Command::new(&python)
            .arg("-u")  // unbuffered stdout — ensures pipe doesn't stall
            .arg(&script)
            .arg(&tmp_path)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| DriverError::NetworkFracture(format!("spawn: {}", e)))?;

        info!("[Driver {}] python3 spawned PID={}", agent_id, child.id().unwrap_or(0));

        let output_result = tokio::time::timeout(
            tokio::time::Duration::from_secs(120),
            child.wait_with_output()
        ).await;

        let output = match output_result {
            Ok(Ok(out)) => out,
            Ok(Err(e)) => {
                warn!("[Driver {}] wait error: {}", agent_id, e);
                return Err(DriverError::NetworkFracture(format!("wait: {}", e)));
            }
            Err(_) => {
                // child consumed by wait_with_output — timeout auto-drops and kills it
                warn!("[Driver {}] Python timeout (120s)", agent_id);
                return Err(DriverError::Timeout);
            }
        };

        let result = if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            warn!("[Driver {}] python failed (exit {}): {}", agent_id, output.status, stderr.trim());
            Err(DriverError::BackendError(stderr.trim().to_string()))
        } else {
            let content = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if content.is_empty() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                error!("[Driver {}] python empty. stderr: {}", agent_id, stderr.trim());
                Err(DriverError::JsonParseError)
            } else {
                Ok(content)
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
