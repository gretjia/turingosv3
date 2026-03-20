use log::{warn, error};

#[derive(Debug)]
pub enum HarnessError {
    NetworkFracture(String),
    SpacetimeTimeout,
    HardwareTruncation,
    SemanticCollapse,
}

pub enum WatchdogState {
    Continue,
    SelfHeal,
    SuspendAndSOS,
}

pub struct AgentSupervisor {
    pub agent_id: usize,
    pub total_agents: usize,
    pub strike_count: usize,
}

impl AgentSupervisor {
    pub fn new(agent_id: usize, total_agents: usize) -> Self {
        Self {
            agent_id,
            total_agents,
            strike_count: 0,
        }
    }

    /// Thermodynamic Annealing: LLM generation temperature as a function of
    /// agent identity AND exploration progress.
    /// - Early (progress≈0): wide spread [0.1, 1.5] → maximum entropy, diverse exploration
    /// - Late  (progress≈1): narrow spread [0.3, 0.6] → exploitation around known paths
    pub fn apply_cognitive_divergence(&self, progress: f32) -> f32 {
        let progress = progress.clamp(0.0, 1.0);
        let t_min = 0.1 + 0.2 * progress;   // 0.1 → 0.3
        let t_max = 1.5 - 0.9 * progress;   // 1.5 → 0.6
        let agent_fraction = self.agent_id as f32 / self.total_agents.max(1) as f32;
        t_min + (t_max - t_min) * agent_fraction
    }

    pub fn handle_rejection(&mut self, err: &HarnessError) -> WatchdogState {
        self.strike_count += 1;
        match self.strike_count {
            1..=3 => WatchdogState::Continue,
            4..=8 => {
                warn!("[Watchdog {}] Agent exhibiting repetitive failure ({:?}, {} strikes). Initiating SelfHeal.", self.agent_id, err, self.strike_count);
                WatchdogState::SelfHeal
            },
            _ => {
                error!("CRITICAL: [Watchdog {}] Agent zombified after {} strikes ({:?}). Suspending and issuing SOS.", self.agent_id, self.strike_count, err);
                WatchdogState::SuspendAndSOS
            }
        }
    }

    pub fn reset_strikes(&mut self) {
        self.strike_count = 0;
    }
}
