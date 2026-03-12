use turingosv3::kernel::{AIBlackBox, Input, Output, Action, MachineState};
use reqwest::blocking::Client;
use serde_json::json;
use log::{info, error, debug};
use std::time::Duration;

pub struct NetworkedHanoiAgent {
    pub api_url: String,
    pub model_name: String,
    pub client: Client,
    pub current_step: u64,
    pub total_steps: u64,
}

impl NetworkedHanoiAgent {
    pub fn new(api_url: &str, model_name: &str, total_steps: u64) -> Self {
        NetworkedHanoiAgent {
            api_url: api_url.to_string(),
            model_name: model_name.to_string(),
            client: Client::builder()
                .timeout(Duration::from_secs(120)) // Deep reasoning might take time
                .build()
                .unwrap(),
            current_step: 0,
            total_steps,
        }
    }

    fn call_llm(&self, prompt: &str) -> Option<String> {
        let payload = json!({
            "model": self.model_name,
            "messages": [
                {
                    "role": "system",
                    "content": "You are an agent executing the Tower of Hanoi MAKER protocol. You must strictly output the next move and only the next move, in the format 'Move disk X from Y to Z'."
                },
                {
                    "role": "user",
                    "content": prompt
                }
            ],
            "temperature": 0.0, // Strictly deterministic for now
            "max_tokens": 100
        });

        match self.client.post(&self.api_url)
            .json(&payload)
            .send() {
            Ok(response) => {
                if response.status().is_success() {
                    if let Ok(json_body) = response.json::<serde_json::Value>() {
                        if let Some(content) = json_body["choices"][0]["message"]["content"].as_str() {
                            return Some(content.trim().to_string());
                        }
                    }
                } else {
                    error!("LLM API Error: {:?}", response.status());
                }
            }
            Err(e) => {
                error!("Network failure communicating with {}: {}", self.api_url, e);
            }
        }
        None
    }
}

impl AIBlackBox for NetworkedHanoiAgent {
    fn delta(&mut self, _input: &Input) -> Output {
        self.current_step += 1;
        info!(">>> Agent computing Hanoi Step {}/{} via Network LLM...", self.current_step, self.total_steps);

        let q_o = if self.current_step >= self.total_steps {
            MachineState::Halt
        } else {
            MachineState::Running
        };

        // Construct context-aware prompt based on the previous steps or current state.
        let prompt = format!("Provide the single action for Step {} of the 20-disk Tower of Hanoi problem.", self.current_step);
        
        let llm_payload = match self.call_llm(&prompt) {
            Some(answer) => {
                debug!("LLM Output: {}", answer);
                answer
            },
            None => {
                error!("LLM Failed to generate valid output. Submitting failure state.");
                "LLM_NETWORK_ERROR_OR_HALLUCINATION".to_string()
            }
        };

        let mut citations = vec![];
        if self.current_step > 1 {
            citations.push(format!("hanoi_step_{}", self.current_step - 1));
        }

        Output {
            q_o,
            a_o: Action {
                file_id: format!("hanoi_step_{}", self.current_step),
                author: format!("NetworkAgent_{}", self.model_name),
                payload: llm_payload,
                citations,
                stake: 1, // Will burn if LLM produced garbage that the kernel rejects
            }
        }
    }
}
