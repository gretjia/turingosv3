/// TuringOS Actor Runtime — Lock-free Concurrent Agent Model
///
/// Architecture:
///   watch::channel — universe snapshot broadcast (lock-free reads)
///   mpsc::channel — mempool (agent → reactor submissions)
///   Each agent runs in its own tokio task, reading snapshots and
///   submitting transactions independently.

use crate::sdk::snapshot::UniverseSnapshot;
use crate::kernel::{File, Tape};
use rand::distributions::{WeightedIndex, Distribution};

/// A transaction submitted by an agent to the reactor
#[derive(Debug, Clone)]
pub struct MinerTx {
    pub agent_id: String,
    pub model_name: String,
    pub payload: String,
    pub parent_id: Option<String>,
    pub action_type: String,
}

/// Tunable parameters for Boltzmann selection.
/// All configurable via env vars for AutoResearch sweep.
/// See: experiments/zeta_sum_proof/PARAMS.md
pub struct BoltzmannParams {
    /// Max frontier nodes before lowest-scored are pruned.
    /// Source: DeepSeek econ audit 2026-04-02 §6.9 — "Frontier Size Cap"
    /// 190 nodes = over-dilution. 30 ≈ 2×swarm_size focuses compute.
    /// Env: FRONTIER_CAP (default 30, 0 = unlimited)
    pub frontier_cap: usize,

    /// Exponent for depth weighting: score × log(depth+1)^depth_weight
    /// Source: DeepSeek econ audit 2026-04-02 §6.8 — "Adjust Boltzmann Selection"
    /// 0 = no depth bias. 1.0 = log. Higher = stronger depth preference.
    /// Env: DEPTH_WEIGHT (default 1.0)
    pub depth_weight: f64,

    /// Price Gate alpha: child must exceed parent × (1 + alpha/depth) to mask parent.
    /// Source: DeepSeek econ audit 2026-04-02 §6.2 — "Depth-Boosted Frontier Rule"
    /// Higher alpha = parents stickier at shallow depths, easier to extend at deep depths.
    /// Env: PRICE_GATE_ALPHA (default 0.05)
    pub price_gate_alpha: f64,
}

impl Default for BoltzmannParams {
    fn default() -> Self {
        Self {
            frontier_cap: 30,
            depth_weight: 1.0,
            price_gate_alpha: 0.05,
        }
    }
}

impl BoltzmannParams {
    /// Load from environment variables, falling back to defaults.
    pub fn from_env() -> Self {
        Self {
            frontier_cap: std::env::var("FRONTIER_CAP").ok()
                .and_then(|s| s.parse().ok()).unwrap_or(30),
            depth_weight: std::env::var("DEPTH_WEIGHT").ok()
                .and_then(|s| s.parse().ok()).unwrap_or(1.0),
            price_gate_alpha: std::env::var("PRICE_GATE_ALPHA").ok()
                .and_then(|s| s.parse().ok()).unwrap_or(0.05),
        }
    }
}

/// Frontier Price Gate (depth-boosted):
///   Parent is in frontier iff (a) no children, OR
///   (b) ALL children have price <= parent.price × (1 + alpha/depth)
///
/// Architect directive 2026-04-02:
///   "仅当子节点价格高于父节点的时候，父节点才可以被mask，否则父节点也参与随机竞选"
/// DeepSeek econ audit 2026-04-02 §6.2:
///   "child replaces parent if child_price > parent_price × (1 + α/depth)"
///   Higher depth → lower hurdle → easier to extend deep chains.
fn is_frontier(node: &File, tape: &Tape, alpha: f64) -> bool {
    match tape.reverse_citations.get(&node.id) {
        None => true,
        Some(children) if children.is_empty() => true,
        Some(children) => {
            let depth = chain_depth(&node.id, tape).max(1) as f64;
            // Depth-boosted threshold: shallow parents are stickier
            // Source: PARAMS.md — PRICE_GATE_ALPHA
            let threshold = node.price * (1.0 + alpha / depth);
            !children.iter().any(|cid| {
                tape.files.get(cid).map_or(false, |child| child.price > threshold)
            })
        }
    }
}

