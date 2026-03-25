/// Immutable Universe Snapshot
///
/// A frozen view of the entire TuringOS state at a point in time.
/// Past spacetime is absolute — agents read snapshots, never mutate history.
/// This enables lock-free concurrent reads (Append-Only DAG guarantee).

use std::collections::HashMap;
use crate::kernel::Tape;

#[derive(Clone, Default)]
pub struct UniverseSnapshot {
    /// The complete DAG of all appended nodes
    pub tape: Tape,
    /// All agent balances at snapshot time
    pub balances: HashMap<String, f64>,
    /// Top-N market price leaderboard (formatted string)
    pub market_ticker: String,
    /// Graveyard tombstones per node (failure records)
    pub tombstones: HashMap<String, String>,
    /// Generation counter — increments on rebirth. Agents use this to detect
    /// world resets and purge stale private context (phantom context prevention).
    pub generation: u32,
}
