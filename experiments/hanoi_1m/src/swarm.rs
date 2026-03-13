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
    pub fn new(api_url: &str, model_name: &str, target_steps: u64, swarm_size: usize, timeout_secs: u64) -> Self {
        SpeculativeSwarmAgent {
            api_url: api_url.to_string(),
            model_name: model_name.to_string(),
            http_client: Client::builder()
                .timeout(Duration::from_secs(timeout_secs))
                .build()
                .unwrap(),
            current_step: 0,
            total_steps: target_steps,
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
            "max_tokens": 4096,
            "stream": false
        });

        // Add 3 retries for transient 500/502 errors when hitting llama.cpp too hard
        for attempt in 1..=3 {
            match client.post(&url).json(&payload).send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        if let Ok(json_body) = response.json::<serde_json::Value>().await {
                            // Support both OpenAI format (llama.cpp) and native Ollama format
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
                            } else if let Some(message) = json_body.get("message") {
                                if let Some(content) = message.get("content").and_then(|v| v.as_str()) {
                                    final_content.push_str(content);
                                }
                            }

                            if !final_content.is_empty() {
                                let text = final_content.trim().to_string();
                                // Red-Flagging: Discard if empty, too short, or lacks the required State tag.
                                if text.len() > 10 && text.contains("[State:") {
                                    return Some((agent_id, text));
                                } else {
                                    log::warn!("Agent {} triggered Red-Flag: Invalid format or empty payload. Payload: {}", agent_id, text);
                                    return None; // Format error, don't retry, just die
                                }
                            } else {
                                log::error!("Agent {} API Response parsed but no content found. Body: {}", agent_id, json_body);
                            }
                        } else {
                            log::error!("Agent {} failed to parse JSON response body.", agent_id);
                        }
                    } else {
                        log::error!("Agent {} HTTP Error: {} on attempt {}", agent_id, response.status(), attempt);
                        // 500/502/503 errors usually mean the server queue is full
                        tokio::time::sleep(Duration::from_secs(5 * attempt)).await;
                        continue;
                    }
                },
                Err(e) => {
                    log::error!("Agent {} HTTP Request Failed: {} on attempt {}", agent_id, e, attempt);
                    tokio::time::sleep(Duration::from_secs(5 * attempt)).await;
                    continue;
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
            // [Shield Goodhart Problem & Detail Encapsulation]: 
            // We must ONLY pass the clean physical state `[State: ...]` to the next step.
            // We MUST strip out the `Thinking Process:` and `[Moves: ...]` to prevent Context Corruption!
            let full_payload = file.payload.clone();
            
            // Extract just the "[State: ...]" block using basic string parsing
            if let Some(start_idx) = full_payload.find("[State:") {
                if let Some(end_idx) = full_payload[start_idx..].find("]") {
                    last_state = full_payload[start_idx..start_idx + end_idx + 1].to_string();
                } else {
                    last_state = full_payload[start_idx..].to_string();
                }
            } else {
                // Fallback just in case, though Red-Flag should prevent this from entering Tape
                last_state = full_payload;
            }
            
            parent_id = file.id.clone();
        }

        let prompt = format!(
            "Current State:\n{}\n\nProvide the logical NEXT STATE for the 20-disk Tower of Hanoi.\n\nCRITICAL INSTRUCTION: You MUST use the exact tags [Moves: ...] and [State: ...]. Do not write any other text.\n\nOUTPUT FORMAT:\n[Moves: describe the single move here]\n[State: describe the exact new state of all pegs here]", 
            last_state
        );
        
        let answers = self.rt.block_on(async {
            let mut set = JoinSet::new();
            for i in 0..self.swarm_size {
                let c = self.http_client.clone();
                let u = self.api_url.clone();
                let m = self.model_name.clone();
                let p = prompt.clone();
                set.spawn(async move { 
                    // Stagger the initial requests to prevent slamming the Llama HTTP server all at once
                    tokio::time::sleep(Duration::from_secs(i as u64 * 2)).await;
                    Self::call_llm_async(c, u, m, p, i).await 
                });
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
            // [Fix]: The entire swarm failed red-flagging.
            // We must NOT advance to the next step, otherwise we leave a gaping hole in the timeline.
            // We retract the step counter so the next tick retries the SAME step.
            self.current_step -= 1;
            Output {
                q_o: MachineState::Running,
                a_o: Action {
                    file_id: format!("step_{}_failed", self.current_step + 1),
                    author: "System".to_string(),
                    payload: "paradox: swarm completely failed".to_string(), // Will be killed by Guillotine
                    citations: vec![],
                    stake: 0,
                }
            }
        }
    }
}
