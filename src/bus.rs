use crate::kernel::{Kernel, File};
use crate::sdk::tool::{TuringTool, ToolSignal};
use std::collections::{HashMap, VecDeque};

#[derive(Debug, Clone, Default)]
pub struct Graveyard {
    pub tombstones: HashMap<String, VecDeque<String>>,
}

impl Graveyard {
    pub fn new() -> Self {
        Self { tombstones: HashMap::new() }
    }
    
    pub fn record_death(&mut self, node_id: &str, reason: &str) {
        let entry = self.tombstones.entry(node_id.to_string()).or_insert_with(VecDeque::new);
        // Dedup: don't store identical errors, preserve unique failure diversity
        if !entry.iter().any(|existing| existing == reason) {
            entry.push_back(reason.to_string());
        }
        // Keep up to 10 unique errors (was 3, too few for N=15 swarm)
        if entry.len() > 10 {
            entry.pop_front();
        }
    }
    
    pub fn get_tombstones(&self, node_id: &str) -> String {
        if let Some(graves) = self.tombstones.get(node_id) {
            if graves.is_empty() { return String::new(); }
            let mut s = String::from("\n=== 🪦 GRAVEYARD: RECENT BANKRUPTCIES ON THIS NODE ===\n");
            for (i, reason) in graves.iter().enumerate() {
                s.push_str(&format!("Failure {}: {}\n", i + 1, reason));
            }
            s.push_str("=====================================================\n");
            s
        } else {
            String::new()
        }
    }
}

pub struct TuringBus {
    pub kernel: Kernel,
    pub tools: Vec<Box<dyn TuringTool>>,
    pub clock: usize,
    pub graveyard: Graveyard,
}

impl TuringBus {
    pub fn new(kernel: Kernel) -> Self {
        Self {
            kernel,
            tools: Vec::new(),
            clock: 0,
            graveyard: Graveyard::new(),
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

    pub fn get_tombstones(&self, node_id: &str) -> String {
        self.graveyard.get_tombstones(node_id)
    }

    /// Inject capital into a specific agent (generation rebirth / Chapter 11 reorganization).
    /// Delegates to WalletTool — bus is pure router.
    pub fn fund_agent(&mut self, agent_id: &str, amount: f64) {
        use crate::sdk::tools::wallet::WalletTool;
        for tool in &mut self.tools {
            if tool.manifest() == "core.tool.crypto_wallet" {
                if let Some(wallet) = tool.as_any_mut().downcast_mut::<WalletTool>() {
                    wallet.fund_agent(agent_id, amount);
                }
                break;
            }
        }
    }

    /// Redistribute global_pool among surviving agents between theorems
    pub fn redistribute_pool(&mut self) {
        use crate::sdk::tools::wallet::WalletTool;
        for tool in &mut self.tools {
            if tool.manifest() == "core.tool.crypto_wallet" {
                if let Some(wallet) = tool.as_any_mut().downcast_mut::<WalletTool>() {
                    wallet.redistribute_pool();
                }
                break;
            }
        }
    }

    /// Extract all agent balances for cross-theorem persistence
    pub fn extract_wallet_balances(&self) -> std::collections::HashMap<String, f64> {
        let mut balances = std::collections::HashMap::new();
        for i in 0..100 {
            let agent_id = format!("Agent_{}", i);
            let balance = self.get_agent_balance(&agent_id);
            if balance > 0.0 {
                balances.insert(agent_id, balance);
            }
        }
        balances
    }

    /// Freeze the current universe state into an immutable snapshot.
    /// Agents read this snapshot lock-free — past spacetime is absolute.
    pub fn get_immutable_snapshot(&self) -> crate::sdk::snapshot::UniverseSnapshot {
        let mut balances = std::collections::HashMap::new();
        for i in 0..100 {
            let aid = format!("Agent_{}", i);
            balances.insert(aid.clone(), self.get_agent_balance(&aid));
        }
        let mut tombstones = std::collections::HashMap::new();
        for id in self.kernel.tape.files.keys() {
            let g = self.get_tombstones(id);
            if !g.is_empty() { tombstones.insert(id.clone(), g); }
        }
        let rg = self.get_tombstones("root");
        if !rg.is_empty() { tombstones.insert("root".to_string(), rg); }

        crate::sdk::snapshot::UniverseSnapshot {
            tape: self.kernel.tape.clone(),
            balances,
            market_ticker: self.kernel.get_market_ticker(3),
            tombstones,
        }
    }

    pub fn halt_and_settle(&mut self, omega_id: &str) {
        let golden_path = self.kernel.trace_golden_path(omega_id);
        for tool in &mut self.tools {
            tool.on_halt(&golden_path, &mut self.kernel.tape);
        }
    }

    pub fn append(&mut self, mut file: File) -> Result<(), String> {
        let mut final_reward = 0.0;
        let mut is_invest_only = false;
        let mut invest_target = String::new();
        let mut invest_amount = 0.0;
        
        // 1. Pre-append hooks
        for tool in &mut self.tools {
            match tool.on_pre_append(&file.author, &file.payload) {
                ToolSignal::Pass => {}
                ToolSignal::Modify(new_payload) => {
                    file.payload = new_payload;
                }
                ToolSignal::Veto(reason) => {
                    log::warn!(">>> [TOOL VETO] Author: {}, Reason: {}", file.author, reason);
                    let parent_id = if file.citations.is_empty() {
                        "root".to_string()
                    } else {
                        file.citations[0].clone()
                    };
                    self.graveyard.record_death(&parent_id, &reason);
                    return Err(reason);
                }
                ToolSignal::YieldReward { payload, reward } => {
                    file.payload = payload;
                    final_reward += reward;
                }
                ToolSignal::InvestOnly { target_node, amount } => {
                    is_invest_only = true;
                    invest_target = target_node;
                    invest_amount = amount;
                    break; // Break tool chain, bypass Lean4 membrane
                }
            }
        }

        if is_invest_only {
            // 🌟 Inject capital directly into historical node
            if let Some(node) = self.kernel.tape.files.get_mut(&invest_target) {
                node.intrinsic_reward += invest_amount;
                log::info!(">>> [MARKET PUMP] Node {} received VC funding of {:.2}! Market Cap surging!", invest_target, invest_amount);
                // Trigger global gravity recalculation
                self.kernel.hayekian_map_reduce();
            } else {
                log::warn!(">>> [VC ERROR] Node {} does not exist. Investment burned.", invest_target);
            }
            return Ok(()); // End turn, no new node created
        }

        // 2. Kernel append (with causality enforcement)
        let node = match self.kernel.append_tape(file.clone(), final_reward) {
            Ok(node) => node,
            Err(reason) => {
                log::warn!(">>> [KERNEL REJECT] {}", reason);
                return Err(reason);
            }
        };

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
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }

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
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
    
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
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
}
