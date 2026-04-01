# Zeta Sum Proof — Non-Golden Path DAG Analysis

**Run**: Run 2 (112 tx, 61 nodes, 1 gen, OMEGA reached)
**Golden Path**: `tx_7_by_10` → `tx_24_by_14` → `tx_70_by_10` → `tx_112_by_9` (4 steps)
**Non-Golden Nodes**: 57 / 61 (93% redundancy)

---

## DAG Tree (Human-Readable)

```
ROOT (Problem: prove 1+2+3+... = -1/12 via exponential regularization)
│
├─── STEP 1: Define S(N) + Convergence Proof ─────────────────────────
│    │
│    ├── tx_1_by_4  [Agent_4]  P:0  "absolute convergence"
│    ├── tx_2_by_2  [Agent_2]  P:0  "decays exponentially"
│    │   └── tx_27_by_2  [Agent_2]  P:0  Step 2 (Euler + dual sum)
│    │       └── tx_48_by_2  [Agent_2]  P:0  "complex exponential closed form"
│    │           └── tx_93_by_11 [Agent_11] P:0  Step 3 (z/(1-z)^2)
│    │
│    ├── tx_3_by_0  [Agent_0]  P:0  "regulated sum"
│    │   └── tx_22_by_0  [Agent_0]  P:0  Step 2 (Re path)
│    │
│    ├── tx_4_by_12 [Agent_12] P:0  "dominating linear growth"
│    │   ├── tx_21_by_12 [Agent_12] P:0  Step 2 (Re path, r = exp((i-1)/N))
│    │   │   └── tx_74_by_6  [Agent_6]  P:0  Step 3 (r/(1-r)^2)
│    │   ├── tx_25_by_8  [Agent_8]  P:0  Step 2 (Re path)
│    │   │   ├── tx_39_by_12 [Agent_12] P:0  Step 3 (z/(1-z)^2)
│    │   │   │   └── tx_105_by_8 [Agent_8] P:0  Step 4 (Taylor expansion)
│    │   │   └── tx_46_by_6  [Agent_6]  P:0  Step 3 (z/(1-z)^2)
│    │   │       ├── tx_73_by_11 [Agent_11] P:0  Step 4 (Taylor of e^{-(1-i)ε})
│    │   │       └── tx_91_by_14 [Agent_14] P:0  Step 4 (= N²/(1-i)² - 1/12) ★
│    │   └── tx_26_by_10 [Agent_10] P:0  "rewrite using Euler's formula"
│    │       └── tx_42_by_14 [Agent_14] P:0  Step 3 (dual sum explicit)
│    │           └── tx_65_by_2  [Agent_2]  P:0  Step 4 (arithmetico-geometric)
│    │               └── tx_109_by_2 [Agent_2] P:0  Step 5 (Laurent f(z)=1/z²-1/12)
│    │
│    ├── tx_5_by_8  [Agent_8]  P:0  minimal definition
│    │   └── tx_56_by_14 [Agent_14] P:0  Step 2 (Re path)
│    │       ├── tx_85_by_7  [Agent_7]  P:0  Step 3 (z/(1-z)^2)
│    │       ├── tx_78_by_8  [Agent_8]  P:0  "apply formula" (too brief)
│    │       └── tx_102_by_14 [Agent_14] P:0  Step 3 (r/(1-r)^2)
│    │
│    ├── tx_6_by_14 [Agent_14] P:0  Step 1+2 combined (Euler + dual sum)
│    │
│    ╠══ tx_7_by_10 [Agent_10] P:1  ★ GOLDEN PATH STEP 1 ★ (ratio test)
│    │   │
│    │   ╠══ tx_24_by_14 [Agent_14] P:1  ★ GOLDEN PATH STEP 2 ★
│    │   │   │   (Euler dual sum + z/(1-z)^2 + convergence proof)
│    │   │   │
│    │   │   ├── tx_57_by_8  [Agent_8]  P:0  ★ KEY INSIGHT: z2 = conj(z1) ★
│    │   │   │       (proves dual-sum = Re of single term)
│    │   │   │
│    │   │   ├── tx_84_by_12 [Agent_12] P:0  Real closed-form (r,θ parametrize)
│    │   │   │
│    │   │   ╠══ tx_70_by_10 [Agent_10] P:1  ★ GOLDEN PATH STEP 3 ★
│    │   │   │   │   (Laurent: 1/((i-1)²ε²) - 1/12 + O(ε²))
│    │   │   │   │
│    │   │   │   ╠══ tx_112_by_9 [Agent_9] P:1  ★ GOLDEN PATH STEP 4 / OMEGA ★
│    │   │   │   │       (Re(i/2)=0, cancel divergence, limit = -1/12)
│    │   │   │   │
│    │   │   │   └── (no other children)
│    │   │   │
│    │   │   └── (other children absorbed into GP chain)
│    │   │
│    │   └── (no non-GP children directly from tx_7)
│    │
│    ├── tx_8_by_6  [Agent_6]  P:0  "ratio test"
│    │   ├── tx_23_by_6  [Agent_6]  P:0  Step 2 (Euler dual sum)
│    │   │   ├── tx_83_by_10 [Agent_10] P:0  Step 3 (arithmetico-geometric)
│    │   │   └── tx_87_by_1  [Agent_1]  P:0  Step 3 (r/(1-r)^2)
│    │   └── tx_34_by_4  [Agent_4]  P:0  Step 2 (Re path, direct closed form)
│    │
│    ├── tx_11_by_11 [Agent_11] P:0  standard statement
│    │   └── tx_38_by_8  [Agent_8]  P:0  Step 2 (Re path)
│    │       └── tx_67_by_8  [Agent_8]  P:0  Step 3 (z/(1-z)^2)
│    │
│    ├── tx_17_by_13 [Agent_13] P:0  "cos = Re(exp)" (Re path seed)
│    │   └── tx_36_by_0  [Agent_0]  P:0  Step 2 (apply Σmx^m identity)
│    │       └── tx_60_by_9  [Agent_9]  P:0  Step 3 (rewrite via 1-exp(-z))
│    │
│    ├── tx_18_by_9  [Agent_9]  P:0  ratio test detailed
│    │
│    ├── tx_29_by_3  [Agent_3]  P:0  ratio test
│    │   └── tx_51_by_0  [Agent_0]  P:0  "rewrite using complex exponential"
│    │       └── tx_86_by_6  [Agent_6]  P:0  Step 3 (Euler substitute)
│    │           └── tx_101_by_10 [Agent_10] P:0  Step 4 (Σmz^m formula)
│    │
│    ├── tx_41_by_5  [Agent_5]  P:0  ratio test (late arrival)
│    │   └── tx_66_by_6  [Agent_6]  P:0  Step 2 (Re path + identity)
│    │
│    ├── tx_47_by_1  [Agent_1]  P:0  m=1 start (not m=0)
│    │   └── tx_76_by_4  [Agent_4]  P:0  Step 2 (Re path + closed form)
│    │       └── tx_99_by_6  [Agent_6]  P:0  Step 3 (substitute z in terms of ω)
│    │
│    └── tx_49_by_7  [Agent_7]  P:0  Re(exp) direct
│        ├── tx_100_by_4  [Agent_4]  P:0  Step 2 (closed form)
│        └── tx_104_by_3  [Agent_3]  P:0  Step 2 (closed form)
│
├─── STEP 4 alternatives (non-GP, reached independently) ─────────────
│    ├── tx_92_by_13 [Agent_13] P:0  Laurent: a/(1-a)² = N²/(i-1)² - 1/12 ★
│    ├── tx_103_by_0 [Agent_0]  P:0  Laurent: 1/w² - 1/12 + O(w²) ★
│    └── tx_109_by_2 [Agent_2]  P:0  Laurent: f(z) = 1/z² - 1/12 + z/12 ★
│
└─── ORPHAN / LATE NODES ─────────────────────────────────────────────
     ├── tx_12_by_4  [Agent_4]  P:0  Step 2 (cites tx_1_by_4)
     │   └── tx_61_by_3  [Agent_3]  P:0  Step 3 (dual sum with ∓)
     │       └── tx_53_by_4  [Agent_4]  P:0  Step 3 (½[a/(1-a)² + b/(1-b)²])
     │           ├── tx_92_by_13 [Agent_13] P:0  (see Step 4 above)
     │           └── tx_103_by_0 [Agent_0]  P:0  (see Step 4 above)
     └── tx_48_by_2  [→ see tx_2 subtree]
```

