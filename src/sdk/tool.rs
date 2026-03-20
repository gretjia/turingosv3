use crate::kernel::{Tape, File as TapeNode};

pub enum ToolSignal {
    Pass,
    Modify(String),
    Veto(String),
    YieldReward { payload: String, reward: f64 },
    InvestOnly { target_node: String, amount: f64 }, 
}

pub trait TuringTool: Send + Sync {
    fn manifest(&self) -> &'static str;
    /// Downcast support for inter-tool communication (e.g., bus → wallet redistribution)
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
    fn on_boot(&mut self) {}
    
    fn on_init(&mut self, _agents: &[String]) {}
    
    fn on_pre_append(&mut self, _author: &str, _payload: &str) -> ToolSignal { ToolSignal::Pass }
    fn on_post_append(&mut self, _author: &str, _node: &TapeNode) {}
    
    fn on_halt(&mut self, _golden_path: &[String], _tape: &mut Tape) {} 
    
    fn query_state(&self, _key: &str) -> Option<String> { None }

    /// Decision to skip Reduce based on volume (Legacy)
    fn should_skip_reduce(&mut self, _current_volume: usize) -> bool { false }

    /// Decision to skip Reduce based on price gap (Advanced)
    fn should_skip_reduce_by_price(&mut self, _current_max_price: f64) -> bool { false }
}

pub struct AntiZombiePruningTool {
    max_consecutive_tactics: usize,
}

impl AntiZombiePruningTool {
    pub fn new(max_consecutive_tactics: usize) -> Self {
        Self { max_consecutive_tactics }
    }
}

impl TuringTool for AntiZombiePruningTool {
    fn manifest(&self) -> &'static str { "core.skill.anti_zombie_pruning_shield" }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }

    fn on_pre_append(&mut self, _author: &str, payload: &str) -> ToolSignal {
        // Extract pure Tactic lines
        let tactics: Vec<&str> = payload.lines()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty() && !l.starts_with("--"))
            .collect();
            
        let mut consecutive_count = 1;
        
        for i in 1..tactics.len() {
            // [Hayekian Circuit Breaker]: Redundant tactics with no state displacement
            if tactics[i] == tactics[i - 1] {
                consecutive_count += 1;
                if consecutive_count >= self.max_consecutive_tactics {
                    log::warn!(">>> [PRUNE] Zombie Tactic Stack Detected: '{}' repeated {} times.", tactics[i], consecutive_count);
                    return ToolSignal::Veto(format!("Zombie Behavior: LLM is trapped in a useless tactic loop '{}'.", tactics[i]));
                }
            } else {
                consecutive_count = 1;
            }
        }
        // Period-2 cycle detection: A → B → A → B
        if tactics.len() >= 4 {
            let n = tactics.len();
            let mut cycle2_count = 0;
            for i in (2..n).rev() {
                if tactics[i] == tactics[i - 2] {
                    cycle2_count += 1;
                } else {
                    break;
                }
            }
            if cycle2_count >= 3 {
                log::warn!(">>> [PRUNE] Period-2 Zombie Cycle Detected: '{}' <-> '{}'", tactics[n-1], tactics[n-2]);
                return ToolSignal::Veto(format!("Zombie Behavior: LLM trapped in period-2 cycle '{}' <-> '{}'", tactics[n-1], tactics[n-2]));
            }
        }

        ToolSignal::Pass
    }
}

pub struct OverwhelmingGapArbitratorTool {
    last_max_price: f64,
    threshold_ratio: f64,
}

impl OverwhelmingGapArbitratorTool {
    pub fn new(threshold_ratio: f64) -> Self {
        Self { last_max_price: 1.0, threshold_ratio }
    }
}

impl TuringTool for OverwhelmingGapArbitratorTool {
    fn manifest(&self) -> &'static str { "core.skill.overwhelming_gap_arbitrator" }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }

    fn should_skip_reduce_by_price(&mut self, current_max_price: f64) -> bool {
        if current_max_price >= self.last_max_price * self.threshold_ratio {
            log::info!(">>> [PHASE TRANSITION] Overwhelming Price Gap ({:.2} >= {:.2})! Unleashing MapReduce.", current_max_price, self.last_max_price * self.threshold_ratio);
            self.last_max_price = current_max_price;
            false 
        } else {
            log::debug!(">>> [ARBITRATOR] Flat market. Skipping Reduce. Prune & Softmax mode active.");
            true 
        }
    }
}
