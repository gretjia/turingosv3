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
    pub stakes: Vec<StakeRecord>,
    pub global_pool: f64,
    pending_self_stakes: HashMap<String, f64>,
    /// Track agents who actually participated (staked) this theorem — no free riders
    participants: HashSet<String>,
}

impl WalletTool {
    pub fn new() -> Self {
        Self { balances: HashMap::new(), stakes: Vec::new(), global_pool: 0.0, pending_self_stakes: HashMap::new(), participants: HashSet::new() }
    }

    /// Redistribute global_pool among PARTICIPANTS only (agents who staked this theorem).
    /// Hayekian principle: no labor, no pay. Idle agents get nothing.
    pub fn redistribute_pool(&mut self) {
        let eligible: Vec<String> = self.participants.iter()
            .filter(|id| self.balances.get(*id).copied().unwrap_or(0.0) >= 1.0)
            .cloned()
            .collect();
        if eligible.is_empty() || self.global_pool <= 0.0 { return; }
        let share = self.global_pool / eligible.len() as f64;
        for id in &eligible {
            *self.balances.get_mut(id).unwrap() += share;
        }
        log::info!(">>> [REDISTRIBUTION] Pool {:.2} split among {} participants ({:.2} each). {} idle agents excluded.",
                   self.global_pool, eligible.len(), share, self.balances.len() - eligible.len());
        self.global_pool = 0.0;
        self.participants.clear();
    }

    fn parse_payment(&self, payload: &str) -> Option<(String, f64)> {
        let tag = "[Tool: Wallet | Action: Stake | Node: ";
        let start = payload.find(tag)?;
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

    fn on_init(&mut self, agents: &[String]) {
        self.stakes.clear();
        self.global_pool = 0.0;
        for agent in agents {
            self.balances.insert(agent.clone(), 10000.0);
            log::info!(">>> [WALLET] Agent {} funded with 10,000 Coins.", agent);
        }
    }

    fn on_pre_append(&mut self, author: &str, payload: &str) -> ToolSignal {
        // Only enforce wallet calls for non-system agents.
        if author == "System" { return ToolSignal::Pass; }

        let (target, amount) = match self.parse_payment(payload) {
            Some(cmd) => cmd,
            None => return ToolSignal::Veto("Bankruptcy/Fraud: Missing Wallet Tool call.".into()),
        };

        // 【奥地利学派的底线】：自由出价，但拒绝零元购（防粉尘攻击）
        if amount < 1.0 {
            return ToolSignal::Veto("Market Rule Violation: Stake must be >= 1.0. No free lunch.".into());
        }

        let balance = *self.balances.get(author).unwrap_or(&0.0);
        if balance < amount {
            return ToolSignal::Veto(format!("Bankrupt: Insufficient funds. Balance: {:.2}", balance));
        }

        // Record participation — this agent is working, not free-riding
        self.participants.insert(author.to_string());

        // 🌟 物理扣款（风险前置）！
        *self.balances.get_mut(author).unwrap() -= amount;
        self.global_pool += amount;

        if target.to_lowercase() == "self" {
            log::info!(">>> [SELF-STAKE] Agent {} stakes {:.2} on own output. Balance after: {:.2}",
                       author, amount, self.balances.get(author).unwrap_or(&0.0));
            self.pending_self_stakes.insert(author.to_string(), amount);
            let clean_payload = payload.split("[Tool: Wallet").next().unwrap_or(payload).trim().to_string();
            
            // 🌟 资本即引力：大模型的自由报价，直接成为新节点的内生悬赏 (Intrinsic Reward)！
            ToolSignal::YieldReward {
                payload: clean_payload,
                reward: amount, 
            }
        } else {
            // 纯金融 VC 投资行为：记录股权，等待 Halt 时的赢家通吃结算
            self.stakes.push(StakeRecord { agent_id: author.to_string(), amount, target_node: target.clone() });
            log::info!(">>> [FREE MARKET VC] Agent {} invested {:.2} Coins into Node {}!", author, amount, target);
            ToolSignal::InvestOnly { target_node: target, amount }
        }
    }

    fn on_post_append(&mut self, author: &str, node: &TapeNode) {
        if let Some(amount) = self.pending_self_stakes.remove(author) {
            self.stakes.push(StakeRecord { agent_id: author.to_string(), amount, target_node: node.id.clone() });
        }
    }

    fn on_halt(&mut self, golden_path: &[String], tape: &mut Tape) {
        log::info!("==== [SMART CONTRACT] HALT REACHED! INITIATING SETTLEMENT ====");
        let golden_set: HashSet<_> = golden_path.iter().cloned().collect();
        
        let mut golden_stakes_total = 0.0;
        let mut agent_winning_amounts = HashMap::new();

        for stake in &self.stakes {
            if golden_set.contains(&stake.target_node) {
                golden_stakes_total += stake.amount;
                *agent_winning_amounts.entry(stake.agent_id.clone()).or_insert(0.0) += stake.amount;
            }
        }

        if golden_stakes_total > 0.0 {
            for (agent, amount) in agent_winning_amounts {
                let share = (amount / golden_stakes_total) * self.global_pool;
                *self.balances.entry(agent.clone()).or_insert(0.0) += share;
                log::info!(">>> [PAYOUT] Agent {} won {:.2} Coins! (Pool Share: {:.2}%)", agent, share, (amount/golden_stakes_total)*100.0);
            }
        }

        // V1 fix: Tape is Append-Only. No node deletion — ever.
        // Dead branches are permanently archived as RLAIF training data.
        log::info!(">>> [IMMUTABLE SPACETIME] Settlement complete. {} total nodes preserved.", tape.files.len());

        self.global_pool = 0.0;
    }

    fn query_state(&self, key: &str) -> Option<String> {
        if key.starts_with("balance_") {
            let agent = key.trim_start_matches("balance_");
            return Some(self.balances.get(agent).unwrap_or(&0.0).to_string());
        }
        None
    }
}