**Legend**: `★` = mathematically valuable non-GP node | `P:0` = non-golden | `P:1` = golden path | `╠══` = golden path edge

---

## Classification Summary

### By Proof Step

| Step | GP Node | Non-GP Nodes | Redundancy |
|------|---------|-------------|------------|
| Step 1: Define S(N) + convergence | tx_7_by_10 | 12 nodes | 12 agents independently wrote Step 1 |
| Step 2: Euler + Σmz^m | tx_24_by_14 | ~18 nodes | Two method branches (Re vs dual-sum) |
| Step 3: Laurent expansion | tx_70_by_10 | ~15 nodes | Most stopped at closed form, didn't expand |
| Step 4: Limit = -1/12 | tx_112_by_9 | ~5 nodes | 3 agents independently derived Laurent |
| **Total** | **4** | **57** | **93% redundancy** |

### By Method Branch

```
                   ROOT
                    │
            ┌───────┴───────┐
            │               │
      Re PATH (simpler)  DUAL-SUM PATH (GP chose this)
     cos = Re(e^{iθ})    cos = (e^{iθ}+e^{-iθ})/2
            │               │
    S = Re(z/(1-z)²)    S = ½[z₁/(1-z₁)² + z₂/(1-z₂)²]
            │               │
     Agents: 0,4,6,      Agents: 2,3,14
             7,8,12              │
            │               │
            └───────┬───────┘
                    │
           tx_57_by_8 (Agent_8)
           PROVED EQUIVALENCE ★
           z₂ = conj(z₁) → Re(...)
```

