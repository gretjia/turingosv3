use std::collections::{HashMap, HashSet};
use crate::amm::UniswapPool;

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
    /// TuringSwap: per-node AMM liquidity pools
    pub amms: HashMap<FileId, UniswapPool>,
    /// Bounty escrow: finite genesis budget, no fiat printing
    pub bounty_escrow: f64,
}

impl Kernel {
    pub fn new() -> Self {
        Self {
            tape: Tape::default(),
            gamma: 0.99,
            amms: HashMap::new(),
            bounty_escrow: 0.0,
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
        // Sort descending by market price
        active_nodes.sort_by(|a, b| b.price.partial_cmp(&a.price).unwrap_or(std::cmp::Ordering::Equal));
        
        let mut ticker = format!("\n=== 📈 GLOBAL MARKET LEADERBOARD (Top {}) ===\n", top_n);
        if active_nodes.is_empty() {
            ticker.push_str("- Market is empty. Be the first to IPO!\n");
        } else {
            for (i, node) in active_nodes.iter().take(top_n).enumerate() {
                ticker.push_str(&format!("Rank {}: [Node ID: {}] | Market Cap: {:.2} Coins\n", i + 1, node.id, node.price));
            }
        }
        ticker.push_str("============================================\n");
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

    // ── TuringSwap AMM Operations ──────────────────────────────────────

    /// Create an AMM pool for a newly appended node (IDO).
    pub fn create_pool(&mut self, node_id: &str, initial_coin: f64) -> Result<(), String> {
        if self.amms.contains_key(node_id) {
            return Err(format!("Pool already exists for {}", node_id));
        }
        let pool = UniswapPool::launch(node_id.to_string(), initial_coin)?;
        self.amms.insert(node_id.to_string(), pool);
        Ok(())
    }

    /// Quote: how many coins to buy `tokens` citation tokens from a node's pool?
    pub fn quote_citation(&self, node_id: &str, tokens: f64) -> Result<f64, String> {
        self.amms.get(node_id)
            .ok_or_else(|| format!("Pool not found: {}", node_id))?
            .get_amount_in(tokens)
    }

    /// Execute citation purchase: pay coins, receive tokens.
    pub fn buy_citation(&mut self, node_id: &str, coins_in: f64) -> Result<f64, String> {
        self.amms.get_mut(node_id)
            .ok_or_else(|| format!("Pool not found: {}", node_id))?
            .swap_coin_for_token(coins_in)
    }

    /// Sell tokens back to a pool for coins (founder cash-out).
    pub fn sell_tokens(&mut self, node_id: &str, tokens_in: f64) -> Result<f64, String> {
        self.amms.get_mut(node_id)
            .ok_or_else(|| format!("Pool not found: {}", node_id))?
            .swap_token_for_coin(tokens_in)
    }

    /// OMEGA settlement: inject bounty escrow into Golden Path pools.
    /// Only distributes to nodes that actually have pools.
    pub fn liquidate_bounty(&mut self, golden_path: &[String]) {
        if golden_path.is_empty() || self.bounty_escrow <= 0.0 { return; }
        let valid_nodes: Vec<&String> = golden_path.iter()
            .filter(|nid| self.amms.contains_key(*nid))
            .collect();
        if valid_nodes.is_empty() { return; }
        let per_node = self.bounty_escrow / valid_nodes.len() as f64;
        for nid in &valid_nodes {
            if let Some(pool) = self.amms.get_mut(*nid) {
                let _ = pool.inject_liquidity(per_node);
            }
        }
        self.bounty_escrow = 0.0;
    }

    /// Sync File.price from AMM pool state. Replaces hayekian_map_reduce.
    /// Called periodically by the clock heartbeat (same topology slot).
    pub fn refresh_prices(&mut self) {
        for (nid, pool) in &self.amms {
            if let Some(file) = self.tape.files.get_mut(nid) {
                file.price = pool.coin_reserve;
            }
        }
    }
}
