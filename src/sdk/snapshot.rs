/// Immutable Universe Snapshot
///
/// A frozen view of the entire TuringOS state at a point in time.
/// Past spacetime is absolute — agents read snapshots, never mutate history.
/// This enables lock-free concurrent reads (Append-Only DAG guarantee).

use std::collections::HashMap;
use crate::kernel::Tape;

/// Frozen view of a single AMM pool
#[derive(Clone, Default)]
pub struct PoolSnapshot {
    pub coin_reserve: f64,
    pub token_reserve: f64,
    pub spot_price: f64,
    /// Cost in coins to buy 100 citation tokens (f64::INFINITY if impossible)
    pub citation_cost_100: f64,
}

#[derive(Clone, Default)]
pub struct UniverseSnapshot {
    /// The complete DAG of all appended nodes
    pub tape: Tape,
    /// All agent balances at snapshot time
    pub balances: HashMap<String, f64>,
    /// TuringSwap: agent token portfolios (agent -> node -> tokens)
    pub portfolios: HashMap<String, HashMap<String, f64>>,
    /// TuringSwap: AMM pool states per node
    pub pool_states: HashMap<String, PoolSnapshot>,
    /// Top-N market price leaderboard (formatted string)
    pub market_ticker: String,
    /// Graveyard tombstones per node (failure records)
    pub tombstones: HashMap<String, String>,
    /// Generation counter — increments on rebirth. Agents use this to detect
    /// world resets and purge stale private context (phantom context prevention).
    pub generation: u32,
    /// Remaining bounty escrow (finite genesis budget)
    pub bounty_remaining: f64,
}
