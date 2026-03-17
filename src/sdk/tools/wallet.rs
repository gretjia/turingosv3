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
}

impl WalletTool {
    pub fn new() -> Self {
        Self { balances: HashMap::new(), stakes: Vec::new(), global_pool: 0.0, pending_self_stakes: HashMap::new() }
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
            None => return ToolSignal::Veto("Payment Required: Missing Wallet Tool call.".into()),
        };

        if amount <= 0.0 {
            return ToolSignal::Veto("Invalid Transaction: Stake must be positive.".into());
        }

        let balance = *self.balances.get(author).unwrap_or(&0.0);
        if balance < amount {
            return ToolSignal::Veto(format!("Bankrupt: Insufficient funds. Balance: {:.2}", balance));
        }

        // Deduct
        *self.balances.get_mut(author).unwrap() -= amount;
        self.global_pool += amount;

        if target.to_lowercase() == "self" {
            self.pending_self_stakes.insert(author.to_string(), amount);
            // Clean payload
            let clean_payload = payload.split("[Tool: Wallet").next().unwrap_or(payload).trim().to_string();
            ToolSignal::Modify(clean_payload)
        } else {
            self.stakes.push(StakeRecord { agent_id: author.to_string(), amount, target_node: target.clone() });
            log::info!(">>> [WALLET VC] {} invested {:.2} in existing Node {}.", author, amount, target);
            ToolSignal::InvestOnly
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

        let initial_size = tape.files.len();
        tape.files.retain(|id, _| golden_set.contains(id));
        log::info!(">>> [GC] Dead branches pruned: {} nodes vaporized.", initial_size - tape.files.len());
        
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
