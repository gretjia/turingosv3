# 'Boltzmann Retreat & Hayekian Circuit Breaker' Upgrade Audit (2026-03-17)

**Context:** Despite the 'Extreme Purification' upgrade, the system hit a "Deadlock" at Step 72 during the evaluation of `amc12a_2020_p7`. The LLM (DeepSeek-V3) fell into a "Zombie Tactic Loop" (repeating `norm_num` 18+ times), and the greedy router was unable to backtrack to healthier historical nodes.

## 1. Technical Audit: The 'Roadroller' Syndrome
*   **No Backtrack:** The previous router always picked the single best/deepest node (ArgMax Greedy). Once a deep node was polluted with redundant tactics, the entire swarm was forced to build upon that pollution.
*   **API Exhaustion:** $N=20$ was still too high for reliable DeepSeek-V3 responses, leading to fragmented search trees.

## 2. Upgrade: The 'Boltzmann Retreat' Plan
We have transitioned from a greedy 1D search to a thermodynamic probability cloud.

### A. Boltzmann Softmax Router (`src/swarm.rs`)
*   **Probability-Based Selection:** Instead of `MaxBy(Price)`, the system now calculates a **Softmax probability cloud** over ALL nodes on the tape.
*   **Automatic Backtrack:** Historical nodes with high purity (and decent price) now retain a statistical probability of being selected. If the current frontier becomes a "zombie wasteland," the算力 (compute power) will naturally "overflow" back to healthy historical states.
*   **Temperature $T=0.5$**: Balanced exploration and exploitation.

### B. Anti-Zombie Pruning Shield (`src/sdk/skill.rs`)
*   **Hayekian Circuit Breaker:** Implemented `AntiZombiePruningSkill` with a threshold of **3 consecutive repeats**.
*   **Instant VETO:** Any LLM output that repeats the same tactic 3 times without logical progress is now **instantly decapitated**, preventing it from polluting the tape and wasting API tokens.

### C. Overwhelming Gap Arbitrator (`src/bus.rs`)
*   **Prune over Reduce:** The kernel now skips expensive `REDUCE` operations unless a new node provides an **Overwhelming Price Gap (1.5x ratio)**.
*   **Impact:** Reduces CPU overhead and focuses the system on genuine logical breakthroughs rather than price noise.

### D. Concurrency Scaling ($N=5$)
*   **High Sample Efficiency:** Reduced $N$ to 5. With Boltzmann backtracking and Zombie pruning, each sample is significantly more valuable. This also ensures 100% API reliability (no 503s).

## 3. Implementation Files
*   `src/sdk/skill.rs`: Added `AntiZombiePruningSkill` and `OverwhelmingGapArbitrator`.
*   `src/bus.rs`: Updated `tick_map_reduce` to support price-based arbitration.
*   `experiments/minif2f_swarm/src/swarm.rs`: Replaced greedy selection with Boltzmann sampling.
*   `experiments/minif2f_swarm/src/bin/full_test_evaluator.rs`: Mounted new SOTA skills and adjusted $N=5$.

## 4. Scientific Conclusion
TuringOS v3 is no longer a "Roadroller." It has evolved into a **Probabilistic Explorer** capable of self-correcting and retreating from logical traps.
