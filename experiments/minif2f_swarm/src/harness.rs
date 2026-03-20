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

    pub fn apply_cognitive_divergence(&self) -> f32 {
        0.2 + (0.6 * (self.agent_id as f32 / self.total_agents.max(1) as f32))
    }

    pub fn handle_rejection(&mut self, err: &HarnessError) -> WatchdogState {
        self.strike_count += 1;
        match self.strike_count {
            1..=2 => WatchdogState::Continue,
            3 => {
                warn!("[Watchdog {}] Agent failing ({:?}, {} strikes). SelfHeal attempt.", self.agent_id, err, self.strike_count);
                WatchdogState::SelfHeal
            },
            _ => {
                // Suspend quickly to prevent SelfHeal prompt accumulation → 400 death spiral
                error!("[Watchdog {}] Agent zombified after {} strikes ({:?}). Suspending.", self.agent_id, self.strike_count, err);
                WatchdogState::SuspendAndSOS
            }
        }
    }

    pub fn reset_strikes(&mut self) {
        self.strike_count = 0;
    }
}
