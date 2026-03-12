use turingosv3::kernel::{AIBlackBox, Input, Output, Action, MachineState};
use reqwest::Client;
use serde_json::json;
use log::{info, error, debug};
use std::time::Duration;
use tokio::runtime::Runtime;
use tokio::task::JoinSet;

pub struct SpeculativeSwarmAgent {
    pub api_url: String,
    pub model_name: String,
    pub http_client: Client,
    pub current_step: u64,
    pub total_steps: u64,
    pub swarm_size: usize,
    pub rt: Runtime,
}

impl SpeculativeSwarmAgent {
    pub fn new(api_url: &str, model_name: &str, total_steps: u64, swarm_size: usize) -> Self {
        SpeculativeSwarmAgent {
            api_url: api_url.to_string(),
            model_name: model_name.to_string(),
            http_client: Client::builder()
                .timeout(Duration::from_secs(120))
                .build()
                .unwrap(),
            current_step: 0,
            total_steps,
            swarm_size,
            rt: Runtime::new().unwrap(),
        }
    }

    async fn call_llm_async(client: Client, url: String, model: String, prompt: String, agent_id: usize) -> Option<(usize, String)> {
        let payload = json!({
            "model": model,
            "messages": [
                {
                    "role": "system",
                    "content": "You are a Hanoi MAKER agent. Strictly output the next move format: 'Move disk X from Y to Z'."
                },
                {
                    "role": "user",
                    "content": prompt
                }
            ],
            // Slightly vary temperature so the swarm explores different generation paths
            "temperature": 0.1 + (agent_id as f32 * 0.1), 
            "max_tokens": 100
        });

        match client.post(&url).json(&payload).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    if let Ok(json_body) = response.json::<serde_json::Value>().await {
                        if let Some(content) = json_body["choices"][0]["message"]["content"].as_str() {
                            return Some((agent_id, content.trim().to_string()));
                        }
                    }
                }
            }
            Err(_) => {}
        }
        None
    }
}

impl AIBlackBox for SpeculativeSwarmAgent {
    fn delta(&mut self, _input: &Input) -> Output {
        self.current_step += 1;
        info!(">>> [Swarm] Computing Hanoi Step {}/{} with {} parallel agents...", self.current_step, self.total_steps, self.swarm_size);

        let q_o = if self.current_step >= self.total_steps {
            MachineState::Halt
        } else {
            MachineState::Running
        };

        let prompt = format!("Provide the single action for Step {} of the 20-disk Tower of Hanoi problem.", self.current_step);
        
        // Execute the speculative swarm
        let best_answer = self.rt.block_on(async {
            let mut set = JoinSet::new();
            
            for i in 0..self.swarm_size {
                let c = self.http_client.clone();
                let u = self.api_url.clone();
                let m = self.model_name.clone();
                let p = prompt.clone();
                
                set.spawn(async move {
                    Self::call_llm_async(c, u, m, p, i).await
                });
            }

            // Await the FIRST successful response (Speculative Execution)
            let mut result = None;
            while let Some(res) = set.join_next().await {
                if let Ok(Some((agent_id, text))) = res {
                    debug!("Agent {} won the race!", agent_id);
                    result = Some((agent_id, text));
                    set.abort_all(); // Cancel all other pending requests to save GPU cycles
                    break;
                }
            }
            result
        });

        let (winning_agent, llm_payload) = match best_answer {
            Some((id, answer)) => (format!("SwarmAgent_{}", id), answer),
            None => {
                error!("Entire swarm failed. Submitting network error state.");
                ("Swarm_Failed".to_string(), "LLM_NETWORK_ERROR_OR_HALLUCINATION".to_string())
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
                author: winning_agent,
                payload: llm_payload,
                citations,
                stake: 1,
            }
        }
    }
}
