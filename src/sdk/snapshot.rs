/// Immutable Universe Snapshot
///
/// A frozen view of the entire TuringOS state at a point in time.
/// Past spacetime is absolute — agents read snapshots, never mutate history.
/// This enables lock-free concurrent reads (Append-Only DAG guarantee).

use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use crate::kernel::Tape;

/// Frozen view of a single binary prediction market
#[derive(Clone, Default, Serialize, Deserialize)]
pub struct MarketSnapshot {
    /// Bayesian probability that this node is on the Golden Path
    pub yes_price: f64,
    /// 1 - yes_price
    pub no_price: f64,
    /// Pool reserves (for advanced agents)
    pub yes_reserve: f64,
    pub no_reserve: f64,
    /// Whether market has been resolved
    pub resolved: Option<bool>,
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct UniverseSnapshot {
    /// The complete DAG of all appended nodes
    pub tape: Tape,
    /// All agent balances at snapshot time
    pub balances: HashMap<String, f64>,
    /// Turing-Polymarket: agent YES/NO/LP holdings per node
    pub portfolios: HashMap<String, HashMap<String, (f64, f64, f64)>>,
    /// Turing-Polymarket: binary market states per node
    pub markets: HashMap<String, MarketSnapshot>,
    /// Top-N market price leaderboard (formatted string)
    pub market_ticker: String,
    /// Graveyard tombstones per node (failure records)
    pub tombstones: HashMap<String, String>,
    /// Generation counter — increments on rebirth
    pub generation: u32,
}
