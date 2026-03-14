use std::time::Duration;
use reqwest::Client;
use serde_json::json;
use log::{info, error, warn};
use tokio::time::sleep;

pub struct ResilientLLMClient {
    client: Client,
    api_url: String,
    model_name: String,
}

impl ResilientLLMClient {
    pub fn new(api_url: &str, model_name: &str, timeout_secs: u64) -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(timeout_secs))
                .build()
                .unwrap(),
            api_url: api_url.to_string(),
            model_name: model_name.to_string(),
        }
    }

    /// 执行具备热力学韧性与指数退避的网络请求
    pub async fn resilient_generate(&self, prompt: &str, agent_id: usize, max_retries: u32) -> Option<String> {
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
            "temperature": 0.1 + (agent_id as f32 * 0.2), 
            "max_tokens": 8192,
            "stream": false
        });

        let mut backoff_secs = 5;

        for attempt in 1..=max_retries {
            let request = self.client.post(&self.api_url)
                // Use resilient timeout internally as well, ignoring global client timeout for these heavy requests
                .timeout(Duration::from_secs(1200))
                .json(&payload)
                .send()
                .await;

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
                            return Some(final_content);
                        } else {
                            error!("[Driver {}] API Response parsed but no content found. Body: {}", agent_id, json_body);
                        }
                    } else {
                        error!("[Driver {}] JSON Parse Fault. Backend returned malformed data.", agent_id);
                    }
                }
                Ok(response) => warn!("[Driver {}] GPU Backpressure (HTTP {}). Slots full? attempt: {}", agent_id, response.status(), attempt),
                Err(e) => warn!("[Driver {}] Network I/O Fracture: {} on attempt {}", agent_id, e, attempt),
            }

            // 物理法则的妥协：触发时空膨胀，错峰让出 GPU VRAM
            if attempt < max_retries {
                info!("[Driver {}] Spacetime Dilation: Sleeping for {}s...", agent_id, backoff_secs);
                sleep(Duration::from_secs(backoff_secs)).await;
                backoff_secs *= 2; // 指数衰减
            }
        }

        error!("FATAL: [Driver {}] completely starved after {} retries.", agent_id, max_retries);
        None
    }
}
