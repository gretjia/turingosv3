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
    pub clients: Vec<Arc<ResilientLLMClient>>,
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
            clients: vec![Arc::new(ResilientLLMClient::new(api_url, model_name, timeout_secs))],
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

    /// Multi-model constructor: heterogeneous agent pool
    pub fn new_multi(clients: Vec<Arc<ResilientLLMClient>>, target_steps: u64, swarm_size: usize, sentinel: WalSentinel, recovered_files: Vec<File>, initial_problem_statement: String) -> Self {
        let mut queued_outputs = Vec::new();
        let mut max_step = 0;
        for f in recovered_files.into_iter().rev() {
            if let Some(step_str) = f.id.strip_prefix("step_") {
                if let Some(step_num) = step_str.split('_').next() {
                    if let Ok(num) = step_num.parse::<u64>() {
                        if num > max_step { max_step = num; }
                    }
                }
            }
            queued_outputs.push(Output {
                q_o: MachineState::Running,
                a_o: Action { file_id: f.id, author: f.author, payload: f.payload, citations: f.citations, stake: f.stake }
            });
        }
        SpeculativeSwarmAgent {
            clients,
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
    progress: f32,
) -> Option<(usize, String)> {
    // Millisecond jitter — true concurrent launch, no artificial stagger
    use rand::Rng;
    let jitter_ms = rand::thread_rng().gen_range(0..300);
    sleep(Duration::from_millis(jitter_ms)).await;

    let mut supervisor = crate::harness::AgentSupervisor::new(i, total_agents);
    let mut current_prompt = prompt;

    loop {
        let temp = supervisor.apply_cognitive_divergence(progress);
        let result = client.resilient_generate(&current_prompt, i, temp).await;
        
        let harness_err = match result {
            Ok(raw_text) => {
                let mut full_state = String::new();
                if let Some(pure_state) = distill_pure_state(&raw_text) {
                    full_state.push_str(&pure_state);
                    
                    // Recover Tool call if it exists since distill_pure_state strips everything else
                    if let Some(tool_start) = raw_text.rfind("[Tool: Wallet") {
                        if let Some(tool_end) = raw_text[tool_start..].find(']') {
                            full_state.push_str(" ");
                            full_state.push_str(&raw_text[tool_start..=tool_start+tool_end]);
                        }
                    }
                    return Some((i, full_state));
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
        
        // Depth-weighted frontier selection with thermodynamic annealing
        // Frontier = nodes with no children (leaf nodes of the DAG)
        let all_nodes: Vec<&File> = input.s_i.visible_tape.files.values()
            .filter(|f| !f.payload.contains("failed") && f.stake > 0)
            .collect();

        let frontier_nodes: Vec<&File> = all_nodes.iter()
            .filter(|f| {
                !input.s_i.visible_tape.reverse_citations.get(&f.id)
                    .map_or(false, |children| !children.is_empty())
            })
            .copied()
            .collect();

        let nodes_to_select = if frontier_nodes.is_empty() { &all_nodes } else { &frontier_nodes };

        let selected_head = if nodes_to_select.is_empty() {
            None
        } else {
            // Thermodynamic Annealing: Boltzmann router temperature
            let progress = self.current_step as f64 / self.total_steps.max(1) as f64;
            let temperature = 2.0 - 1.7 * progress; // 2.0 → 0.3
            let depth_alpha = 0.1; // Depth preference strength

            // Compute DAG depth for each node (pure topological property)
            let node_depths: Vec<usize> = nodes_to_select.iter().map(|n| {
                let mut depth = 0;
                let mut current = &n.id;
                while let Some(file) = input.s_i.visible_tape.files.get(current) {
                    if file.citations.is_empty() { break; }
                    depth += 1;
                    current = &file.citations[0];
                }
                depth
            }).collect();

            // Score = intrinsic_reward × (1 + α × depth)
            let scores: Vec<f64> = nodes_to_select.iter().zip(node_depths.iter())
                .map(|(n, &d)| (n.intrinsic_reward + 0.01) * (1.0 + depth_alpha * d as f64))
                .collect();

            let max_score = scores.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

            let weights: Vec<f64> = scores.iter()
                .map(|&s| ((s - max_score) / temperature).exp())
                .collect();

            let weight_sum: f64 = weights.iter().sum();

            use rand::distributions::{WeightedIndex, Distribution};
            let mut rng = rand::thread_rng();

            match WeightedIndex::new(&weights) {
                Ok(dist) => {
                    let idx = dist.sample(&mut rng);
                    let node = nodes_to_select[idx];
                    info!(
                        ">>> [ROUTER] Frontier selected Node {} (Reward: {:.2}, Depth: {}, Prob: {:.2}%, Frontier size: {})",
                        node.id, node.intrinsic_reward, node_depths[idx], (weights[idx] / weight_sum) * 100.0, nodes_to_select.len()
                    );
                    Some(node)
                }
                Err(_) => {
                    nodes_to_select.iter().max_by(|a, b| a.intrinsic_reward.partial_cmp(&b.intrinsic_reward).unwrap_or(std::cmp::Ordering::Equal)).copied()
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

        // Try Mac path first, fallback to Linux path
        let economic_operative = std::fs::read_to_string("/Users/zephryj/projects/turingosv3/skills/economic_operative.md")
            .or_else(|_| std::fs::read_to_string("/home/zephryj/projects/turingosv3/skills/economic_operative.md"))
            .unwrap_or_default();
        
        let mut tombstones_str = String::new();
        if !parent_id.is_empty() {
            if let Some(graves) = input.s_i.tombstones.get(&parent_id) {
                tombstones_str = graves.clone();
            }
        } else {
            if let Some(graves) = input.s_i.tombstones.get("root") {
                tombstones_str = graves.clone();
            }
        }

        // Build Frontier Market Ticker — prices ARE information (Hayek 1945)
        let frontier_market = {
            let reverse_citations = &input.s_i.visible_tape.reverse_citations;
            let mut frontier_with_depth: Vec<(&File, usize)> = input.s_i.visible_tape.files.values()
                .filter(|f| !f.payload.contains("failed") && f.stake > 0)
                .filter(|f| reverse_citations.get(&f.id).map_or(true, |c| c.is_empty()))
                .map(|f| {
                    let mut depth = 0;
                    let mut cur = &f.id;
                    while let Some(file) = input.s_i.visible_tape.files.get(cur) {
                        if file.citations.is_empty() { break; }
                        depth += 1;
                        cur = &file.citations[0];
                    }
                    (f, depth)
                })
                .collect();
            frontier_with_depth.sort_by(|a, b| b.1.cmp(&a.1).then(b.0.intrinsic_reward.partial_cmp(&a.0.intrinsic_reward).unwrap_or(std::cmp::Ordering::Equal)));

            let mut ticker = String::from("\n=== FRONTIER MARKET (Top investment opportunities) ===\n");
            if frontier_with_depth.is_empty() {
                ticker.push_str("Market is empty. Be the first miner to IPO!\n");
            } else {
                for (i, (node, depth)) in frontier_with_depth.iter().take(5).enumerate() {
                    ticker.push_str(&format!("Rank {}: [Node: {}] | Reward: {:.2} | Proof Depth: {}\n", i + 1, node.id, node.intrinsic_reward, depth));
                }
            }
            ticker.push_str("=== To invest in a node: [State: INVEST] [Tool: Wallet | Action: Stake | Node: <ID> | Amount: <FLOAT>] ===\n");
            ticker
        };

        let answers = self.rt.block_on(async {
            let mut set = JoinSet::new();

            // Phase 1: Spawn — one batch of agents, skip bankrupt ones
            let mut spawned = 0;
            for new_id in 0..100 {
                if spawned >= self.swarm_size { break; }

                let agent_name = format!("Agent_{}", new_id);
                let balance = input.s_i.agent_balances.get(&agent_name).copied().unwrap_or(0.0);

                if balance < 1.0 {
                    log::warn!(">>> [LIQUIDATION] Agent {} is bankrupt (balance: {:.2}). Stripped of execution rights.", agent_name, balance);
                    continue;
                }

                let p = format!(
                    "Current Lean 4 Proof State:\n{}\n\n{}\n{}\n{}\n{}\n[YOUR WALLET BALANCE: {:.2} TuringCoins]\n\nYou have TWO choices each step:\n\nOPTION A (Mine): Produce a Lean 4 tactic block and stake on your own work. You may write MULTIPLE tactic lines (separated by \\n) as a single submission.\n[Tactic: your lean 4 tactic] [Tool: Wallet | Action: Stake | Node: self | Amount: <FLOAT>]\nFor multi-line: [Tactic: have h := some_lemma 1\\n  simp at h\\n  exact h] [Tool: Wallet | Action: Stake | Node: self | Amount: <FLOAT>]\n\nOPTION B (Invest): Study the FRONTIER MARKET above and invest in a promising node.\n[State: INVEST] [Tool: Wallet | Action: Stake | Node: <node_id> | Amount: <FLOAT>]\n\nYou are FREE to choose either path based on your judgment.\n\nUSER SPACE THERMODYNAMIC SANDBOX:\nYou may use <think>...</think> tags to reason freely.\n\nWARNING: If your account balance reaches 0, you DIE. Stake wisely — survival is the first priority.",
                    last_state,
                    economic_operative,
                    frontier_market,
                    input.s_i.market_ticker,
                    tombstones_str,
                    balance
                );

                // Heterogeneous model routing: round-robin across client pool
                let c = self.clients[new_id % self.clients.len()].clone();
                log::info!(">>> [DISPATCH] Agent {} → {} (line {})", new_id, c.model_name(), new_id % self.clients.len());
                let total_agents = self.swarm_size;
                let agent_progress = self.current_step as f32 / self.total_steps.max(1) as f32;
                set.spawn(async move {
                    run_agent(new_id, total_agents, c, p, agent_progress).await
                });
                spawned += 1;
            }

            if set.is_empty() {
                log::error!(">>> [MACROECONOMICS] Liquidity Crisis! All agents bankrupt. Market Collapsed.");
                return vec![];
            }

            // Phase 2: Drain — collect ALL surviving branches (true multiverse)
            let mut results: Vec<(usize, String)> = Vec::new();

            while let Some(join_result) = set.join_next().await {
                match join_result {
                    Ok(Some((agent_id, pure_state))) => {
                        // Extract the tactic string and preserve the Tool call
                        let mut tactic = pure_state.clone();
                        let mut tool_call = String::new();

                        if let Some(tool_idx) = tactic.find("[Tool: Wallet") {
                            tool_call = tactic[tool_idx..].to_string();
                            tactic = tactic[..tool_idx].trim().to_string();
                        }

                        let tactic_trimmed = tactic.trim();
                        if tactic_trimmed.starts_with("[Tactic:") && tactic_trimmed.ends_with("]") {
                            tactic = tactic_trimmed[8..tactic_trimmed.len()-1].trim().to_string();
                            // Support multi-line tactics: LLM uses literal \n to separate lines
                            tactic = tactic.replace("\\n", "\n");
                        }

                        let tactic_payload = if tool_call.is_empty() {
                            tactic
                        } else {
                            format!("{} {}", tactic, tool_call)
                        };

                        let new_payload = format!("{}\n  {}", last_state, tactic_payload);
                        log::info!(">>> [MULTIVERSE] Agent {} generated a valid universe branch.", agent_id);
                        results.push((agent_id, new_payload));
                    }
                    Ok(None) => {
                        log::warn!("Agent naturally exited (Watchdog).");
                    }
                    Err(join_err) => {
                        if join_err.is_panic() {
                            log::error!("CRITICAL: An Agent Tokio Thread SILENTLY PANICKED!");
                        } else if join_err.is_cancelled() {
                            log::error!("Agent task cancelled unexpectedly.");
                        } else {
                            log::error!("Agent task failed: {}", join_err);
                        }
                    }
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
                    payload: "[System: swarm round yielded no valid output]".to_string(),
                    citations: vec![],
                    stake: 0,
                }
            }
        }
    }
}
