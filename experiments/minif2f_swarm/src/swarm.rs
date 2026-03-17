use turingosv3::kernel::{AIBlackBox, Input, Output, Action, MachineState, File};
use turingosv3::sdk::membrane::distill_pure_state;
use turingosv3::drivers::llm_http::ResilientLLMClient;
use log::{info, error};
use std::sync::Arc;
use tokio::runtime::Runtime;
use tokio::task::JoinSet;
use std::time::Duration;
use tokio::time::sleep;
use std::collections::HashSet;
use crate::wal::{WalSentinel, TapeDelta};

pub struct SpeculativeSwarmAgent {
    pub client: Arc<ResilientLLMClient>,
    pub current_step: u64,
    pub total_steps: u64,
    pub swarm_size: usize,
    pub rt: Runtime,
    pub queued_outputs: Vec<Output>,
    pub consecutive_failures: usize,
    pub sentinel: WalSentinel,
    pub known_files: HashSet<String>,
    pub initial_problem_statement: String,
}

impl SpeculativeSwarmAgent {
    pub fn new(api_url: &str, model_name: &str, target_steps: u64, swarm_size: usize, timeout_secs: u64, sentinel: WalSentinel, recovered_files: Vec<File>, initial_problem_statement: String) -> Self {
        let mut queued_outputs = Vec::new();
        let mut max_step = 0;
        
        // We push backwards so pop() returns them in chronological order
        for f in recovered_files.into_iter().rev() {
            if let Some(step_str) = f.id.strip_prefix("step_") {
                if let Some(step_num) = step_str.split('_').next() {
                    if let Ok(num) = step_num.parse::<u64>() {
                        if num > max_step {
                            max_step = num;
                        }
                    }
                }
            }
            queued_outputs.push(Output {
                q_o: MachineState::Running,
                a_o: Action {
                    file_id: f.id,
                    author: f.author,
                    payload: f.payload,
                    citations: f.citations,
                    stake: f.stake,
                }
            });
        }

        SpeculativeSwarmAgent {
            client: Arc::new(ResilientLLMClient::new(api_url, model_name, timeout_secs)),
            current_step: max_step,
            total_steps: target_steps,
            swarm_size,
            rt: Runtime::new().unwrap(),
            queued_outputs,
            consecutive_failures: 0,
            sentinel,
            known_files: HashSet::new(),
            initial_problem_statement,
        }
    }
}

async fn run_agent(
    i: usize,
    total_agents: usize,
    client: Arc<ResilientLLMClient>,
    prompt: String,
) -> Option<(usize, String)> {
    // Stagger the branches heavily to prevent DDoSing the llama.cpp server and triggering timeout avalanche
    sleep(Duration::from_secs(i as u64 * 10)).await;
    
    let mut supervisor = crate::harness::AgentSupervisor::new(i, total_agents);
    let mut current_prompt = prompt;
    
    loop {
        let temp = supervisor.apply_cognitive_divergence();
        let result = client.resilient_generate(&current_prompt, i, temp).await;
        
        let harness_err = match result {
            Ok(raw_text) => {
                if let Some(pure_state) = distill_pure_state(&raw_text) {
                    return Some((i, pure_state));
                } else {
                    crate::harness::HarnessError::SemanticCollapse
                }
            }
            Err(e) => match e {
                turingosv3::drivers::llm_http::DriverError::Timeout => crate::harness::HarnessError::SpacetimeTimeout,
                turingosv3::drivers::llm_http::DriverError::NetworkFracture(msg) => crate::harness::HarnessError::NetworkFracture(msg),
                turingosv3::drivers::llm_http::DriverError::JsonParseError => crate::harness::HarnessError::HardwareTruncation,
                turingosv3::drivers::llm_http::DriverError::BackendError(_) => crate::harness::HarnessError::HardwareTruncation,
            }
        };
        
        match supervisor.handle_rejection(&harness_err) {
            crate::harness::WatchdogState::Continue => {
                sleep(Duration::from_secs(5)).await;
            },
            crate::harness::WatchdogState::SelfHeal => {
                current_prompt.push_str("\n\n[SYSTEM SOS]: Your previous response was truncated by physical limits. You MUST summarize your <think> process under 500 words and output [State: ...] immediately!");
                sleep(Duration::from_secs(5)).await;
            },
            crate::harness::WatchdogState::SuspendAndSOS => {
                error!("Agent {} suspended indefinitely waiting for human intervention.", i);
                return None;
            }
        }
    }
}

