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
        if !entry.iter().any(|existing| existing == reason) {
            entry.push_back(reason.to_string());
        }
        if entry.len() > 10 {
            entry.pop_front();
        }
    }

    pub fn get_tombstones(&self, node_id: &str) -> String {
        if let Some(graves) = self.tombstones.get(node_id) {
            if graves.is_empty() { return String::new(); }
            let mut s = String::from("\n=== GRAVEYARD: RECENT BANKRUPTCIES ON THIS NODE ===\n");
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

    /// Inject capital into a specific agent (generation rebirth).
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
    pub fn extract_wallet_balances(&self) -> HashMap<String, f64> {
        let mut balances = HashMap::new();
        for i in 0..100 {
            let agent_id = format!("Agent_{}", i);
            let balance = self.get_agent_balance(&agent_id);
            if balance > 0.0 {
                balances.insert(agent_id, balance);
            }
        }
        balances
    }

    // ── TuringSwap helper: deduct coins from agent via WalletTool ──

    fn deduct_balance(&mut self, agent_id: &str, amount: f64) -> Result<(), String> {
        use crate::sdk::tools::wallet::WalletTool;
        for tool in &mut self.tools {
            if tool.manifest() == "core.tool.crypto_wallet" {
                if let Some(wallet) = tool.as_any_mut().downcast_mut::<WalletTool>() {
                    let bal = *wallet.balances.get(agent_id).unwrap_or(&0.0);
                    if bal < amount {
                        return Err(format!(
                            "Insufficient for citation: need {:.2}, have {:.2}", amount, bal
                        ));
                    }
                    *wallet.balances.get_mut(agent_id).unwrap() -= amount;
                    return Ok(());
                }
            }
        }
        Err("WalletTool not found".into())
    }

    // ── TuringSwap helper: add tokens to agent portfolio ──

    fn add_portfolio_tokens(&mut self, agent_id: &str, node_id: &str, tokens: f64) {
        use crate::sdk::tools::wallet::WalletTool;
        for tool in &mut self.tools {
            if tool.manifest() == "core.tool.crypto_wallet" {
                if let Some(wallet) = tool.as_any_mut().downcast_mut::<WalletTool>() {
                    *wallet.portfolios
                        .entry(agent_id.to_string()).or_default()
                        .entry(node_id.to_string()).or_insert(0.0) += tokens;
                }
                break;
            }
        }
    }

    /// Freeze the current universe state into an immutable snapshot.
    pub fn get_immutable_snapshot(&self) -> crate::sdk::snapshot::UniverseSnapshot {
        use crate::sdk::snapshot::PoolSnapshot;
        use crate::sdk::tools::wallet::WalletTool;

        let mut balances = HashMap::new();
        for i in 0..100 {
            let aid = format!("Agent_{}", i);
            balances.insert(aid.clone(), self.get_agent_balance(&aid));
        }
        let mut tombstones = HashMap::new();
        for id in self.kernel.tape.files.keys() {
            let g = self.get_tombstones(id);
            if !g.is_empty() { tombstones.insert(id.clone(), g); }
        }
        let rg = self.get_tombstones("root");
        if !rg.is_empty() { tombstones.insert("root".to_string(), rg); }

        // AMM pool snapshots
        let pool_states: HashMap<String, PoolSnapshot> = self.kernel.amms.iter()
            .map(|(nid, pool)| {
                (nid.clone(), PoolSnapshot {
                    coin_reserve: pool.coin_reserve,
                    token_reserve: pool.token_reserve,
                    spot_price: pool.spot_price(),
                    citation_cost_100: pool.get_amount_in(100.0).unwrap_or(f64::INFINITY),
                })
            })
            .collect();

        // Portfolio snapshot
        let mut portfolios = HashMap::new();
        for tool in &self.tools {
            if tool.manifest() == "core.tool.crypto_wallet" {
                if let Some(wallet) = tool.as_any().downcast_ref::<WalletTool>() {
                    portfolios = wallet.portfolios.clone();
                }
                break;
            }
        }

        crate::sdk::snapshot::UniverseSnapshot {
            tape: self.kernel.tape.clone(),
            balances,
            portfolios,
            pool_states,
            market_ticker: self.kernel.get_market_ticker(3),
            tombstones,
            generation: 0, // Evaluator overrides this with actual generation count
            bounty_remaining: self.kernel.bounty_escrow,
        }
    }

    pub fn halt_and_settle(&mut self, omega_id: &str) {
        use crate::sdk::tools::wallet::WalletTool;

        let golden_path = self.kernel.trace_golden_path(omega_id);

        // 1. Inject bounty escrow into Golden Path pools
        self.kernel.liquidate_bounty(&golden_path);
        log::info!(">>> [OMEGA SETTLEMENT] Bounty injected into {} GP pools", golden_path.len());

        // 2. Cash out all agents holding GP tokens
        // Collect portfolio data first to avoid borrow conflicts
        let mut cashouts: Vec<(String, String, f64)> = Vec::new(); // (agent, node, tokens)
        for tool in &self.tools {
            if tool.manifest() == "core.tool.crypto_wallet" {
                if let Some(wallet) = tool.as_any().downcast_ref::<WalletTool>() {
                    for (agent_id, holdings) in &wallet.portfolios {
                        for (nid, tokens) in holdings {
                            if *tokens > 0.0 && golden_path.contains(nid) {
                                cashouts.push((agent_id.clone(), nid.clone(), *tokens));
                            }
                        }
                    }
                }
                break;
            }
        }

        // Execute cashouts
        for (agent_id, nid, tokens) in &cashouts {
            if let Ok(coins) = self.kernel.sell_tokens(nid, *tokens) {
                // Credit balance
                for tool in &mut self.tools {
                    if tool.manifest() == "core.tool.crypto_wallet" {
                        if let Some(wallet) = tool.as_any_mut().downcast_mut::<WalletTool>() {
                            *wallet.balances.entry(agent_id.clone()).or_insert(0.0) += coins;
                            if let Some(holdings) = wallet.portfolios.get_mut(agent_id) {
                                holdings.insert(nid.clone(), 0.0);
                            }
                        }
                        break;
                    }
                }
                log::info!(">>> [CASH OUT] {} sold {:.0} tokens of {} for {:.2} Coins",
                    agent_id, tokens, nid, coins);
            }
        }

        // 3. Legacy tool halt hooks
        for tool in &mut self.tools {
            tool.on_halt(&golden_path, &mut self.kernel.tape);
        }

        self.kernel.refresh_prices();
    }

    pub fn append(&mut self, mut file: File) -> Result<(), String> {
        let mut final_reward = 0.0;
        let mut is_invest_only = false;
        let mut invest_target = String::new();
        let mut invest_amount = 0.0;

        // ── Phase 1: Tool pre-append hooks (balance check, deduction) ──
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
                    break;
                }
            }
        }

        // ── Phase 2: InvestOnly → AMM swap (replaces direct intrinsic_reward mutation) ──
        if is_invest_only {
            if self.kernel.amms.contains_key(&invest_target) {
                // TuringSwap path: buy tokens via AMM
                match self.kernel.buy_citation(&invest_target, invest_amount) {
                    Ok(tokens) => {
                        self.add_portfolio_tokens(&file.author, &invest_target, tokens);
                        log::info!(">>> [MARKET PUMP] {} bought {:.1} tokens of {} for {:.2} Coins",
                            file.author, tokens, invest_target, invest_amount);
                    }
                    Err(e) => {
                        log::warn!(">>> [AMM ERROR] {}", e);
                    }
                }
            } else {
                // No pool exists — investment cannot proceed in TuringSwap regime.
                // Do NOT mutate intrinsic_reward directly (violates Append-Only + SKILL-only minting).
                log::warn!(">>> [INVEST REJECTED] Node {} has no AMM pool. Investment of {:.2} returned to {}.",
                    invest_target, invest_amount, file.author);
                // Refund the agent
                use crate::sdk::tools::wallet::WalletTool;
                for tool in &mut self.tools {
                    if tool.manifest() == "core.tool.crypto_wallet" {
                        if let Some(wallet) = tool.as_any_mut().downcast_mut::<WalletTool>() {
                            *wallet.balances.entry(file.author.clone()).or_insert(0.0) += invest_amount;
                        }
                        break;
                    }
                }
            }
            self.kernel.refresh_prices();
            return Ok(());
        }

        // ── Phase 3: AMM citation purchase (引用即买入) ──
        let citation_token_amount = 100.0;
        for parent_id in &file.citations {
            if self.kernel.amms.contains_key(parent_id) {
                match self.kernel.quote_citation(parent_id, citation_token_amount) {
                    Ok(cost) => {
                        match self.deduct_balance(&file.author, cost) {
                            Ok(()) => {
                                match self.kernel.buy_citation(parent_id, cost) {
                                    Ok(tokens) => {
                                        self.add_portfolio_tokens(&file.author, parent_id, tokens);
                                        log::info!(">>> [CITATION BUY] {} bought {:.1} tokens of {} for {:.2}",
                                            file.author, tokens, parent_id, cost);
                                    }
                                    Err(e) => log::warn!(">>> [AMM BUY ERROR] {}", e),
                                }
                            }
                            Err(e) => {
                                log::warn!(">>> [CITATION COST] {} cannot afford {:.2} for {}: {}",
                                    file.author, cost, parent_id, e);
                                // Not fatal — citation still happens, just no token purchase
                            }
                        }
                    }
                    Err(e) => log::warn!(">>> [QUOTE ERROR] {}: {}", parent_id, e),
                }
            }
        }

        // ── Phase 4: Kernel append (unchanged — Append-Only DAG) ──
        let new_node_id = {
            let node = match self.kernel.append_tape(file.clone(), final_reward) {
                Ok(node) => node,
                Err(reason) => {
                    log::warn!(">>> [KERNEL REJECT] {}", reason);
                    return Err(reason);
                }
            };
            node.id.clone()
            // node ref dropped here — frees kernel borrow
        };

        // ── Phase 5: AMM pool creation (IDO) + founder tokens ──
        if final_reward > 0.0 {
            match self.kernel.create_pool(&new_node_id, final_reward) {
                Ok(()) => {
                    self.add_portfolio_tokens(&file.author, &new_node_id, 1000.0);
                    log::info!(">>> [IPO] {} launched pool for {} (IDO: {:.2}, Founder: 1000 tokens)",
                        file.author, new_node_id, final_reward);
                }
                Err(e) => log::warn!(">>> [POOL ERROR] {}", e),
            }
        }

        // ── Phase 6: Tool post-append hooks + price refresh ──
        let appended_node = self.kernel.tape.files.get(&new_node_id).unwrap();
        for tool in &mut self.tools {
            tool.on_post_append(&file.author, appended_node);
        }

        self.kernel.refresh_prices();
        self.clock += 1;
        Ok(())
    }

    /// Clock-driven price refresh. Same topology slot as old hayekian_map_reduce.
    pub fn tick_refresh_prices(&mut self) {
        self.kernel.refresh_prices();
    }

    /// Legacy: tick_map_reduce redirects to refresh_prices for backward compat
    pub fn tick_map_reduce(&mut self) {
        self.tick_refresh_prices();
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
            false
        } else {
            true
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