/// Lineage score with depth weighting.
///
/// Base: exponentially-decayed weighted average of ancestor prices.
///   score = Σ(w_i × ancestor_i.price) / Σ(w_i), w_i = 0.5^i
///
/// Depth boost: final_score = base_score × log(depth+1)^depth_weight
///   Source: DeepSeek econ audit 2026-04-02 §6.8
///   depth=1 → ×0.69, depth=5 → ×1.79, depth=10 → ×2.40 (at weight=1.0)
///   This gives deep chains ~3.5× more compute than shallow ones.
fn lineage_score(node: &File, tape: &Tape, depth_weight: f64) -> f64 {
    let mut total_weight = 0.0_f64;
    let mut total_score = 0.0_f64;
    let mut decay = 1.0_f64;
    // Self
    total_score += decay * node.price;
    total_weight += decay;

    // Walk ancestors (up to 3 levels)
    let mut cur_id = node.citations.first().cloned();
    for _ in 0..3 {
        decay *= 0.5;
        match cur_id {
            Some(ref pid) => {
                if let Some(ancestor) = tape.files.get(pid) {
                    total_score += decay * ancestor.price;
                    total_weight += decay;
                    cur_id = ancestor.citations.first().cloned();
                } else {
                    break;
                }
            }
            None => {
                total_score += decay * 0.5; // root: neutral prior
                total_weight += decay;
                break;
            }
        }
    }

    let base_score = total_score / total_weight;

    // Full chain depth for depth weighting
    let full_depth = chain_depth(&node.id, tape);
    // Source: PARAMS.md — DEPTH_WEIGHT
    let depth_factor = ((full_depth as f64) + 1.0).ln().max(0.1).powf(depth_weight);

    base_score * depth_factor
}

/// Trace chain depth from a node back to root.
fn chain_depth(node_id: &str, tape: &Tape) -> usize {
    let mut depth = 0;
    let mut cur = node_id.to_string();
    while let Some(node) = tape.files.get(&cur) {
        if node.citations.is_empty() { break; }
        cur = node.citations[0].clone();
        depth += 1;
    }
    depth
}

/// Collect frontier nodes, apply Price Gate + cap.
fn collect_frontier<'a>(tape: &'a Tape, params: &BoltzmannParams) -> Vec<&'a File> {
    let mut frontier: Vec<&File> = tape.files.values()
        .filter(|f| f.stake > 0)
        .filter(|f| is_frontier(f, tape, params.price_gate_alpha))
        .collect();

    // Frontier cap: keep top N by lineage score, prune rest.
    // Source: DeepSeek econ audit 2026-04-02 §6.9 — prevents 190-node dilution.
    // Env: FRONTIER_CAP (default 30, 0 = unlimited)
    if params.frontier_cap > 0 && frontier.len() > params.frontier_cap {
        frontier.sort_by(|a, b| {
            let sa = lineage_score(a, tape, params.depth_weight);
            let sb = lineage_score(b, tape, params.depth_weight);
            sb.partial_cmp(&sa).unwrap_or(std::cmp::Ordering::Equal)
        });
        frontier.truncate(params.frontier_cap);
    }

    frontier
}

/// Boltzmann softmax selection over frontier nodes, weighted by lineage score.
/// Uses BoltzmannParams for all tunable knobs.
pub fn boltzmann_select_parent(snapshot: &UniverseSnapshot, temperature: f64) -> Option<String> {
    boltzmann_select_with_params(snapshot, temperature, &BoltzmannParams::from_env())
}

pub fn boltzmann_select_with_params(
    snapshot: &UniverseSnapshot,
    temperature: f64,
    params: &BoltzmannParams,
) -> Option<String> {
    let frontier = collect_frontier(&snapshot.tape, params);

    if frontier.is_empty() { return None; }
    if frontier.len() == 1 { return Some(frontier[0].id.clone()); }

    let scores: Vec<f64> = frontier.iter()
        .map(|n| lineage_score(n, &snapshot.tape, params.depth_weight))
        .collect();

    let max_score = scores.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let t = temperature.max(0.01);

    let weights: Vec<f64> = scores.iter()
        .map(|s| ((s - max_score) / t).exp().max(1e-9))
        .collect();

    let dist = WeightedIndex::new(&weights).ok()?;
    let mut rng = rand::thread_rng();
    let idx = dist.sample(&mut rng);
    let selected = &frontier[idx];

    log::info!(">>> [BOLTZMANN T={:.2}] Selected {} (L:{:.2} P:{:.2}) from {} frontier nodes",
        t, selected.id, scores[idx], selected.price, frontier.len());
    Some(selected.id.clone())
}