impl AIBlackBox for SpeculativeSwarmAgent {
    fn delta(&mut self, input: &Input) -> Output {
        // WAL check: Identify new files in the visible tape
        let mut new_files = Vec::new();
        for (id, file) in &input.s_i.visible_tape.files {
            if !self.known_files.contains(id) {
                new_files.push(file.clone());
                self.known_files.insert(id.clone());
            }
        }
        if !new_files.is_empty() {
            self.sentinel.record_delta(TapeDelta { files: new_files });
        }

        if let Some(output) = self.queued_outputs.pop() {
            return output;
        }

        self.current_step += 1;
        info!(">>> [Swarm] Computing Step {}/{} with {} parallel branches...", self.current_step, self.total_steps, self.swarm_size);

        let q_o = if self.current_step >= self.total_steps { MachineState::Halt } else { MachineState::Running };

        let last_state;
        let mut parent_id = "".to_string();
        
        // 🌟 Boltzmann Softmax Selection (The Backtrack Engine)
        // We look at ALL visible files on the tape, not just the current head.
        // This allows the system to "jump back" to a historically better (purer) node
        // if the current frontier is polluted with zombie tactics.
        let all_nodes: Vec<&File> = input.s_i.visible_tape.files.values()
            .filter(|f| !f.payload.contains("failed") && f.stake > 0) // Only healthy nodes
            .collect();

        let selected_head = if all_nodes.is_empty() {
            None
        } else {
            // Temperature T: larger T = more exploration/backtracking, smaller T = more greedy
            let temperature = 0.5; 
            
            let prices: Vec<f64> = all_nodes.iter().map(|n| n.price).collect();
            let max_price = prices.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
            
            let weights: Vec<f64> = prices.iter()
                .map(|&p| ((p - max_price) / temperature).exp())
                .collect();
            
            let weight_sum: f64 = weights.iter().sum();
            
            use rand::distributions::{WeightedIndex, Distribution};
            let mut rng = rand::thread_rng();
            
            match WeightedIndex::new(&weights) {
                Ok(dist) => {
                    let idx = dist.sample(&mut rng);
                    let node = all_nodes[idx];
                    info!(
                        ">>> [ROUTER] Softmax selected Node {} (Price: {:.2}, Prob: {:.2}%)", 
                        node.id, node.price, (weights[idx] / weight_sum) * 100.0
                    );
                    Some(node)
                }
                Err(_) => {
                    // Fallback to greedy if weights collapse
                    all_nodes.iter().max_by(|a, b| a.price.partial_cmp(&b.price).unwrap_or(std::cmp::Ordering::Equal)).copied()
                }
            }
        };

        if let Some(file) = selected_head {
            last_state = file.payload.clone();
            parent_id = file.id.clone();
        } else {
            // Very first step! Seed it with the actual Lean 4 theorem.
            last_state = self.initial_problem_statement.clone();
        }

        let economic_operative = std::fs::read_to_string("/home/zephryj/projects/turingosv3/skills/economic_operative.md").unwrap_or_default();
        
        let answers = self.rt.block_on(async {
            let mut set = JoinSet::new();
            let mut next_agent_id = 0;
            loop {
                while set.len() < self.swarm_size {
                    let new_id = next_agent_id;
                    next_agent_id += 1;
                    
                    let agent_name = format!("Agent_{}", new_id);
                    let balance = input.s_i.agent_balances.get(&agent_name).copied().unwrap_or(0.0);
                    
                    let p = format!(
                        "Current Lean 4 Proof State:\n{}\n\n{}\n\n[YOUR WALLET BALANCE: {:.2} TuringCoins]\n\nProvide the next single logical Lean 4 Tactic to advance this proof.\n\nUSER SPACE THERMODYNAMIC SANDBOX:\nYou are permitted to go mad and deduce freely! You may use <think>...</think> tags. You may write 10,000 words to deduce, hypothesize, and self-correct. The OS will not interfere with your intelligence divergence. Release all your computing power to solve this problem!\n\nKERNEL SPACE PHASE-TRANSITION (CRITICAL):\nHowever, at the very end of your thought process, you MUST output your final decision in the exact format: [Tactic: your single lean 4 tactic here] and follow the Wallet Tool invocation rules.", 
                        last_state,
                        economic_operative,
                        balance
                    );
                    
                    let c = self.client.clone();
                    let total_agents = self.swarm_size;
                    set.spawn(async move { 
                        run_agent(new_id, total_agents, c, p).await
                    });
                }

                // 2. Wait for the next task event (completion, panic, success)
                match set.join_next().await {
                    Some(Ok(Some((agent_id, pure_state)))) => {
                        // Extract the tactic string from [Tactic: rfl]
                        let tactic = if pure_state.starts_with("[Tactic:") && pure_state.ends_with("]") {
                            pure_state[8..pure_state.len()-1].trim().to_string()
                        } else {
                            pure_state
                        };
                        
                        // Append to previous tactics/state
                        let new_payload = format!("{}\n  {}", last_state, tactic);
                        
                        return vec![(agent_id, new_payload)];
                    }
                    Some(Ok(None)) => {
                         // An agent hit Watchdog SuspendAndSOS and gracefully exited.
                         log::warn!("Agent naturally exited (likely executed by Watchdog). Waiting for next resurrection.");
                    }
                    Some(Err(join_err)) => {
                         // An agent task SILENTLY PANICKED or was cancelled!
                         if join_err.is_panic() {
                             log::error!("CRITICAL: An Agent Tokio Thread SILENTLY PANICKED!");
                         } else if join_err.is_cancelled() {
                             log::error!("Agent task cancelled unexpectedly.");
                         } else {
                             log::error!("Agent task failed: {}", join_err);
                         }
                    }
                    None => {
                         // This is the true void. We should never really hit this if we respawn at the top,
                         // but if we do, the `loop` will just jump back to the top and `while set.len() < size` will respawn everything.
                         log::error!("JoinSet empty. The void was reached. Respawning.");
                    }
                }
            }
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
