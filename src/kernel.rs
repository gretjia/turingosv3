use std::collections::{HashMap, HashSet};

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
}

impl Kernel {
    pub fn new() -> Self {
        Self {
            tape: Tape::default(),
            gamma: 0.99,
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

    /// O(V+E) Time-Arrow backpropagation.
    /// Since Tape is Append-Only, time_arrow is a natural topological sort.
    /// Iterating in reverse guarantees all children are settled before their parents.
    pub fn hayekian_map_reduce(&mut self) {
        // Step 1: Reset prices to intrinsic reward
        for node in self.tape.files.values_mut() {
            node.price = node.intrinsic_reward;
        }

        // Step 2: Single-pass reverse Time-Arrow propagation
        for id in self.tape.time_arrow.iter().rev() {
            let mut imputed_val = 0.0;

            if let Some(children) = self.tape.reverse_citations.get(id) {
                for child_id in children {
                    if let Some(child_file) = self.tape.files.get(child_id) {
                        // V7: guard against division by zero (belt-and-suspenders)
                        let citation_count = (child_file.citations.len() as f64).max(1.0);
                        let weight = 1.0 / citation_count;
                        imputed_val += self.gamma * weight * child_file.price;
                    }
                }
            }

            if let Some(file) = self.tape.files.get_mut(id) {
                file.price = file.intrinsic_reward + imputed_val;
            }
        }
    }
}
