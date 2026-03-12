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
                        return Some((agent_id, content.trim().to_string()));
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
        
        if let Some(head_id) = input.s_i.current_head.paths.iter().next() {
            if let Some(file) = input.s_i.visible_tape.files.get(head_id) {
                last_state = file.payload.clone();
                parent_id = head_id.clone();
            }
        }

        let prompt = format!(
            "Current State:\n{}\n\nProvide the logical NEXT STATE for the 20-disk Tower of Hanoi.", 
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
