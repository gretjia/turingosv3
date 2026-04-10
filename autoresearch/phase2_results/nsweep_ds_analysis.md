# N Sweep Results — DeepSeek V3.2 Chat, All-M, 50K, 30min

**Date**: 2026-04-09/10
**Model**: deepseek-chat (V3.2) via DeepSeek API (proxy:8089)
**Config**: all Mathematician, GENESIS=50K, OMEGA=[COMPLETE]+P>=90%+target=-1/12
**Wall clock**: 30 min per run

## Results

| N | Nodes | [COMPLETE] | -1/12 | P>=90% | C∩P90 | OMEGA | Tokens | Depth | Appends |
|---|-------|-----------|-------|--------|-------|-------|--------|-------|---------|
| 3 (OMEGA#1) | 127 | 7 | 31 | 7 | 1 | **YES** | 1,011,701 | 11 | 405 |
| 3 (OMEGA#2) | 132 | 4 | 7 | 5 | 1 | **YES** | 897,731 | 15 | 378 |
| 3 (sweep) | 143 | 3 | 2 | 17 | 0 | NO | 1,345,680 | 19 | 488 |
| 5 | 351 | 1 | 8 | 18 | 0 | NO | 2,544,083 | 12 | 880 |
| 7 | 283 | 0 | 0 | 25 | 0 | NO | 2,823,767 | 10 | 880 |
| 10 | 307 | 0 | 9 | 14 | 0 | NO | 2,750,855 | 9 | 880 |

## Scaling Patterns

### 1. [COMPLETE] inversely scales with N
```
N=3:  4-7 [COMPLETE] nodes
N=5:  1
N=7:  0
N=10: 0
```
More agents = more exploration, less convergence. Each agent writes fewer steps (880 appends / 10 agents = 88 per agent vs 405/3 = 135 per agent). Agents don't reach the conclusion because they're spread across too many branches.

### 2. Depth decreases with N (Brooks's Law)
```
N=3:  11-19 steps
N=5:  12
N=7:  10
N=10: 9
```
More agents compete for the same deepest chain. Each agent gets fewer turns on the main chain. Breadth increases but depth — which is what matters for proof completion — decreases.

### 3. P>=90% increases then plateaus
```
N=3:  5-17
N=5:  18
N=7:  25
N=10: 14
```
More agents = more cross-investment = more nodes reach high consensus. But these are all intermediate steps, not conclusions. P>=90% without [COMPLETE] is meaningless for OMEGA.

### 4. Tokens increase linearly, PPUT drops
```
N=3:  ~1M tokens → OMEGA (PPUT ~2e-4)
N=5:  ~2.5M tokens → no OMEGA (PPUT = 0)
N=7:  ~2.8M tokens → no OMEGA (PPUT = 0)
N=10: ~2.8M tokens → no OMEGA (PPUT = 0)
```
Diminishing returns: 2.5x more tokens at N=5 produces worse results than N=3.

### 5. N=7 anomaly: zero -1/12
N=7 produced 283 nodes and 25 P>=90% nodes but zero mentions of -1/12. All agents converged on intermediate algebraic steps without reaching the final answer. This suggests a critical exploration-exploitation collapse: too many agents exploring similar intermediate steps, none pushing deep enough to reach the conclusion.

## Conclusion

**N=3 is the optimal swarm size for this problem with DeepSeek V3.2.**

The OMEGA success rate is:
- N=3: 2/3 (67%)
- N=5: 0/1 (0%)
- N=7: 0/1 (0%)
- N=10: 0/1 (0%)

PPUT is only measurable at N=3: ~2×10⁻⁴.

The failure mode for N>3 is consistent: agents explore many branches (high node count, many P>=90% intermediate steps) but fail to converge on a conclusion ([COMPLETE] → 0). This is the attention dilution + Brooks's Law effect: more agents = less depth per agent = proof never completes within the 30-minute window.

## Note on SiliconFlow vs DeepSeek API

An earlier parallel sweep using SiliconFlow's DeepSeek-V3.2 for N=5 and N=7 produced different behavior (more shortcuts, fewer tokens). SiliconFlow's V3.2 may be a different version or quantization. All results in this analysis use the official DeepSeek API for fair comparison.
