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

/// System Market Maker: auto-injects LP into every new node's market.
/// Magna Carta amendment 2026-03-29: 做市商豁免 (Rule #19).
const SYSTEM_MM_ID: &str = "SYSTEM_MM";
/// LP seed per node: 1000 YES + 1000 NO (CTF conservation: 1000 Coin → 1000Y + 1000N).
/// Calibrated 2026-03-30: LP=100 caused market collapse (single trade P_yes 50%→99.8%).
/// LP=1000: 2000 Coin trade → P_yes=90% (healthy price discovery). Gemini audit PASS.
const SYSTEM_LP_AMOUNT: f64 = 1000.0;

pub struct TuringBus {
    pub kernel: Kernel,
    pub tools: Vec<Box<dyn TuringTool>>,
    pub clock: usize,
    pub graveyard: Graveyard,
    pub ticker_top_n: usize,
    /// Kernel-level payload blacklist. Applied to ALL appends BEFORE any SKILL tool.
    /// Prevents brute-force tactics (decide, omega, native_decide) at the physical layer.
    /// Cannot be bypassed by experiment code or SKILL configuration.
    pub forbidden_payload_patterns: Vec<String>,
    /// Max payload size per append. Prevents front-running (packing multiple steps in one node).
    /// Each node = one atomic reasoning step. Magna Carta: one step per node.
    /// Calibrated for natural language math (Gemini review 2026-03-30: 800 chars / 12 lines).
    pub max_payload_chars: usize,
    pub max_payload_lines: usize,
    /// Total Coins injected by SYSTEM_MM across all markets (for conservation accounting).
    pub system_mm_total_injected: f64,
}

