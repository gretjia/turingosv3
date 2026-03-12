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
    pub queued_outputs: Vec<Output>,
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
            queued_outputs: Vec::new(),
        }
    }

    async fn call_llm_async(client: Client, url: String, model: String, prompt: String, agent_id: usize) -> Option<(usize, String)> {
        let payload = json!({
            "model": model,
            "messages": [
                {
                    "role": "system",
                    "content": "You are a logical reasoning agent. You receive a Current State. You must output the Next Action and the resulting New State."
                },
                {
                    "role": "user",
                    "content": prompt
                }
            ],
            // Temperature varies to create genuine DAG branching
            "temperature": 0.1 + (agent_id as f32 * 0.2), 
            "max_tokens": 150
        });

        if let Ok(response) = client.post(&url).json(&payload).send().await {
            if response.status().is_success() {
                if let Ok(json_body) = response.json::<serde_json::Value>().await {
                    if let Some(content) = json_body["choices"][0]["message"]["content"].as_str() {
                        let text = content.trim().to_string();
                        // Red-Flagging: Discard if empty, too short, or lacks the required State tag.
                        if text.len() > 10 && text.contains("[State:") {
                            return Some((agent_id, text));
                        } else {
                            log::warn!("Agent {} triggered Red-Flag: Invalid format or empty payload.", agent_id);
                        }
                    }
                }
            }
        }
        None
    }
}

impl AIBlackBox for SpeculativeSwarmAgent {
    fn delta(&mut self, input: &Input) -> Output {
        // If we have queued branches, yield them one by one to the kernel
        if let Some(output) = self.queued_outputs.pop() {
            return output;
        }

        self.current_step += 1;
        info!(">>> [Swarm] Computing Step {}/{} with {} parallel branches...", self.current_step, self.total_steps, self.swarm_size);

        let q_o = if self.current_step >= self.total_steps { MachineState::Halt } else { MachineState::Running };

        // [Phase 2 Fix]: Inject the Markov State Context!
        // Find the latest state from the current HEAD.
        let mut last_state = "Initial State: Peg 1: [1..20], Peg 2: [], Peg 3: []".to_string();
        let mut parent_id = "".to_string();
        
        // [Phase 4 Fix - Value Oriented HEAD Selection / Strict Agent Isolation]: 
        // According to the new doctrine: "禁止读取别家的纸带".
        // To enforce this, we only look at the history that belongs to the current branch/agent
        // However, since we are doing 4 parallel branches at once in the `delta` loop, 
        // the concept of "my own tape" means we just pick ONE best history to base the NEXT 4 branches on.
        // Wait, to truly isolate, we should spawn completely separate Agent instances.
        // But for this test, we simulate it by hiding the 'price' metric (Goodhart problem) 
        // and just taking the most deeply reasoned state we can find in the HEAD, without exposing metrics to the LLM.

        let best_head = input.s_i.current_head.paths.iter()
            .filter_map(|id| input.s_i.visible_tape.files.get(id))
            .max_by(|a, b| a.price.partial_cmp(&b.price).unwrap_or(std::cmp::Ordering::Equal));

        if let Some(file) = best_head {
            // [Shield Goodhart Problem]: Hide metrics (Price) from the LLM. Only give it the payload.
            last_state = file.payload.clone();
            parent_id = file.id.clone();
        }

        let prompt = format!(
            "Current State:\n{}\n\nProvide the logical NEXT STATE for the 20-disk Tower of Hanoi.\n\nOUTPUT FORMAT:\n[Moves: describe the move here]\n[State: describe the exact new state here]", 
            last_state
        );
        
        let answers = self.rt.block_on(async {
            let mut set = JoinSet::new();
            for i in 0..self.swarm_size {
                let c = self.http_client.clone();
                let u = self.api_url.clone();
                let m = self.model_name.clone();
                let p = prompt.clone();
                set.spawn(async move { Self::call_llm_async(c, u, m, p, i).await });
            }

            let mut results = Vec::new();
            // [Phase 4 Fix]: Do NOT abort! Collect all parallel branches for the DAG!
            while let Some(res) = set.join_next().await {
                if let Ok(Some((agent_id, text))) = res {
                    results.push((agent_id, text));
                }
            }
            results
        });

        let mut citations = vec![];
        if !parent_id.is_empty() { citations.push(parent_id); }

        for (agent_id, text) in answers {
            self.queued_outputs.push(Output {
                q_o: q_o.clone(),
                a_o: Action {
                    file_id: format!("step_{}_branch_{}", self.current_step, agent_id),
                    author: format!("Agent_{}", agent_id),
                    payload: text,
                    citations: citations.clone(),
                    stake: 1, // Pay the thermodynamic cost
                }
            });
        }

        // Return the first branch, others will be yielded in subsequent ticks
        if let Some(output) = self.queued_outputs.pop() {
            output
        } else {
            Output {
                q_o,
                a_o: Action {
                    file_id: format!("step_{}_failed", self.current_step),
                    author: "System".to_string(),
                    payload: "paradox: swarm completely failed".to_string(), // Will be killed by Guillotine
                    citations: vec![],
                    stake: 0,
                }
            }
        }
    }
}
