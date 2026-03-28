use std::collections::{HashMap, HashSet};
use crate::sdk::tool::{TuringTool, ToolSignal};
use crate::kernel::{Tape, File as TapeNode};

pub struct StakeRecord {
    pub agent_id: String,
    pub amount: f64,
    pub target_node: String, // "self" or specific node ID
}

pub struct WalletTool {
    pub balances: HashMap<String, f64>,
    /// Turing-Polymarket: agent YES/NO/LP holdings (agent_id -> node_id -> (yes, no, lp))
    pub portfolios: HashMap<String, HashMap<String, (f64, f64, f64)>>,
    pub stakes: Vec<StakeRecord>,
    pub global_pool: f64,
    pending_self_stakes: HashMap<String, f64>,
    /// Track agents who actually participated (staked) this theorem — no free riders
    participants: HashSet<String>,
}

impl WalletTool {
    pub fn new() -> Self {
        Self { balances: HashMap::new(), portfolios: HashMap::new(), stakes: Vec::new(), global_pool: 0.0, pending_self_stakes: HashMap::new(), participants: HashSet::new() }
    }

    // redistribute_pool: ABOLISHED (Magna Carta Law 2 — no central reallocation)
    // fund_agent: ABOLISHED (Magna Carta Law 2 — no post-genesis money printing)
    // The ONLY legal Coin injection is on_init GENESIS below.

    fn parse_payment(&self, payload: &str) -> Option<(String, f64)> {
        // Support both "Action: Invest" (preferred) and "Action: Stake" (backward compat)
        let tag_invest = "[Tool: Wallet | Action: Invest | Node: ";
        let tag_stake = "[Tool: Wallet | Action: Stake | Node: ";
        let (tag, start) = if let Some(pos) = payload.find(tag_invest) {
            (tag_invest, pos)
        } else if let Some(pos) = payload.find(tag_stake) {
            (tag_stake, pos)
        } else {
            return None;
        };
        let rest = &payload[start + tag.len()..];
        let node_end = rest.find(" | Amount: ")?;
        let target_node = rest[..node_end].trim().to_string();
        
        let amt_rest = &rest[node_end + 11..];
        let amt_end = amt_rest.find(']')?;
        let amount: f64 = amt_rest[..amt_end].trim().parse().unwrap_or(0.0);
        
        Some((target_node, amount))
    }
}

impl TuringTool for WalletTool {
    fn manifest(&self) -> &'static str { "core.tool.crypto_wallet" }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
    fn as_any(&self) -> &dyn std::any::Any { self }

    fn on_init(&mut self, agents: &[String]) {
        // GENESIS: One-time fixed allocation. This is the ONLY legal Coin injection.
        // After this, the system NEVER creates new Coins. (Magna Carta Law 2)
        // Total system Coins = agents.len() × 10,000 = constant forever.
        self.stakes.clear();
        for agent in agents {
            self.balances.insert(agent.clone(), 10000.0);
            log::info!(">>> [GENESIS] Agent {} allocated 10,000 Coins.", agent);
        }
    }

    fn on_pre_append(&mut self, author: &str, payload: &str) -> ToolSignal {
        if author == "System" { return ToolSignal::Pass; }

        // Magna Carta Law 1: append is FREE. Only invest costs money.
        // If no wallet tag present → free topology append → Pass through.
        // If wallet tag present but malformed → Veto (bad input).
        let has_wallet_tag = payload.contains("[Tool: Wallet");
        if !has_wallet_tag {
            return ToolSignal::Pass; // Free append — zero cost topology
        }

        let (target, amount) = match self.parse_payment(payload) {
            Some(cmd) => cmd,
            None => return ToolSignal::Veto("Malformed Wallet tag. Check syntax.".into()),
        };

        // Invest must be >= 1.0 (防粉尘攻击 on financial actions only)
        if amount < 1.0 {
            return ToolSignal::Veto("Invest amount must be >= 1.0.".into());
        }

        let balance = *self.balances.get(author).unwrap_or(&0.0);
        if balance < amount {
            if balance < 1.0 {
                return ToolSignal::Veto(format!("Bankrupt: Liquidated. Balance: {:.2}", balance));
            } else {
                return ToolSignal::Veto(format!(
                    "Margin Call: Insufficient liquidity. Balance: {:.2}, Requested: {:.2}. Reduce your stake!",
                    balance, amount
                ));
            }
        }

        self.participants.insert(author.to_string());

        // Deduct from agent balance (risk upfront)
        *self.balances.get_mut(author).unwrap() -= amount;
        // Note: Coins flow into CTF vault via prediction market, NOT into global_pool.

        if target.to_lowercase() == "self" {
            log::info!(">>> [SELF-INVEST] Agent {} invests {:.2} on own output. Balance after: {:.2}",
                       author, amount, self.balances.get(author).unwrap_or(&0.0));
            self.pending_self_stakes.insert(author.to_string(), amount);
            let clean_payload = payload.split("[Tool: Wallet").next().unwrap_or(payload).trim().to_string();
            
            // 🌟 资本即引力：大模型的自由报价，直接成为新节点的内生悬赏 (Intrinsic Reward)！
            ToolSignal::YieldReward {
                payload: clean_payload,
                reward: amount, 
            }
        } else {
            // Polymarket: detect direction from SHORT: prefix (策略层解析)
            use crate::sdk::tool::BetDirection;
            let (real_target, direction) = if target.starts_with("SHORT:") {
                (target["SHORT:".len()..].to_string(), BetDirection::Short)
            } else {
                (target.clone(), BetDirection::Long)
            };
            let side = if direction == BetDirection::Short { "SHORT" } else { "LONG" };
            self.stakes.push(StakeRecord { agent_id: author.to_string(), amount, target_node: real_target.clone() });
            log::info!(">>> [BET {}] Agent {} bet {:.2} Coins on Node {}!", side, author, amount, real_target);
            ToolSignal::InvestOnly { target_node: real_target, amount, direction }
        }
    }

    fn on_post_append(&mut self, author: &str, node: &TapeNode) {
        if let Some(amount) = self.pending_self_stakes.remove(author) {
            self.stakes.push(StakeRecord { agent_id: author.to_string(), amount, target_node: node.id.clone() });
        }
    }

    fn on_halt(&mut self, _golden_path: &[String], tape: &mut Tape) {
        // Polymarket regime: settlement is handled by bus.rs halt_and_settle()
        // via binary market resolution + redemption. Legacy global_pool payout DISABLED
        // to prevent double-payment (Codex audit #1 fix).
        log::info!(">>> [IMMUTABLE SPACETIME] Settlement complete. {} total nodes preserved.", tape.files.len());
        self.global_pool = 0.0;
        self.stakes.clear();
    }

    fn query_state(&self, key: &str) -> Option<String> {
        if key.starts_with("balance_") {
            let agent = key.trim_start_matches("balance_");
            return Some(self.balances.get(agent).unwrap_or(&0.0).to_string());
        }
        None
    }
}
