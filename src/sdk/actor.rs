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

/// Build a formatted proof chain from the highest-priced node's ancestor path.
/// Returns (chain_string, selected_parent_id).
pub fn build_chain_from_snapshot(
    snapshot: &UniverseSnapshot,
    problem: &str,
) -> (String, Option<String>) {
    if snapshot.tape.files.is_empty() {
        return (problem.to_string(), None);
    }

    // Find the highest-priced node
    let best_node = snapshot.tape.files.values()
        .filter(|f| f.stake > 0)
        .max_by(|a, b| a.price.partial_cmp(&b.price).unwrap_or(std::cmp::Ordering::Equal));

    let best_node = match best_node {
        Some(n) => n,
        None => return (problem.to_string(), None),
    };

    // Trace ancestor chain back to root
    let mut chain = Vec::new();
    let mut cur_id = best_node.id.clone();
    while let Some(node) = snapshot.tape.files.get(&cur_id) {
        let step = node.payload.lines().last().unwrap_or(&node.payload).trim().to_string();
        chain.push((cur_id.clone(), step, node.price));
        if node.citations.is_empty() { break; }
        cur_id = node.citations[0].clone();
    }
    chain.reverse();

    let parent_id = best_node.id.clone();

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
