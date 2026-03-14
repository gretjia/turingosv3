use turingosv3::kernel::{AIBlackBox, Input, Output, Action, MachineState};
use turingosv3::sdk::membrane::distill_pure_state;
use turingosv3::drivers::llm_http::ResilientLLMClient;
use log::{info, error};
use std::sync::Arc;
use tokio::runtime::Runtime;
use tokio::task::JoinSet;
use std::time::Duration;
use tokio::time::sleep;

pub struct SpeculativeSwarmAgent {
    pub client: Arc<ResilientLLMClient>,
    pub current_step: u64,
    pub total_steps: u64,
    pub swarm_size: usize,
    pub rt: Runtime,
    pub queued_outputs: Vec<Output>,
    pub consecutive_failures: usize,
}

impl SpeculativeSwarmAgent {
    pub fn new(api_url: &str, model_name: &str, target_steps: u64, swarm_size: usize, timeout_secs: u64) -> Self {
        SpeculativeSwarmAgent {
            client: Arc::new(ResilientLLMClient::new(api_url, model_name, timeout_secs)),
            current_step: 0,
            total_steps: target_steps,
            swarm_size,
            rt: Runtime::new().unwrap(),
            queued_outputs: Vec::new(),
            consecutive_failures: 0,
        }
    }
}

impl AIBlackBox for SpeculativeSwarmAgent {
    fn delta(&mut self, input: &Input) -> Output {
        if let Some(output) = self.queued_outputs.pop() {
            return output;
        }

        self.current_step += 1;
        info!(">>> [Swarm] Computing Step {}/{} with {} parallel branches...", self.current_step, self.total_steps, self.swarm_size);

        let q_o = if self.current_step >= self.total_steps { MachineState::Halt } else { MachineState::Running };

        let mut last_state = "Initial State: Peg 1: [1..20], Peg 2: [], Peg 3: []".to_string();
        let mut parent_id = "".to_string();
        
        let best_head = input.s_i.current_head.paths.iter()
            .filter_map(|id| input.s_i.visible_tape.files.get(id))
            .max_by(|a, b| a.price.partial_cmp(&b.price).unwrap_or(std::cmp::Ordering::Equal));

        if let Some(file) = best_head {
            let full_payload = file.payload.clone();
            if let Some(pure_state) = distill_pure_state(&full_payload) {
                last_state = pure_state;
            } else {
                last_state = full_payload;
            }
            parent_id = file.id.clone();
        }

        let prompt = format!(
            "Current State:\n{}\n\nProvide the logical NEXT STATE for the 20-disk Tower of Hanoi.\n\nUSER SPACE THERMODYNAMIC SANDBOX:\nYou are permitted to go mad and deduce freely! You may use <think>...</think> tags. You may write 10,000 words to deduce, hypothesize, and self-correct. The OS will not interfere with your intelligence divergence. Release all your computing power to solve this problem!\n\nKERNEL SPACE PHASE-TRANSITION (CRITICAL):\nHowever, at the very end of your thought process, you MUST output your final decision in the exact format: [State: describe the exact new state of all pegs here]. Everything before this final tag is considered your draft and will be safely ignored.", 
            last_state
        );
        
        let answers = self.rt.block_on(async {
            let mut set = JoinSet::new();
            for i in 0..self.swarm_size {
                let c = self.client.clone();
                let p = prompt.clone();
                set.spawn(async move { 
                    // Stagger the branches heavily to prevent DDoSing the llama.cpp server and triggering timeout avalanche
                    sleep(Duration::from_secs(i as u64 * 10)).await;
                    let result = c.resilient_generate(&p, i, 5).await;
                    if let Some(raw_text) = result {
                        if let Some(pure_state) = distill_pure_state(&raw_text) {
                            return Some((i, pure_state));
                        }
                    }
                    None
                });
            }

            let mut results = Vec::new();
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
                    stake: 1, 
                }
            });
        }

        if let Some(output) = self.queued_outputs.pop() {
            self.consecutive_failures = 0; 
            output
        } else {
            self.current_step -= 1;
            self.consecutive_failures += 1;

            if self.consecutive_failures >= 20 {
                error!("Swarm hit maximum consecutive failures (20). HALTING SYSTEM for debug.");
                return Output {
                    q_o: MachineState::Halt,
                    a_o: Action {
                        file_id: "system_aborted_due_to_failures".to_string(),
                        author: "System".to_string(),
                        payload: "[State: HALTED DUE TO REPEATED API FAILURES]".to_string(),
                        citations: vec![],
                        stake: 1, 
                    }
                };
            }

            Output {
                q_o: MachineState::Running,
                a_o: Action {
                    file_id: format!("step_{}_failed", self.current_step + 1),
                    author: "System".to_string(),
                    payload: "paradox: swarm completely failed".to_string(), 
                    citations: vec![],
                    stake: 0,
                }
            }
        }
    }
}