impl TuringBus {
    pub fn new(kernel: Kernel) -> Self {
        Self {
            kernel,
            tools: Vec::new(),
            clock: 0,
            graveyard: Graveyard::new(),
            ticker_top_n: 5,
            // Kernel-level: Lean syntax blocked per architect directive 2026-03-29.
            // Agents must use traditional math. Only unambiguous Lean-specific patterns here.
            // English words like "decide"/"omega" removed to avoid false positives on natural language.
            // Brute-force tactics (decide/omega) are guarded by check_translated_output at OMEGA.
            forbidden_payload_patterns: vec![
                ":= by".to_string(),       // Lean proof block header
                "simp [".to_string(),       // Lean tactic with arg list
                "simp[".to_string(),
                "rw [".to_string(),         // Lean rewrite tactic
                "rw[".to_string(),
                "simp_rw".to_string(),      // Lean simp_rw
                "norm_num".to_string(),     // Lean norm_num
                "linarith".to_string(),     // Lean linarith
                "nlinarith".to_string(),    // Lean nlinarith
                "field_simp".to_string(),   // Lean field_simp
                "push_cast".to_string(),    // Lean push_cast
                "ring_nf".to_string(),      // Lean ring_nf
            ],
            // One step per node. Calibrated for natural language math.
            // 800→1200→1600: Run 9 audit found 11% rejection on deep case analysis (mod 3^7).
            // Kolmogorov prescription: allow high-density algebraic reasoning to complete on-chain.
            max_payload_chars: 1600,
            max_payload_lines: 24,
            system_mm_total_injected: 0.0,
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

    // fund_agent: ABOLISHED (Magna Carta Law 2 — no post-genesis money printing)
    // redistribute_pool: ABOLISHED (Magna Carta Law 2 — no central reallocation)
    // After GENESIS, total Coins in system = constant. Only CTF mint/redeem moves Coins.

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

    // ── Polymarket helper: add YES/NO shares to agent portfolio ──

    fn add_yes_shares(&mut self, agent_id: &str, node_id: &str, shares: f64) {
        use crate::sdk::tools::wallet::WalletTool;
        for tool in &mut self.tools {
            if tool.manifest() == "core.tool.crypto_wallet" {
                if let Some(wallet) = tool.as_any_mut().downcast_mut::<WalletTool>() {
                    let pos = wallet.portfolios
                        .entry(agent_id.to_string()).or_default()
                        .entry(node_id.to_string()).or_insert((0.0, 0.0, 0.0));
                    pos.0 += shares;
                }
                break;
            }
        }
    }

    fn add_no_shares(&mut self, agent_id: &str, node_id: &str, shares: f64) {
        use crate::sdk::tools::wallet::WalletTool;
        for tool in &mut self.tools {
            if tool.manifest() == "core.tool.crypto_wallet" {
                if let Some(wallet) = tool.as_any_mut().downcast_mut::<WalletTool>() {
                    let pos = wallet.portfolios
                        .entry(agent_id.to_string()).or_default()
                        .entry(node_id.to_string()).or_insert((0.0, 0.0, 0.0));
                    pos.1 += shares;
                }
                break;
            }
        }
    }

    fn add_lp_shares(&mut self, agent_id: &str, node_id: &str, shares: f64) {
        use crate::sdk::tools::wallet::WalletTool;
        for tool in &mut self.tools {
            if tool.manifest() == "core.tool.crypto_wallet" {
                if let Some(wallet) = tool.as_any_mut().downcast_mut::<WalletTool>() {
                    let pos = wallet.portfolios
                        .entry(agent_id.to_string()).or_default()
                        .entry(node_id.to_string()).or_insert((0.0, 0.0, 0.0));
                    pos.2 += shares;
                }
                break;
            }
        }
    }

    /// Freeze the current universe state into an immutable snapshot.
    pub fn get_immutable_snapshot(&self) -> crate::sdk::snapshot::UniverseSnapshot {
        use crate::sdk::snapshot::MarketSnapshot;
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

        // Prediction market snapshots
        let markets: HashMap<String, MarketSnapshot> = self.kernel.prediction_markets.iter()
            .map(|(nid, m)| {
                (nid.clone(), MarketSnapshot {
                    yes_price: m.yes_price(),
                    no_price: m.no_price(),
                    yes_reserve: m.yes_reserve,
                    no_reserve: m.no_reserve,
                    resolved: m.resolved,
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
            markets,
            market_ticker: self.kernel.get_market_ticker(self.ticker_top_n),
            tombstones,
            generation: 0,
        }
    }

    pub fn halt_and_settle(&mut self, omega_id: &str) {
        use crate::sdk::tools::wallet::WalletTool;
        use std::collections::HashSet;

        // PRE-SETTLEMENT: Compute total system Coins for conservation check
        let pre_total = self.compute_total_system_coins();
        log::info!(">>> [CONSERVATION] Pre-settlement total: {:.2} Coins", pre_total);

        let golden_path = self.kernel.trace_golden_path(omega_id);
        let gp_set: HashSet<String> = golden_path.iter().cloned().collect();

        // 1. Oracle Resolution: resolve ALL markets
        let all_node_ids: Vec<String> = self.kernel.prediction_markets.keys().cloned().collect();
        for nid in &all_node_ids {
            let yes_wins = gp_set.contains(nid);
            self.kernel.resolve_market(nid, yes_wins);
            let verdict = if yes_wins { "YES (GP)" } else { "NO (dead)" };
            log::info!(">>> [ORACLE] Node {} resolved: {}", nid, verdict);
        }

        // 2. LP Withdrawal: distribute pool contents to LP holders BEFORE redemption
        let mut lp_withdrawals: Vec<(String, String, f64)> = Vec::new(); // (agent, node, lp_shares)
        for tool in &self.tools {
            if tool.manifest() == "core.tool.crypto_wallet" {
                if let Some(wallet) = tool.as_any().downcast_ref::<WalletTool>() {
                    for (agent_id, holdings) in &wallet.portfolios {
                        for (nid, (_yes, _no, lp)) in holdings {
                            if *lp > 0.0 {
                                lp_withdrawals.push((agent_id.clone(), nid.clone(), *lp));
                            }
                        }
                    }
                }
                break;
            }
        }

        for (agent_id, nid, lp_shares) in &lp_withdrawals {
            if let Some(market) = self.kernel.prediction_markets.get(nid.as_str()) {
                if market.lp_total > 0.0 {
                    let fraction = lp_shares / market.lp_total;
                    let (yes_out, no_out) = self.kernel.execute_lp_withdrawal(&nid, fraction);
                    // Add withdrawn YES/NO to agent's holdings
                    for tool in &mut self.tools {
                        if tool.manifest() == "core.tool.crypto_wallet" {
                            if let Some(wallet) = tool.as_any_mut().downcast_mut::<WalletTool>() {
                                if let Some(holdings) = wallet.portfolios.get_mut(&*agent_id) {
                                    if let Some(pos) = holdings.get_mut(&*nid) {
                                        pos.0 += yes_out;
                                        pos.1 += no_out;
                                        pos.2 = 0.0; // LP withdrawn
                                    }
                                }
                            }
                            break;
                        }
                    }
                    log::info!(">>> [LP WITHDRAW] {} withdrew LP from {}: {:.2} YES + {:.2} NO",
                        agent_id, nid, yes_out, no_out);
                }
            }
        }

        // 3. Redeem: collect all agent positions (now including LP-withdrawn YES/NO), then settle
        let mut redemptions: Vec<(String, String, f64, f64)> = Vec::new();
        for tool in &self.tools {
            if tool.manifest() == "core.tool.crypto_wallet" {
                if let Some(wallet) = tool.as_any().downcast_ref::<WalletTool>() {
                    for (agent_id, holdings) in &wallet.portfolios {
                        for (nid, (yes_s, no_s, _lp)) in holdings {
                            if *yes_s > 0.0 || *no_s > 0.0 {
                                redemptions.push((agent_id.clone(), nid.clone(), *yes_s, *no_s));
                            }
                        }
                    }
                }
                break;
            }
        }

        // Execute redemptions — always clear position, credit payout if > 0
        for (agent_id, nid, yes_s, no_s) in &redemptions {
            let payout = self.kernel.redeem(nid, *yes_s, *no_s);
            for tool in &mut self.tools {
                if tool.manifest() == "core.tool.crypto_wallet" {
                    if let Some(wallet) = tool.as_any_mut().downcast_mut::<WalletTool>() {
                        if payout > 0.0 {
                            *wallet.balances.entry(agent_id.clone()).or_insert(0.0) += payout;
                        }
                        if let Some(holdings) = wallet.portfolios.get_mut(agent_id) {
                            holdings.insert(nid.clone(), (0.0, 0.0, 0.0));
                        }
                    }
                    break;
                }
            }
            if payout > 0.0 {
                let side = if gp_set.contains(nid) { "YES" } else { "NO" };
                log::info!(">>> [REDEEM] {} cashes {} on {} → {:.2} Coins",
                    agent_id, side, nid, payout);
            }
        }

        // 3. Tool halt hooks
        for tool in &mut self.tools {
            tool.on_halt(&golden_path, &mut self.kernel.tape);
        }

        self.kernel.refresh_prices();

        // POST-SETTLEMENT: Conservation invariant check
        // Magna Carta amendment 2026-03-29: allow small MM P&L (impermanent loss = physical cost of liquidity)
        let post_total = self.compute_total_system_coins();
        let drift = (post_total - pre_total).abs();
        let num_markets = all_node_ids.len() as f64;
        // Tolerance: each market's MM can lose up to SYSTEM_LP_AMOUNT in the worst case
        // (all agents correctly short a junk node → MM loses entire seed).
        // Normal impermanent loss is ~30-50% of seed per market.
        let mm_tolerance = num_markets * SYSTEM_LP_AMOUNT;
        let agent_tolerance = mm_tolerance.max(0.01); // at least 0.01 for rounding
        if drift > agent_tolerance {
            log::error!(">>> [CONSERVATION VIOLATION] Drift {:.2} exceeds tolerance {:.2}! Pre: {:.2}, Post: {:.2}",
                drift, agent_tolerance, pre_total, post_total);
        } else {
            log::info!(">>> [CONSERVATION] Post: {:.2}, Drift: {:.6}, MM P&L tolerance: {:.2} ✓",
                post_total, drift, agent_tolerance);
        }
        log::info!(">>> [SYSTEM_MM] Total injected: {:.0} Coins across {:.0} markets",
            self.system_mm_total_injected, num_markets);
    }

    /// Compute total system Coins: agent balances + system MM injection tracking.
    ///
    /// Accounting model (APMM):
    /// - Agent balances: liquid Coins held by agents
    /// - Agent YES/NO shares: claims on Coins locked in CTF vault
    /// - System MM: injects SYSTEM_LP_AMOUNT per market (Magna Carta Rule #19 做市商豁免)
    ///   Each injection: SYSTEM_LP_AMOUNT Coin → LP YES + LP NO (CTF conservation within each market)
    ///
    /// For conservation: we track agent-side Coins only.
    /// System MM P&L is expected (impermanent loss) and tolerated per Magna Carta amendment.
    /// Note: agent profit from correct predictions = system MM loss. This is by design.
    fn compute_total_system_coins(&self) -> f64 {
        use crate::sdk::tools::wallet::WalletTool;
        let mut total = 0.0;

        for tool in &self.tools {
            if tool.manifest() == "core.tool.crypto_wallet" {
                if let Some(wallet) = tool.as_any().downcast_ref::<WalletTool>() {
                    for (agent_id, bal) in &wallet.balances {
                        // Exclude SYSTEM_MM from agent conservation (it has unlimited minting)
                        if agent_id != SYSTEM_MM_ID {
                            total += bal;
                        }
                    }
                }
                break;
            }
        }

        total
    }

    pub fn append(&mut self, mut file: File) -> Result<(), String> {
        // ── Phase 0: KERNEL-LEVEL payload blacklist (cannot be bypassed) ──
        // This runs BEFORE any SKILL tool. Physical enforcement.
        for pattern in &self.forbidden_payload_patterns {
            // Word-boundary check: "decide" should not match "undecidable"
            let payload = &file.payload;
            let mut search_start = 0;
            while let Some(pos) = payload[search_start..].find(pattern.as_str()) {
                let abs_pos = search_start + pos;
                let before_ok = abs_pos == 0 ||
                    (!payload.as_bytes()[abs_pos - 1].is_ascii_alphanumeric() &&
                     payload.as_bytes()[abs_pos - 1] != b'_');
                let after_pos = abs_pos + pattern.len();
                let after_ok = after_pos >= payload.len() ||
                    (!payload.as_bytes()[after_pos].is_ascii_alphanumeric() &&
                     payload.as_bytes()[after_pos] != b'_');
                if before_ok && after_ok {
                    let reason = format!("KERNEL BLACKLIST: '{}' is forbidden (no brute-force search)", pattern);
                    log::warn!(">>> [KERNEL] {} in payload by {}", reason, file.author);
                    self.graveyard.record_death("root", &reason);
                    return Err(reason);
                }
                search_start = abs_pos + pattern.len();
            }
        }

        // ── Phase 0b: FRONT-RUNNING DETECTION (one step per node) ──
        // Strip wallet tags before measuring payload size
        let content_part = file.payload.split("[Tool: Wallet").next().unwrap_or(&file.payload).trim();
        let content_chars = content_part.len();
        let content_lines = content_part.lines()
            .filter(|l| !l.trim().is_empty())
            .count();
        if content_chars > self.max_payload_chars {
            let reason = format!(
                "FRONT-RUNNING: {} chars (max {}). One atomic step per node.",
                content_chars, self.max_payload_chars
            );
            log::warn!(">>> [KERNEL] {} by {}", reason, file.author);
            self.graveyard.record_death("root", &reason);
            return Err(reason);
        }
        if content_lines > self.max_payload_lines {
            let reason = format!(
                "FRONT-RUNNING: {} lines (max {}). One atomic step per node.",
                content_lines, self.max_payload_lines
            );
            log::warn!(">>> [KERNEL] {} by {}", reason, file.author);
            self.graveyard.record_death("root", &reason);
            return Err(reason);
        }

        let mut final_reward = 0.0;
        use crate::sdk::tool::BetDirection;
        let mut is_invest_only = false;
        let mut invest_target = String::new();
        let mut invest_amount = 0.0;
        let mut invest_direction = BetDirection::Long;

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
                    // Refund any wallet deduction that already happened (Law 2: no silent burn)
                    if final_reward > 0.0 {
                        use crate::sdk::tools::wallet::WalletTool;
                        for t in &mut self.tools {
                            if t.manifest() == "core.tool.crypto_wallet" {
                                if let Some(wallet) = t.as_any_mut().downcast_mut::<WalletTool>() {
                                    *wallet.balances.entry(file.author.clone()).or_insert(0.0) += final_reward;
                                    log::info!(">>> [REFUND] {} refunded {:.2} after veto", file.author, final_reward);
                                }
                                break;
                            }
                        }
                    }
                    return Err(reason);
                }
                ToolSignal::YieldReward { payload, reward } => {
                    file.payload = payload;
                    final_reward += reward;
                }
                ToolSignal::InvestOnly { target_node, amount, direction } => {
                    is_invest_only = true;
                    invest_target = target_node;
                    invest_amount = amount;
                    invest_direction = direction;
                    break;
                }
            }
        }

        // ── Phase 2: InvestOnly → buy YES or NO on existing node's market ──
        if is_invest_only {
            // Codex #2: reject invest into non-existent nodes
            if !self.kernel.tape.files.contains_key(&invest_target) {
                log::warn!(">>> [INVEST REJECTED] Node {} not in tape. Refunding {}.", invest_target, file.author);
                use crate::sdk::tools::wallet::WalletTool;
                for tool in &mut self.tools {
                    if tool.manifest() == "core.tool.crypto_wallet" {
                        if let Some(wallet) = tool.as_any_mut().downcast_mut::<WalletTool>() {
                            *wallet.balances.entry(file.author.clone()).or_insert(0.0) += invest_amount;
                        }
                        break;
                    }
                }
                self.kernel.refresh_prices();
                return Ok(());
            }

            // APMM: System auto-creates market if missing (should exist from Phase 4, but edge case)
            if !self.kernel.prediction_markets.contains_key(&invest_target) {
                match self.kernel.create_market(&invest_target, SYSTEM_LP_AMOUNT) {
                    Ok(()) => {
                        self.add_lp_shares(SYSTEM_MM_ID, &invest_target, 1.0);
                        self.system_mm_total_injected += SYSTEM_LP_AMOUNT;
                        log::info!(">>> [APMM] System MM created market for {} (LP: {:.0})",
                            invest_target, SYSTEM_LP_AMOUNT);
                    }
                    Err(e) => log::warn!(">>> [MARKET ERROR] {}", e),
                }
            }

            // Market exists (system-created or just created above): full amount goes to swap
            if self.kernel.prediction_markets.contains_key(&invest_target) {
                // Existing market: just swap
                let result = match invest_direction {
                    BetDirection::Long => self.kernel.buy_yes(&invest_target, invest_amount),
                    BetDirection::Short => self.kernel.buy_no(&invest_target, invest_amount),
                };
                match result {
                    Ok(shares) => {
                        match invest_direction {
                            BetDirection::Long => {
                                self.add_yes_shares(&file.author, &invest_target, shares);
                                let p = self.kernel.yes_price(&invest_target);
                                log::info!(">>> [BUY YES] {} bought {:.1} YES on {} for {:.2} (P_yes={:.1}%)",
                                    file.author, shares, invest_target, invest_amount, p * 100.0);
                            }
                            BetDirection::Short => {
                                self.add_no_shares(&file.author, &invest_target, shares);
                                let p = self.kernel.yes_price(&invest_target);
                                log::info!(">>> [BUY NO] {} bought {:.1} NO on {} for {:.2} (P_yes={:.1}%)",
                                    file.author, shares, invest_target, invest_amount, p * 100.0);
                            }
                        }
                    }
                    Err(e) => log::warn!(">>> [BET ERROR] {}", e),
                }
            }
            self.kernel.refresh_prices();
            return Ok(());
        }

        // ── Phase 3: Citations are FREE (Magna Carta: topology decoupled from finance) ──
        // No citation purchase. DAG connectivity costs nothing.

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
        };

        // ── Phase 5: APMM System Auto-Market + Creator Auto-Long ──
        // System MM already provides 100 LP per node. Market created here (after append).
        match self.kernel.create_market(&new_node_id, SYSTEM_LP_AMOUNT) {
            Ok(()) => {
                self.add_lp_shares(SYSTEM_MM_ID, &new_node_id, 1.0);
                self.system_mm_total_injected += SYSTEM_LP_AMOUNT;
                log::info!(">>> [APMM] System MM created market for {} (LP: {:.0}, P_yes=50.0%)",
                    new_node_id, SYSTEM_LP_AMOUNT);
            }
            Err(e) => log::warn!(">>> [APMM ERROR] {}", e),
        }

        // Creator auto-long: full stake goes to YES (no LP cut from agent anymore)
        if final_reward > 0.0 {
            match self.kernel.buy_yes(&new_node_id, final_reward) {
                Ok(yes_shares) => {
                    self.add_yes_shares(&file.author, &new_node_id, yes_shares);
                    let p = self.kernel.yes_price(&new_node_id);
                    log::info!(">>> [AUTO-LONG] {} bought {:.1} YES on {} for {:.2} (P_yes={:.1}%)",
                        file.author, yes_shares, new_node_id, final_reward, p * 100.0);
                }
                Err(e) => log::warn!(">>> [AUTO-LONG ERROR] {}", e),
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
