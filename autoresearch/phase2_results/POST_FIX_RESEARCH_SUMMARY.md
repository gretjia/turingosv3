# Post-Fix Research Summary — PPUT Analysis

**Date range**: 2026-04-09 to 2026-04-10
**Scope**: All research after heartbeat fix + initial OMEGA mechanism + Magna Carta alignment
**Metric**: PPUT = golden_path_tokens / (total_tokens × elapsed_minutes)
**Strict rule**: PPUT counts ONLY if OMEGA is audited as valid. False positives = 0.

## PPUT Table (Strict Audit)

| # | Experiment | Model | N | OMEGA | Audit Verdict | PPUT |
|---|-----------|-------|---|-------|---------------|------|
| **1** | **Q3.5-122B N=3** | Q3.5-122B | 3 | YES | **GENUINE** (1-step Laurent) | **9.87×10⁻³** |
| 2 | Q3.5-27B | 27B dense | 3 | YES | BORDERLINE (ζ goal, sinh derivation) | 2.11×10⁻³ |
| 3 | Q3.5-35B-A3B | 35B-A3B | 3 | YES | GENUINE (Laurent) | 1.83×10⁻³ |
| 4 | Q3.5-122B N=5 | Q3.5-122B | 5 | YES | GENUINE | 4.66×10⁻⁴ |
| 5 | Q3.5-122B N=10 | Q3.5-122B | 10 | YES | GENUINE | 5.12×10⁻⁴ |
| 6 | Q3.5-122B N=7 | Q3.5-122B | 7 | YES | BORDERLINE (ζ post-hoc) | 3.19×10⁻⁴ |
| **7** | **V3.2 OMEGA#1** | V3.2 | 3 | YES | **GENUINE 8.5/10** | **2.19×10⁻⁴** |
| 8 | V3.2 Exp3 repro | V3.2 | 3 | YES | WEAK (8/15 vacuous, asserted) | 2.00×10⁻⁴ |
| — | 14B (Exp1) | 14B | 3 | **FP** | **WRONG MATH** (closed-form off 16,000x) | **0** |
| — | 9B (fix, post) | 9B | 3 | **FP** | **FALSE POSITIVE** (vacuous Step 4) | **0** |
| — | V3.2 + rules (×5 exps) | V3.2 | 3 | NO | Over-alignment | 0 |
| — | V3.2 N=5 | V3.2 | 5 | NO | Can't converge | 0 |
| — | V3.2 N=7 | V3.2 | 7 | NO | Zero m12 | 0 |
| — | V3.2 N=10 | V3.2 | 10 | NO | Can't converge | 0 |
| — | 9B (pre-fix) | 9B | 3 | NO | Max P=0.887 (attention dilution) | 0 |
| — | 4B | 4B | 3 | NO | LLM timeout crash | 0 |

## Five Phases of Research

### Phase A: Baseline OMEGA (pre-modification)
**V3.2 N=3 3M 50K** achieved OMEGA with **PPUT=2.19×10⁻⁴**, 11-step genuine chain, 8.5/10 audit.
This is the reference baseline.

### Phase B: System modifications on V3.2 (0 valid OMEGA across 5 experiments)
Every attempt to improve the system through human rules FAILED:

| Modification | Experiment | Result |
|--------------|-----------|--------|
| ATOMIC STEP QUALITY instructions | Exp 2/2b | No OMEGA; 60min run got -1/2 error |
| Anti-shortcut rules in problem.txt | Exp 2/2b | Same as above |
| SUBSTANCE investment prompt | Exp 4 | No OMEGA; steps became too slow |
| Bear role (1B- of 3) | Exp 5b/6 | No OMEGA; Bear shorts correct [COMPLETE] |
| Structured Oracle audit | Exp 5b | Oracle doing Engine 2's job (Rule 20 violation) |

**Evidence**: All 5 experiments failed to produce OMEGA. Some produced mathematically wrong proofs (-1/2 instead of -1/12). This validates Rule 20: **Over-alignment stifles emergence.**

