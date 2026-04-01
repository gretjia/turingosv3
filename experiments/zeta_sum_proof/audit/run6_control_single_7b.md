# Zeta ζ-Sum — Control Group: Single Qwen2.5-7B Direct (No TuringOS)

**Model**: Pro/Qwen2.5-7B-Instruct | **Method**: Single-shot, one prompt, no swarm
**Tokens**: 182 in / 1343 out | **Result**: Claims [COMPLETE]

## Verdict: MATHEMATICALLY WRONG

The 7B model claims to prove 1+2+3+...= -1/12 but the proof is **fatally flawed**:

### Fatal Errors

1. **Step 4: DCT misapplication** — Claims `lim S(N) = Σ lim M(m,N) = Σ m`, which diverges. The model correctly notes this diverges, then hand-waves past it. The DCT **cannot be applied here** because the pointwise limit `M(m,N) → m` is not integrable. The DCT requires a dominating function independent of N, but `m·exp(-m/N)` is dominated by `m` which is NOT integrable on [0,∞). This is exactly the kind of error that makes regularization non-trivial.

2. **Step 5: Wrong integral** — Claims `Σ m·exp(-m/N) ≈ N`. The correct answer is `N²` (geometric series differentiation gives `exp(-1/N)/(1-exp(-1/N))² ≈ N²`). The substitution `u = x/N` gives `N² ∫ u·exp(-u) du = N²`, not `N`.

3. **Step 6: "average value of cos is zero"** — This is wrong. `cos(m/N)` for the relevant range `m ~ O(N)` does NOT average to zero. The whole point of the hint formula is that the cos term creates oscillation that cancels the divergence in a specific way.

4. **Step 7: "By carefully analyzing..."** — Pure hand-wave. Jumps from incorrect intermediate steps to `-1/12` with no computation. The actual proof requires using `z = exp(-(1-i)/N)` and computing `Re[z/(1-z)²]` via Laurent expansion.

### Comparison: Single 7B vs 90-Agent Swarm

| | Single 7B | 90-Agent Swarm (Run 6) |
|--|----------|----------------------|
| Claims proof? | YES | YES |
| Mathematically correct? | **NO** | **NO** (same hand-wave quality) |
| Steps | 7 | 18 (Golden Path) |
| Cost | 1,525 tokens | ~1M+ tokens |
| Time | 3 seconds | 50 minutes |
| Novel insight? | None | None |

### Honest Assessment

Both the single-shot 7B and the 90-agent swarm produce the **same quality of proof** — superficially structured but mathematically wrong at the critical steps. The swarm's 18-step Golden Path is just the same hand-wave spread across more nodes, not a deeper proof.

The swarm's value is NOT in the proof quality (which is bounded by model capability), but in:
1. **Market dynamics**: genuine bull/bear warfare, price discovery
2. **Architecture validation**: 90 concurrent agents, 1748 tx, zero corruption
3. **Scaling laws**: tx/agent budget relationship empirically established

**For actual mathematical correctness, model capability is the bottleneck, not architecture.**
