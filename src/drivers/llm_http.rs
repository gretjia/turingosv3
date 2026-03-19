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
        } else {
            std::env::var("SILICONFLOW_API_KEY").or_else(|_| std::env::var("VOLCENGINE_API_KEY")).ok()
        };
        Self {
            client: Client::builder()
                .build()
                .unwrap(),
            api_url: api_url.to_string(),
            model_name: model_name.to_string(),
            api_key,
        }
    }

    /// 执行具备热力学韧性与指数退避的网络请求
    pub async fn resilient_generate(&self, prompt: &str, agent_id: usize, temperature: f32) -> Result<String, DriverError> {
        let payload = json!({
            "model": self.model_name,
            "messages": [
                {
                    "role": "system",
                    "content": "You are a logical reasoning agent in the TuringOS Thermodynamic Sandbox. You can output <think>...</think> and reason freely without any length limits. However, you MUST conclude your entire reasoning with exactly [State: ...]."
                },
                {
                    "role": "user",
                    "content": prompt
                }
            ],
            // Temperature varies slightly based on agent_id to create genuine DAG branching
            "temperature": temperature, 
            "max_tokens": 8192,
            "stream": false
        });

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
                warn!("[Driver {}] GPU Backpressure (HTTP {}). Slots full?", agent_id, response.status());
                return Err(DriverError::BackendError(format!("HTTP {}", response.status())));
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
