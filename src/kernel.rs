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
    pub target_omega_id: FileId,
    pub gamma: f64,
}

impl Kernel {
    pub fn new(omega: FileId) -> Self {
        Self {
            tape: Tape::default(),
            target_omega_id: omega,
            gamma: 0.99,
        }
    }

    pub fn append_tape(&mut self, mut file: File, reward: f64) -> &File {
        file.intrinsic_reward = reward;
        file.price = reward;
        let id = file.id.clone();
        
        for parent_id in &file.citations {
            self.tape.reverse_citations
                .entry(parent_id.clone())
                .or_insert_with(Vec::new)
                .push(id.clone());
        }
        
        self.tape.files.insert(id.clone(), file);
        self.tape.files.get(&id).unwrap()
    }

    pub fn hayekian_map_reduce(&mut self) {
        // Step 1: Reset market price to absolute intrinsic reward
        for (_, node) in self.tape.files.iter_mut() {
            node.price = node.intrinsic_reward; 
        }

        let mut new_prices = HashMap::new();
        
        for _ in 0..15 {
            for id in self.tape.files.keys() {
                // Pure topological gravity flow, completely oblivious to content
                let mut base_val = self.tape.files.get(id).map(|f| f.intrinsic_reward).unwrap_or(0.0);
                
                // Legacy compatibility for Hanoi target
                if id.starts_with(&self.target_omega_id) { 
                    base_val += 100_000_000_000.0; 
                }
                
                let mut imputed_val = 0.0;
                if let Some(children) = self.tape.reverse_citations.get(id) {
                    for child_id in children {
                        if let Some(child_file) = self.tape.files.get(child_id) {
                            let weight = 1.0 / (child_file.citations.len() as f64);
                            let child_price = new_prices.get(child_id).unwrap_or(&child_file.price);
                            imputed_val += self.gamma * weight * child_price;
                        }
                    }
                }
                new_prices.insert(id.clone(), base_val + imputed_val);
            }
        }
        
        for (id, price) in new_prices {
            if let Some(file) = self.tape.files.get_mut(&id) { file.price = price; }
        }
    }
}
