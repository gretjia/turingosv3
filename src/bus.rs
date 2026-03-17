use crate::kernel::{Kernel, File};
use crate::sdk::tool::{TuringTool, ToolSignal};

pub struct TuringBus {
    pub kernel: Kernel,
    pub tools: Vec<Box<dyn TuringTool>>,
    pub clock: usize,
}

impl TuringBus {
    pub fn new(kernel: Kernel) -> Self {
        Self {
            kernel,
            tools: Vec::new(),
            clock: 0,
        }
    }

    pub fn mount_tool(&mut self, mut tool: Box<dyn TuringTool>) {
        tool.on_boot();
        self.tools.push(tool);
    }

    pub fn init_problem(&mut self, agents: &[String]) {
        for tool in &mut self.tools { 
            tool.on_init(agents); 
        }
    }

    pub fn get_agent_balance(&self, agent_id: &str) -> f64 {
        for tool in &self.tools {
            if let Some(bal) = tool.query_state(&format!("balance_{}", agent_id)) {
                return bal.parse().unwrap_or(0.0);
            }
        }
        0.0
    }

    pub fn halt_and_settle(&mut self, omega_id: &str) {
        let golden_path = self.kernel.trace_golden_path(omega_id);
        for tool in &mut self.tools {
            tool.on_halt(&golden_path, &mut self.kernel.tape);
        }
    }

    pub fn append(&mut self, mut file: File) -> Result<(), String> {
        let mut final_reward = 0.0;
        
        // 1. Pre-append hooks
        for tool in &mut self.tools {
            match tool.on_pre_append(&file.author, &file.payload) {
                ToolSignal::Pass => {}
                ToolSignal::Modify(new_payload) => {
                    file.payload = new_payload;
                }
                ToolSignal::Veto(reason) => {
                    return Err(reason);
                }
                ToolSignal::YieldReward { payload, reward } => {
                    file.payload = payload;
                    final_reward += reward;
                }
                ToolSignal::InvestOnly => {
                    // This node is not appending mathematical truth, it's just a financial transaction.
                    // We allow it to be appended to the tape to record the stake, but it doesn't change state.
                    // The Wallet Tool has already recorded the transaction in its internal ledger.
                }
            }
        }

        // 2. Kernel append
        let node = self.kernel.append_tape(file.clone(), final_reward);

        // 3. Post-append hooks
        for tool in &mut self.tools {
            tool.on_post_append(&file.author, node);
        }

        self.clock += 1;
        Ok(())
    }

    pub fn tick_map_reduce(&mut self) {
        let current_volume = self.kernel.tape.files.len();
        
        // Find current max price in the market
        let current_max_price = self.kernel.tape.files.values()
            .map(|f| f.price)
            .fold(0.0, f64::max);

        let mut skip = false;
        for tool in &mut self.tools {
            if tool.should_skip_reduce(current_volume) {
                skip = true;
            }
            if tool.should_skip_reduce_by_price(current_max_price) {
                skip = true;
            }
        }

        if !skip {
            println!(">>> [Event Bus] Triggering REDUCE (Volume: {}, MaxPrice: {:.2}) <<<", current_volume, current_max_price);
            self.kernel.hayekian_map_reduce();
        }
    }
}

pub struct ThermodynamicHeartbeatTool {
    pub threshold: usize,
    pub last_mr_volume: usize,
}

impl ThermodynamicHeartbeatTool {
    pub fn new(threshold: usize) -> Self {
        Self {
            threshold,
            last_mr_volume: 0,
        }
    }
}

impl TuringTool for ThermodynamicHeartbeatTool {
    fn manifest(&self) -> &'static str {
        "Thermodynamic Heartbeat Skill"
    }

    fn should_skip_reduce(&mut self, current_volume: usize) -> bool {
        if current_volume - self.last_mr_volume >= self.threshold {
            self.last_mr_volume = current_volume;
            false // Do not skip
        } else {
            true // Skip
        }
    }
}

pub struct MembraneGuardTool;

impl TuringTool for MembraneGuardTool {
    fn manifest(&self) -> &'static str {
        "Membrane Guard Skill"
    }
    
    fn on_pre_append(&mut self, _author: &str, payload: &str) -> ToolSignal {
        if payload.contains("paradox") {
            ToolSignal::Veto("Membrane rejected payload".into())
        } else {
            ToolSignal::Pass
        }
    }
}

pub struct WalSnapshotTool;

impl TuringTool for WalSnapshotTool {
    fn manifest(&self) -> &'static str {
        "WAL Snapshot Skill"
    }
}
