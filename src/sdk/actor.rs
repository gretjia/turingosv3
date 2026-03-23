/// TuringOS Actor Runtime — Lock-free Concurrent Agent Model
///
/// Replaces batch-synchronous swarm (block_on + collect-all) with
/// continuous asynchronous agent loops communicating via channels.
///
/// Architecture:
///   watch::channel — universe snapshot broadcast (lock-free reads)
///   mpsc::channel — mempool (agent → reactor submissions)
///   Each agent runs in its own tokio task, reading snapshots and
///   submitting transactions independently. Slow and fast models
///   coexist without blocking each other.

use crate::sdk::snapshot::UniverseSnapshot;
use rand::distributions::{WeightedIndex, Distribution};

/// A transaction submitted by an agent to the reactor
#[derive(Debug, Clone)]
pub struct MinerTx {
    /// Which agent submitted this
    pub agent_id: String,
    /// Which model produced this
    pub model_name: String,
    /// The raw payload (tactic + wallet tag, or search query, etc.)
    pub payload: String,
    /// Which node this builds on (None = builds on root/problem statement)
    pub parent_id: Option<String>,
    /// Action type: "invest", "search", "observe", "view_node"
    pub action_type: String,
}

/// Boltzmann softmax selection over frontier nodes.
/// Temperature T controls exploration: T→0 = greedy, T→∞ = uniform random.
/// Returns the selected node's ID, or None if no frontier exists.
pub fn boltzmann_select_parent(snapshot: &UniverseSnapshot, temperature: f64) -> Option<String> {
    let reverse_citations = &snapshot.tape.reverse_citations;
    let frontier: Vec<_> = snapshot.tape.files.values()
        .filter(|f| f.stake > 0)
        .filter(|f| reverse_citations.get(&f.id).map_or(true, |c| c.is_empty()))
        .collect();

    if frontier.is_empty() { return None; }
    if frontier.len() == 1 { return Some(frontier[0].id.clone()); }

    // Max trick: subtract max price to prevent exp() overflow
    let max_price = frontier.iter().map(|n| n.price).fold(f64::NEG_INFINITY, f64::max);
    let t = temperature.max(0.01); // floor to prevent division by zero

    let weights: Vec<f64> = frontier.iter()
        .map(|n| ((n.price - max_price) / t).exp().max(1e-9))
        .collect();

    let dist = WeightedIndex::new(&weights).ok()?;
    let mut rng = rand::thread_rng();
    let selected = &frontier[dist.sample(&mut rng)];

    log::info!(">>> [BOLTZMANN T={:.2}] Selected {} (P:{:.0}) from {} frontier nodes",
        t, selected.id, selected.price, frontier.len());
    Some(selected.id.clone())
}

/// Build a formatted proof chain from a Boltzmann-selected frontier node's ancestor path.
/// Returns (chain_string, selected_parent_id).
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

    // Boltzmann softmax selection replaces greedy best_node
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
    chain_str.push_str("=== WRITE THE NEXT STEP ===\n");

    // Order Book: 3 competing chains
    let reverse_citations = &snapshot.tape.reverse_citations;
    let mut frontier: Vec<_> = snapshot.tape.files.values()
        .filter(|f| f.stake > 0)
        .filter(|f| reverse_citations.get(&f.id).map_or(true, |c| c.is_empty()))
        .collect();

    if frontier.len() > 1 {
        chain_str.push_str("\n=== ORDER BOOK ===\n");
        // A: highest price
        frontier.sort_by(|a, b| b.price.partial_cmp(&a.price).unwrap_or(std::cmp::Ordering::Equal));
        if let Some(n) = frontier.first() {
            chain_str.push_str(&format!("A(consensus): {} P:{:.0}\n", n.id, n.price));
        }
        // B: different from A, exists
        if let Some(n) = frontier.get(1) {
            chain_str.push_str(&format!("B(alt):       {} P:{:.0}\n", n.id, n.price));
        }
        // C: most recent
        if let Some(last_id) = snapshot.tape.time_arrow.last() {
            if let Some(n) = snapshot.tape.files.get(last_id) {
                if frontier.first().map_or(true, |top| top.id != n.id) {
                    chain_str.push_str(&format!("C(recent):    {} P:{:.0}\n", n.id, n.price));
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
