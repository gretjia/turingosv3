pub enum SkillSignal {
    Pass,
    Modify(String),
    Veto(String),
    YieldReward { payload: String, reward: f64 },
}

pub trait TuringSkill: Send + Sync {
    fn manifest(&self) -> &'static str;
    fn on_boot(&mut self) {}
    fn on_pre_append(&mut self, _payload: &str) -> SkillSignal { SkillSignal::Pass }
    fn on_post_append(&mut self, _node: &crate::kernel::File) {}
    
    /// Decision to skip Reduce based on volume (Legacy)
    fn should_skip_reduce(&mut self, _current_volume: usize) -> bool { false }

    /// Decision to skip Reduce based on price gap (Advanced)
    fn should_skip_reduce_by_price(&mut self, _current_max_price: f64) -> bool { false }
}

pub struct AntiZombiePruningSkill {
    max_consecutive_tactics: usize,
}

impl AntiZombiePruningSkill {
    pub fn new(max_consecutive_tactics: usize) -> Self {
        Self { max_consecutive_tactics }
    }
}

impl TuringSkill for AntiZombiePruningSkill {
    fn manifest(&self) -> &'static str { "core.skill.anti_zombie_pruning_shield" }

    fn on_pre_append(&mut self, payload: &str) -> SkillSignal {
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
                    return SkillSignal::Veto(format!("Zombie Behavior: LLM is trapped in a useless tactic loop '{}'.", tactics[i]));
                }
            } else {
                consecutive_count = 1;
            }
        }
        SkillSignal::Pass
    }
}

pub struct OverwhelmingGapArbitrator {
    last_max_price: f64,
    threshold_ratio: f64,
}

impl OverwhelmingGapArbitrator {
    pub fn new(threshold_ratio: f64) -> Self {
        Self { last_max_price: 1.0, threshold_ratio }
    }
}

impl TuringSkill for OverwhelmingGapArbitrator {
    fn manifest(&self) -> &'static str { "core.skill.overwhelming_gap_arbitrator" }

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
