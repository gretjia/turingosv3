use std::collections::{HashMap, HashSet};
use crate::prediction_market::BinaryMarket;

pub type Token = u64;
pub type FileId = String;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MachineState {
    Running,
    Halt,
}

#[derive(Debug, Clone)]
pub struct File {
    pub id: FileId,
    pub author: String,
    pub payload: String,
    pub citations: Vec<FileId>,
    pub stake: Token,
    pub intrinsic_reward: f64,
    pub price: f64,
}

#[derive(Debug, Clone, Default)]
pub struct Tape {
    pub files: HashMap<FileId, File>,
    pub reverse_citations: HashMap<FileId, Vec<FileId>>,
    /// Time Arrow: insertion-order record of all node IDs.
    /// Since Tape is Append-Only, this is a natural topological sort of the DAG.
    pub time_arrow: Vec<FileId>,
}

// ----------------------------------------------------------------------------
// ONTOLOGY TYPES FOR EXPERIMENTS / AGENTS
// ----------------------------------------------------------------------------
#[derive(Debug, Clone)]
pub struct Head {
    pub paths: HashSet<FileId>,
}

#[derive(Debug, Clone)]
pub struct Q {
    pub q: MachineState,
    pub head: Head,
    pub tape: Tape,
}

pub struct SensorContext {
    pub visible_tape: Tape,
    pub current_head: Head,
    pub agent_balances: HashMap<String, f64>,
    pub market_ticker: String,
    pub tombstones: std::collections::HashMap<String, String>,
}

pub struct Input {
    pub q_i: MachineState,
    pub s_i: SensorContext,
}

pub struct Action {
    pub file_id: FileId,
    pub author: String,
    pub payload: String,
    pub citations: Vec<FileId>,
    pub stake: Token,
}

pub struct Output {
    pub q_o: MachineState,
    pub a_o: Action,
}

pub trait AIBlackBox {
    fn delta(&mut self, input: &Input) -> Output;
}
// ----------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct Kernel {
    pub tape: Tape,
    pub gamma: f64,
    /// Turing-Polymarket: per-node binary prediction markets
    pub prediction_markets: HashMap<FileId, BinaryMarket>,
}

impl Kernel {
    pub fn new() -> Self {
        Self {
            tape: Tape::default(),
            gamma: 0.99,
            prediction_markets: HashMap::new(),
        }
    }

    /// Traces from the OMEGA node backwards to the root to extract the Golden Path.
    pub fn trace_golden_path(&self, omega_node_id: &str) -> Vec<String> {
        let mut path = Vec::new();
        let mut current = omega_node_id.to_string();
        while let Some(node) = self.tape.files.get(&current) {
            path.push(current.clone());
            if node.citations.is_empty() { break; }
            current = node.citations[0].clone(); 
        }
        path
    }

    pub fn get_market_ticker(&self, top_n: usize) -> String {
        let mut active_nodes: Vec<_> = self.tape.files.values().collect();
        active_nodes.sort_by(|a, b| b.price.partial_cmp(&a.price).unwrap_or(std::cmp::Ordering::Equal));

        let mut ticker = format!("\n=== PREDICTION MARKET (Top {}) ===\n", top_n);
        if active_nodes.is_empty() {
            ticker.push_str("- No nodes yet. Be the first to create one!\n");
        } else {
            for (i, node) in active_nodes.iter().take(top_n).enumerate() {
                if let Some(market) = self.prediction_markets.get(&node.id) {
                    if let Some(yes_wins) = market.resolved {
                        let verdict = if yes_wins { "YES" } else { "NO" };
                        ticker.push_str(&format!("#{}: [{}] RESOLVED: {}\n", i + 1, node.id, verdict));
                    } else {
                        ticker.push_str(&format!("#{}: [{}] P_yes: {:.1}%\n", i + 1, node.id, node.price * 100.0));
                    }
                } else {
                    ticker.push_str(&format!("#{}: [{}] Price: {:.2}\n", i + 1, node.id, node.price));
                }
            }
        }
        ticker.push_str("=================================\n");
        ticker
    }

