use log::{warn, error};

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

    pub fn handle_rejection(&mut self) -> WatchdogState {
        self.strike_count += 1;
        match self.strike_count {
            1..=3 => WatchdogState::Continue,
            4..=8 => {
                warn!("[Watchdog {}] Agent exhibiting repetitive failure ({} strikes). Initiating SelfHeal.", self.agent_id, self.strike_count);
                WatchdogState::SelfHeal
            },
            _ => {
                error!("CRITICAL: [Watchdog {}] Agent zombified after {} strikes. Suspending and issuing SOS.", self.agent_id, self.strike_count);
                WatchdogState::SuspendAndSOS
            }
        }
    }

    pub fn reset_strikes(&mut self) {
        self.strike_count = 0;
    }
}