/// Build a formatted proof chain from a Boltzmann-selected frontier node.
pub fn build_chain_from_snapshot(
    snapshot: &UniverseSnapshot,
    problem: &str,
) -> (String, Option<String>) {
    build_chain_from_snapshot_with_temperature(snapshot, problem, 0.5)
}

/// Build chain with explicit Boltzmann temperature parameter.
pub fn build_chain_from_snapshot_with_temperature(
    snapshot: &UniverseSnapshot,
    problem: &str,
    temperature: f64,
) -> (String, Option<String>) {
    if snapshot.tape.files.is_empty() {
        return (problem.to_string(), None);
    }

    let selected_id = match boltzmann_select_parent(snapshot, temperature) {
        Some(id) => id,
        None => return (problem.to_string(), None),
    };

    let selected_node = match snapshot.tape.files.get(&selected_id) {
        Some(n) => n,
        None => return (problem.to_string(), None),
    };

    // Trace ancestor chain back to root
    let mut chain = Vec::new();
    let mut cur_id = selected_node.id.clone();
    while let Some(node) = snapshot.tape.files.get(&cur_id) {
        let step = node.payload.lines().last().unwrap_or(&node.payload).trim().to_string();
        chain.push((cur_id.clone(), step, node.price));
        if node.citations.is_empty() { break; }
        cur_id = node.citations[0].clone();
    }
    chain.reverse();

    let parent_id = selected_node.id.clone();

    let mut chain_str = format!("{}\n\n=== CURRENT BEST PROOF CHAIN ===\n", problem);
    for (i, (_id, step, price)) in chain.iter().enumerate() {
        chain_str.push_str(&format!("Step {} [Price: {:.0}]: {}\n", i + 1, price, step));
    }
    // Sibling visibility: show children of selected parent so agent can invest instead of duplicate
    if let Some(siblings) = snapshot.tape.reverse_citations.get(&selected_node.id) {
        if !siblings.is_empty() {
            chain_str.push_str(&format!("\n=== SIBLING NODES (same parent: {}) ===\n", selected_node.id));
            for sib_id in siblings.iter().take(5) {
                if let Some(sib) = snapshot.tape.files.get(sib_id) {
                    let preview: String = sib.payload.chars().take(80).collect();
                    let preview = preview.replace('\n', " ");
                    chain_str.push_str(&format!("  [{}] P={:.0}% — \"{}\"\n", sib_id, sib.price, preview));
                }
            }
            chain_str.push_str("You may INVEST YES/NO on a sibling instead of appending a duplicate step.\n");
        }
    }

    chain_str.push_str("=== WRITE THE NEXT STEP (or invest on a sibling) ===\n");

    // Order Book: top 3 frontier nodes by lineage score
    let params = BoltzmannParams::from_env();
    let mut frontier = collect_frontier(&snapshot.tape, &params);

    if frontier.len() > 1 {
        chain_str.push_str("\n=== ORDER BOOK ===\n");
        let tape_ref = &snapshot.tape;
        frontier.sort_by(|a, b| {
            let sa = lineage_score(a, tape_ref, params.depth_weight);
            let sb = lineage_score(b, tape_ref, params.depth_weight);
            sb.partial_cmp(&sa).unwrap_or(std::cmp::Ordering::Equal)
        });
        if let Some(n) = frontier.first() {
            let ls = lineage_score(n, tape_ref, params.depth_weight);
            chain_str.push_str(&format!("A(consensus): {} L:{:.2} P:{:.2}\n", n.id, ls, n.price));
        }
        if let Some(n) = frontier.get(1) {
            let ls = lineage_score(n, tape_ref, params.depth_weight);
            chain_str.push_str(&format!("B(alt):       {} L:{:.2} P:{:.2}\n", n.id, ls, n.price));
        }
        if let Some(last_id) = snapshot.tape.time_arrow.last() {
            if let Some(n) = snapshot.tape.files.get(last_id) {
                if frontier.first().map_or(true, |top| top.id != n.id) {
                    chain_str.push_str(&format!("C(recent):    {} P:{:.2}\n", n.id, n.price));
                }
            }
        }
    }

    (chain_str, Some(parent_id))
}

/// Look up a specific node's content (for ViewNode free tool)
pub fn view_node(snapshot: &UniverseSnapshot, node_id: &str) -> String {
    if let Some(node) = snapshot.tape.files.get(node_id) {
        format!("[VIEW {}] Price:{:.0} | {}", node_id, node.price, node.payload)
    } else {
        format!("[VIEW {}] Node not found.", node_id)
    }
}