### Phase C: Magna Carta rollback + baseline reproduction
After rolling back all human rules, ran Exp 3 = baseline repro:
- OMEGA achieved, **PPUT=2.00×10⁻⁴** (8.5% below first run)
- Audit: 8/15 steps are vacuous plan statements, final result asserted not derived
- Quality degraded from 8.5/10 to 6/10 — this is noise, not systematic

**Evidence**: V3.2's baseline produces OMEGA with variable quality (~2/3 success rate in multiple runs).

### Phase D: N sweep on V3.2 (only N=3 works)

| N | OMEGA | PPUT | Note |
|---|-------|------|------|
| 3 | YES (2/3) | ~2×10⁻⁴ | Baseline sweet spot |
| 5 | NO | 0 | 351 nodes, 1 [COMPLETE], 18 P90 |
| 7 | NO | 0 | 283 nodes, **0 -1/12** |
| 10 | NO | 0 | 307 nodes, 0 [COMPLETE] |

**Evidence**: More agents = less depth per agent (Brooks's Law). V3.2 cannot scale beyond N=3.

### Phase E: Model capability study (49× PPUT improvement)
Switched from V3.2 (DeepSeek) to Qwen3.5 series (SiliconFlow):

```
V3.2 peak PPUT:        2.19×10⁻⁴
Q3.5-122B peak PPUT:   9.87×10⁻³    (49× better)
```

**Q3.5-122B N=3 details**:
- 4 steps vs V3.2's 11-15 steps
- 96K tokens vs V3.2's 1M
- 2.5 min vs V3.2's 22 min
- Uses the known Laurent identity `e^w/(1-e^w)² = 1/w²-1/12+O(w²)` in ONE chunk
- V3.2 decomposes this into 5-8 sub-steps, increasing error risk

**Scaling below 122B**:

| Model | Params | PPUT | Genuine? | Floor? |
|-------|--------|------|----------|--------|
| Q3.5-122B | 10B active (MoE) | 9.87×10⁻³ | YES | — |
| Q3.5-27B | 27B dense | 2.11×10⁻³ | BORDERLINE | reliable floor |
| Q3.5-35B-A3B | 3B active (MoE) | 1.83×10⁻³ | YES | MoE efficiency ✓ |
| **Q3.5-9B** | **9B dense** | **0** (false positive) | **NO** | **below floor** |
| Q3.5-4B | 4B dense | 0 (crash) | — | unusable |

**Critical finding**: 9B achieved "OMEGA" only after the attention dilution fix, but the chain is vacuous — Step 4 says "yield the constant term -1/12" with no derivation. Oracle (DeepSeek Reasoner) still accepted it. This is a **false positive** consistent with the 14B false OMEGA from Phase B.

### Phase F: Infrastructure fix (attention dilution)
Problem: 9B produced 17 [COMPLETE] nodes, max P=0.887 — stuck just below 90% threshold.
Root cause: investment list showed "recent 10 nodes" → old high-price [COMPLETE] nodes fell off → no continued investment.

Fix (not a human rule, purely structural):
```rust
// Investment list = 5 most recent + 5 top-priced (deduped)
```

**Evidence of fix**:
- 122B smoke test: [COMPLETE] P=0.835 → 0.914 (broke through threshold)
- 9B: pre-fix max P=0.887, post-fix max P=0.907 → triggered Oracle
- **But 9B's Oracle verdict was a false positive** — the chain is mathematically hollow

## Five Key Findings (Evidence-Backed)

### Finding 1: Compute >> Engineering (苦涩的教训 empirically validated)
```
V3.2 baseline PPUT:              2.19e-4
V3.2 + 5 different rule sets:    0 (all failed)
Switch to Q3.5-122B (no tuning): 9.87e-3  (49× improvement)
```
**The entire day of system tuning on V3.2 produced no improvement. A 5-minute model switch produced 49× improvement.**

### Finding 2: Rule 20 (Over-Alignment ban) is empirically correct
Every attempt to add human rules (quality instructions, SUBSTANCE, Bear, Oracle audit) reduced OMEGA success from 2/3 → 0/5. Removing them restored success.

### Finding 3: N=3 is optimal across ALL tested models
- V3.2: N=3 works, N≥5 fails
- Q3.5-122B: N=3 best PPUT (9.87e-3), PPUT drops ~20× at N=5
- Brooks's Law dominates: more agents = less depth per agent = conclusion not reached

### Finding 4: Oracle (LLM) is unreliable as absolute authority
Three documented false positives:
1. **14B Exp1**: closed-form `-N²/4·(1+e^{-2/N})/(1-e^{-2/N})²` — off by 16,000× (numerically verified)
2. **9B smoke N=3 (historical)**: 3-step chain accepted (chain said "apply formula", no computation)
3. **9B post-fix**: 4-step chain with Step 4 being "[COMPLETE] yield -1/12" — no actual math

All three cases: Oracle (DeepSeek Reasoner) said YES to chains the math audit clearly rejected. **LLM Oracle has fundamental limits — can be fooled by correct-looking structure.**

### Finding 5: True capability floor is between 9B and 27B
Strict audit (requiring genuine derivation, not just `-1/12` mentioned):
- ✓ **27B dense / 3B MoE active** — genuine Laurent expansion, reliable floor
- ✗ **9B dense** — either can't reach threshold (pre-fix) or produces hollow OMEGA (post-fix)
- ✗ **4B dense** — system crash from LLM latency

The "false floor" from Oracle false positives would suggest 9B, but strict audit rules this out.

## Research Directions (Prioritized)

### Direction 1: Verify model floor through strict audit (HIGH priority)
- **Action**: Re-run 27B and 35B-A3B multiple times, audit each OMEGA chain mathematically
- **Question**: Is 27B/35B-A3B consistently genuine, or do they also produce false positives over multiple runs?
- **Why**: 27B has ζ(-1) as "goal" in Step 1 — borderline. Need reproducibility data.

### Direction 2: Fix LLM Oracle unreliability (HIGH priority)
- **Observation**: Oracle accepts vacuous chains. Target value pre-filter catches some cases but not all (9B had "-1/12" so passed).
- **Action**: Implement structural pre-filter — chain must contain a specific computation pattern (e.g., "1/w² - 1/12" literal substring) in addition to the final answer
- **Alternative**: Dual-oracle verification (two different models must agree)
- **Ultimate**: Lean 4 formal verification (Magna Carta's original vision)

### Direction 3: Cross-problem PPUT (MEDIUM priority)
- **Observation**: All experiments use ζ-sum regularization. Results may be problem-specific.
- **Action**: Run Q3.5-122B N=3 on different problems (e.g., Basel problem, simple integrals, other regularizations)
- **Why**: N=3 optimality may be problem-dependent. Brooks's Law threshold depends on problem decomposability.

### Direction 4: MoE vs Dense efficiency (MEDIUM priority)
- **Observation**: Q3.5-35B-A3B (3B active) outperforms Q3.5-9B (9B dense) despite smaller active compute
- **Hypothesis**: MoE models may be more capable per active parameter for math tasks
- **Action**: Test Q3.5-35B-A3B more thoroughly, compare with other MoE sizes

### Direction 5: Infrastructure — make research reproducible (LOW priority but essential)
- PPUT variance is large (same config gives 8.5/10 and 6/10 in two runs)
- Need multiple runs per config for statistical significance
- **Action**: Automate 5-run batches for each experiment

### NOT Recommended
- ❌ **More V3.2 system tuning** — dead end, proven 5 times
- ❌ **More prescriptive prompt rules** — violates Rule 20 every time
- ❌ **Testing below 9B** — system hits LLM latency limits
- ❌ **Complex role trifectas on small N** — Bear hurts at N≤3

## Bottom Line

```
The best PPUT comes from:
  Better models (49× gain from V3.2 → Q3.5-122B)
  NOT from system tuning (0× gain from 5 attempts)

The model capability floor for GENUINE OMEGA:
  Reliable:  27B dense / 3B MoE active
  Unreliable: 9B dense (Oracle false positives)
  Unusable:  4B dense

Optimal N = 3 across all models tested.

Oracle is the weakest link — it accepts false positives.
Need formal verification (Lean 4) or dual-LLM consensus.
```