    /// Append a node to the Tape. Returns Err if causality is violated.
    /// - Rejects duplicate FileId (V6: prevents reverse_citations corruption)
    /// - Strips citations to non-existent nodes (V5: prevents cycles)
    pub(crate) fn append_tape(&mut self, mut file: File, reward: f64) -> Result<&File, String> {
        let id = file.id.clone();

        // V6: Reject duplicate FileId — prevent spacetime overwrite
        if self.tape.files.contains_key(&id) {
            return Err(format!("Spacetime Paradox: Node ID {} already exists.", id));
        }

        // V5: Causality defense — only allow citations to existing nodes
        let mut valid_citations = Vec::new();
        for cid in &file.citations {
            if self.tape.files.contains_key(cid) {
                valid_citations.push(cid.clone());
            } else {
                log::warn!(">>> [CAUSALITY] Stripping ghost citation: {} cited non-existent {}", id, cid);
            }
        }
        file.citations = valid_citations;

        file.intrinsic_reward = reward;
        file.price = reward;

        for parent_id in &file.citations {
            self.tape.reverse_citations
                .entry(parent_id.clone())
                .or_default()
                .push(id.clone());
        }

        self.tape.files.insert(id.clone(), file);
        self.tape.time_arrow.push(id.clone());

        Ok(self.tape.files.get(&id).unwrap())
    }

    // ── Turing-Polymarket Operations ────────────────────────────────────

    /// Create a binary prediction market for a node (genesis ignition).
    pub fn create_market(&mut self, node_id: &str, lp_coins: f64) -> Result<(), String> {
        if self.prediction_markets.contains_key(node_id) {
            return Err(format!("Market already exists for {}", node_id));
        }
        let market = BinaryMarket::create(node_id.to_string(), lp_coins)?;
        self.prediction_markets.insert(node_id.to_string(), market);
        Ok(())
    }

    /// Buy YES shares on a node (bullish: believes node is on GP).
    pub fn buy_yes(&mut self, node_id: &str, coins: f64) -> Result<f64, String> {
        self.prediction_markets.get_mut(node_id)
            .ok_or_else(|| format!("Market not found: {}", node_id))?
            .buy_yes(coins)
    }

    /// Buy NO shares on a node (bearish: believes node is NOT on GP).
    pub fn buy_no(&mut self, node_id: &str, coins: f64) -> Result<f64, String> {
        self.prediction_markets.get_mut(node_id)
            .ok_or_else(|| format!("Market not found: {}", node_id))?
            .buy_no(coins)
    }

    /// Oracle resolution: mark a node's market as resolved.
    pub fn resolve_market(&mut self, node_id: &str, yes_wins: bool) {
        if let Some(market) = self.prediction_markets.get_mut(node_id) {
            market.resolve(yes_wins);
        }
    }

    /// Get YES price (Bayesian probability) for a node.
    pub fn yes_price(&self, node_id: &str) -> f64 {
        self.prediction_markets.get(node_id)
            .map(|m| m.yes_price())
            .unwrap_or(0.0)
    }

    /// Redeem winning shares after resolution.
    pub fn redeem(&self, node_id: &str, yes_shares: f64, no_shares: f64) -> f64 {
        self.prediction_markets.get(node_id)
            .map(|m| m.redeem(yes_shares, no_shares))
            .unwrap_or(0.0)
    }

    /// Compute LP withdrawal: what YES/NO does the LP get from the pool.
    /// Does NOT mutate pool state — bus.rs handles the actual withdrawal.
    pub fn compute_lp_withdrawal(&self, node_id: &str, lp_fraction: f64) -> (f64, f64) {
        if let Some(m) = self.prediction_markets.get(node_id) {
            if m.lp_total == 0.0 { return (0.0, 0.0); }
            (m.yes_reserve * lp_fraction, m.no_reserve * lp_fraction)
        } else {
            (0.0, 0.0)
        }
    }

    /// Actually deduct LP withdrawal from pool reserves.
    pub fn execute_lp_withdrawal(&mut self, node_id: &str, lp_fraction: f64) -> (f64, f64) {
        if let Some(m) = self.prediction_markets.get_mut(node_id) {
            if m.lp_total == 0.0 { return (0.0, 0.0); }
            let yes_out = m.yes_reserve * lp_fraction;
            let no_out = m.no_reserve * lp_fraction;
            m.yes_reserve -= yes_out;
            m.no_reserve -= no_out;
            m.lp_total -= lp_fraction.min(m.lp_total);
            (yes_out, no_out)
        } else {
            (0.0, 0.0)
        }
    }

    /// Sync File.price from prediction market probabilities.
    /// Price = P_yes (Bayesian confidence that node is on GP).
    /// Called periodically by the clock heartbeat (same topology slot).
    pub fn refresh_prices(&mut self) {
        for (nid, market) in &self.prediction_markets {
            if let Some(file) = self.tape.files.get_mut(nid) {
                file.price = market.yes_price();
            }
        }
    }
}