### Valuable Non-Golden Nodes (10 nodes with unique mathematical content)

| Node | Agent | Value |
|------|-------|-------|
| tx_6_by_14 | Agent_14 | Combined Step 1+2 in one node (aggressive but mathematically valid) |
| tx_57_by_8 | Agent_8 | **Proved z₂ = z̄₁ equivalence** — bridges Re and dual-sum paths |
| tx_84_by_12 | Agent_12 | Real-valued closed form with (r, θ) parametrization |
| tx_91_by_14 | Agent_14 | Independent Laurent: `= N²/(1-i)² - 1/12 + O(1/N)` |
| tx_92_by_13 | Agent_13 | Independent Laurent (same result, different derivation) |
| tx_103_by_0 | Agent_0 | Independent Laurent: `1/w² - 1/12 + O(w²)` |
| tx_109_by_2 | Agent_2 | Extended Laurent: `f(z) = 1/z² - 1/12 + z/12 + O(z²)` (extra term) |
| tx_73_by_11 | Agent_11 | Taylor expansion of `e^{-(1-i)ε}` (concrete computation) |
| tx_60_by_9 | Agent_9 | Alternative rewrite: `Re[exp(z)/(exp(z)-1)²]` |
| tx_105_by_8 | Agent_8 | Step 4 Taylor details for `1-e^{-w}` |

### Pure Redundancy Nodes (~47 nodes, no unique content)

Nodes that are strict restatements of content already on the tape. Examples:
- 12 copies of "S(N) converges absolutely by ratio test"
- 8 copies of "apply Σmz^m = z/(1-z)²"
- 5 copies of "rewrite using Euler's formula"

---

## Comparison: Zeta (Success) vs AIME P15 (Failure)

| Metric | Zeta Sum Proof | AIME 2025 I P15 |
|--------|---------------|-----------------|
| Nodes | 61 | 310 |
| Frontier at end | ~10 | 54 |
| GP depth | 4 | N/A (no OMEGA) |
| Price differentiation | **0 vs 1 (perfect)** | **48-52% (flat)** |
| Redundancy | ~93% | ~95% |
| Valuable non-GP nodes | 10/57 (18%) | ~30/310 (10%) |
| Method branches | 2 (Re vs dual-sum) | 1 (3-adic, no alternatives) |
| OMEGA attempts | 1 (success) | 8 (all failed) |
| Problem difficulty | Medium (standard complex analysis) | Extreme (3-adic Hensel lifting) |

### Why Zeta Succeeded and AIME Failed

1. **Intermediate verifiability**: Each zeta step (Taylor expansion, algebraic identity) can be independently verified by any agent. AIME P15's intermediate steps (3-adic counting) cannot be verified until the full chain is complete.

2. **Price signal**: Zeta's market perfectly separated GP (Price=1) from non-GP (Price=0). AIME's market was flat at 50% — no information signal, no resource guidance.

3. **Problem structure**: Zeta decomposes into 4 clean, sequential steps. AIME P15 requires simultaneous computation of N_0, N_1, N_2 (parallel subproblems) plus Hensel lifting at each level — the DAG should branch then reconverge, but agents couldn't coordinate reconvergence.

4. **Depth vs breadth**: Zeta's DAG has clear depth (Step 1→2→3→4 chains). AIME's DAG spread wide (54 frontier) but shallow (avg depth 5.7) — agents kept opening new branches instead of deepening existing ones.
