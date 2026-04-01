# Zeta ζ-Sum — Role Trifecta Run (Pro/Qwen2.5-7B-Instruct)

**99 nodes | 300 tx | 58 traded | 41 untraded | NOT proved**
**Model**: Pro/Qwen2.5-7B-Instruct (SiliconFlow) | **Roles**: 5M/5B+/5B-
**BUY YES**: 141 | **BUY NO**: 54 | **Ratio**: 2.6:1
**Duration**: ~4 min | **Throughput**: ~75 tx/min

> **Experiment 2026-04-01**: Architect hypothesis — role differentiation (5 Math / 5 Bull / 5 Bear)
> drives economic activity and reduces duplicate nodes. Weak model (7B) tests architecture, not math ability.

```
● = P=1 (high price)   ○ = P=0 (low price)
M = Mathematician(0-4)  B+ = Bull(5-9)  B- = Bear(10-14)
[BULL xY B=n] = YES-dominant trades   [BEAR xN B=n] = NO-dominant trades
(50%) = never traded   ⚠WHALE = single position >500C
```

## Citation Tree (3 roots)

```
ROOT (99 nodes, 58 traded, 41 untraded)
├── tx_1_by_14 (A14/B-) [BULL 42432Y/19697N B=24 ⚠WHALE] ● Consider the behavior of M(m,N) for large N.
│   └── tx_27_by_6 (A6/B+) [BEAR 121Y/119N B=5] ● For large N, M(m,N) ≈ m * cos(m/N), oscillates but dominated by m
│       └── tx_57_by_3 (A3/M) [BULL 230Y/42N B=4] ● For large N, S(N) dominated by linear term m → divergent series
│           └── tx_82_by_6 (A6/B+) [BEAR 833N B=1 ⚠WHALE] ○ (leaf — shorted hard)
├── tx_2_by_8 (A8/B+) [BEAR 40N B=1] ○ Define S(N), aim to show lim N→∞ S(N) = -1/12
│   └── tx_36_by_9 (A9/B+) [BEAR 98N B=1] ○ M(m,N) ≈ m as N→∞, so Σm diverges naively
│       ├── tx_39_by_8 (A8/B+) [BEAR 40N B=1] ○ Series Σ m*exp(-m/N) — formula for exponential series
│       │   ├── tx_45_by_2 (A2/M) [MIXED 40Y/40N B=2] ● For large N, integral of m*exp(-m/N) → N²
│       │   │   └── tx_69_by_3 (A3/M) [MIXED 40Y/40N B=2] ● Geometric series + differentiation
│       │   │       └── tx_79_by_13 (A13/B-) [BEAR 40N B=1] ○ exp(-1/N) ≈ 1-1/N, substitute → N²
│       │   │           ├── tx_112_by_0 (A0/M) [BULL 833Y B=1 ⚠WHALE] ● cos(m/N) average → 1/2 ★
│       │   │           │   └── tx_123_by_3 (A3/M) (50%) ○ cos average → 0
│       │   │           │       └── tx_181_by_3 (A3/M) [BULL 191Y B=1] ● exp approach 1, cos oscillates
│       │   │           │           └── tx_219_by_3 (A3/M) [BEAR 59N B=1] ○ S(N) ≈ (1/2)Σm → divergent
│       │   │           │               └── tx_252_by_8 (A8/B+) [BULL 44Y B=2] ● Integral approx ∫x*exp(-x/N)dx
│       │   │           │                   └── tx_281_by_7 (A7/B+) [BULL 40Y B=1] ● (1/2)Σm approximation
│       │   │           │                       └── tx_297_by_5 (A5/B+) [BULL 20Y B=1] ● (leaf)
│       │   │           └── tx_114_by_4 (A4/M) [BULL 98Y B=1] ● cos(m/N) oscillates, average → 1/2
│       │   │               └── tx_162_by_0 (A0/M) (50%) ○ cos average close to 0
│       │   │                   └── tx_205_by_4 (A4/M) [MIXED 40Y/40N B=2] ● cos doesn't affect convergence
│       │   └── tx_48_by_3 (A3/M) [BEAR 176N B=4] ○ exp(-m/N)*cos(m/N)→1, M(m,N)≈m for large N
│       │       └── tx_195_by_4 (A4/M) [BULL 40Y B=1] ● Gamma function integral
│       │           ├── tx_221_by_4 (A4/M) [BEAR 98N B=1] ○ Series expansion exp+cos
│       │           └── tx_222_by_7 (A7/B+) (50%) ○ (dead branch)
│       │               └── tx_238_by_12 (A12/B-) [BEAR 40N B=1] ○ (leaf)
│       └── tx_40_by_3 (A3/M) (50%) ○ M(m,N) decays exponentially → series converges
│           └── tx_66_by_6 (A6/B+) [BULL 209Y B=4] ● S(N) for large N
│               └── tx_93_by_1 (A1/M) [BULL 44Y/40N B=3] ● Apply DCT
│                   ├── tx_103_by_4 (A4/M) (50%) ○ Apply DCT
│                   │   ├── tx_134_by_4 (A4/M) [BEAR 40N B=1] ○ DCT application
│                   │   │   └── tx_148_by_3 (A3/M) [BULL 59Y B=1] ● DCT → interchange limit & sum
│                   │   │       └── tx_168_by_4 (A4/M) [BEAR 40Y/79N B=3] ○ Prepare DCT
│                   │   │           └── tx_194_by_10 (A10/B-) (50%) ○ Prepare DCT
│                   │   │               └── tx_300_by_3 (A3/M) (50%) ○ Apply DCT (leaf)
│                   │   └── tx_137_by_7 (A7/B+) [BULL 298Y B=5] ● Apply DCT
│                   │       └── tx_190_by_14 (A14/B-) [BULL 78Y B=2] ● DCT interchange
│                   │           └── tx_216_by_1 (A1/M) [BULL 360Y/150N B=7] ● DCT → evaluate sum
│                   │               └── tx_229_by_9 (A9/B+) [BULL 212Y/41N B=5] ● Apply DCT
│                   │                   └── tx_242_by_14 (A14/B-) (50%) ○ (leaf)
│                   └── tx_104_by_13 (A13/B-) [MIXED 40Y/40N B=2] ○ exp(-m/N) bound → series converges
│                       └── tx_111_by_1 (A1/M) [BULL 586Y/52N B=14 ⚠WHALE] ● DCT + uniform convergence ★★
│                           └── tx_155_by_10 (A10/B-) [BULL 667Y/125N B=12 ⚠WHALE] ● Limit → termwise ★
│                               └── tx_184_by_3 (A3/M) (50%) ○ (dead branch)
│                                   └── tx_227_by_14 (A14/B-) (50%) ○ DCT → find limit
│                                       └── tx_253_by_0 (A0/M) (50%) ○ DCT → find limit
│                                           └── tx_277_by_12 (A12/B-) (50%) ○ DCT → limit
│                                               └── tx_296_by_3 (A3/M) (50%) ○ DCT (leaf)
└── tx_3_by_4 (A4/M) [BULL 1648Y B=3 ⚠WHALE] ● Define S(N) = Σ M(m,N), converges for fixed N ★
    └── tx_26_by_5 (A5/B+) [BULL 191Y B=1] ● "Limit N→∞ of S(N) = -1/12" (hand-wave)
        ├── tx_41_by_1 (A1/M) [MIXED 40Y/10N B=2] ● exp(-m/N)→1, cos(m/N)→1 for fixed m
        │   └── tx_52_by_14 (A14/B-) [BULL 6019Y/677N B=8 ⚠WHALE] ● exp→1, cos oscillates, averages 0 ★★★
        │       └── tx_83_by_4 (A4/M) [BEAR 176N B=4] ○ S(N) approximated by integral
        │           └── tx_95_by_0 (A0/M) (50%) ○ Σ m*exp(-m/N) with decay for m>0
        │               └── tx_118_by_4 (A4/M) [MIXED 128Y/267N B=6] ○ Integral → N²/12
        │                   └── tx_186_by_2 (A2/M) [BEAR 40N B=1] ○ Integral → N²/12
        │                       └── tx_210_by_3 (A3/M) [BULL 101Y B=2] ● Integral → N²/12
        │                           └── tx_231_by_2 (A2/M) [BULL 180Y/81N B=6] ● Gamma function → N²/12
        │                               └── tx_249_by_4 (A4/M) [BULL 556Y/90N B=9 ⚠WHALE] ● ∫m*exp(-m/N)→N²/12 via IBP
        │                                   └── tx_280_by_8 (A8/B+) [BULL 10489Y B=7 ⚠WHALE] ● ∫x*exp(-x/N)dx ★★★
        └── tx_42_by_4 (A4/M) [MIXED 44Y/79N B=4] ○ Consider limit N→∞
            └── tx_49_by_0 (A0/M) [BEAR 40N B=1] ○ Analyze M(m,N) as N→∞
                ├── tx_54_by_10 (A10/B-) (50%) ○ M(m,N)→m for large N
                │   └── tx_76_by_3 (A3/M) (50%) ○ S(N)→Σm diverges naively
                │       └── tx_90_by_4 (A4/M) (50%) ○ Consider integral
                │           ├── tx_106_by_1 (A1/M) (50%) ○ Consider integral
                │           │   └── tx_139_by_1 (A1/M) (50%) ○ Consider integral
                │           │       ├── tx_188_by_5 (A5/B+) (50%) ○ Consider integral
                │           │       │   ├── tx_214_by_1 (A1/M) (50%) ○ Consider integral
                │           │       │   │   └── tx_246_by_13 (A13/B-) (50%) ○ Consider integral
                │           │       │   │       └── tx_284_by_10 (A10/B-) (50%) ○ Conclude → -1/12
                │           │       │   └── tx_215_by_11 (A11/B-) (50%) ○ divergent integral
                │           │       │       └── tx_282_by_2 (A2/M) [BULL 193Y/42N B=3] ● Consider integral
                │           │       └── tx_204_by_3 (A3/M) (50%) ○ Consider integral
                │           │           ├── tx_211_by_4 (A4/M) (50%) ○ Consider integral
                │           │           │   ├── tx_263_by_1 (A1/M) (50%) ○ Consider integral
                │           │           │   │   └── tx_287_by_3 (A3/M) (50%) ○ (leaf)
                │           │           │   └── tx_267_by_1 (A1/M) (50%) ○ Consider integral
                │           │           │       └── tx_294_by_0 (A0/M) (50%) ○ Consider integral
                │           │           │           └── tx_299_by_1 (A1/M) (50%) ○ (leaf)
                │           │           └── tx_220_by_10 (A10/B-) (50%) ○ Consider integral
                │           │               └── tx_239_by_9 (A9/B+) (50%) ○ Consider integral
                │           │                   └── tx_270_by_7 (A7/B+) [BULL 40Y B=1] ● Consider integral
                │           │                       └── tx_283_by_4 (A4/M) (50%) ○ (leaf)
                │           └── tx_108_by_4 (A4/M) [BULL 44Y B=2] ● S(N) converges, use convergence
                │               └── tx_113_by_3 (A3/M) [BEAR 40N B=1] ○ Consider integral
                │                   └── tx_127_by_7 (A7/B+) (50%) ○ (dead branch)
                │                       └── tx_135_by_3 (A3/M) (50%) ○ (dead branch)
                │                           └── tx_166_by_3 (A3/M) (50%) ○ Consider integral
                │                               └── tx_170_by_1 (A1/M) [BULL 4Y B=1] ● Consider integral
                │                                   └── tx_173_by_4 (A4/M) [BULL 78Y B=2] ● Consider integral
                │                                       └── tx_179_by_4 (A4/M) [BULL 98Y B=1] ● Consider integral
                │                                           └── tx_199_by_8 (A8/B+) [BULL 298Y B=5] ● Apply DCT
                │                                               └── tx_233_by_13 (A13/B-) (50%) ○ (leaf)
                └── tx_55_by_3 (A3/M) [BULL 40Y B=1] ● S(N) dominated by linear term → divergent
                    └── tx_71_by_10 (A10/B-) [BEAR 40Y/194N B=3] ○ M(m,N)→m for large N
                        └── tx_156_by_13 (A13/B-) [BULL 22Y/40N B=3] ○ M(m,N)→m, series diverges
                            ├── tx_264_by_4 (A4/M) (50%) ○ exp(-m/N) → convergence analysis
                            │   └── tx_278_by_6 (A6/B+) (50%) ○ exp(-m/N) → convergence
                            └── tx_265_by_4 (A4/M) (50%) ○ Analyze terms (leaf)
```

## Role Activity Breakdown

| Role | Agents | Nodes Created | % of Total |
|------|--------|--------------|------------|
| Mathematician | 0-4 | 60 | 61% |
| Bull | 5-9 | 20 | 20% |
| Bear | 10-14 | 19 | 19% |

## Trading Activity by Role

| Role | YES Buys | NO Buys | Net Direction |
|------|----------|---------|---------------|
| Mathematician | 23 | 3 | BULL |
| Bull | 91 | 8 | BULL |
| Bear | 27 | 43 | BEAR |

## Market Scorecard — vs Run 2 (DeepSeek, no roles)

| Metric | Run 2 (DeepSeek) | Run 3 (Qwen2.5-7B, roles) |
|--------|-----------------|---------------------------|
| Nodes | 61 | **99** |
| Traded | 26/61 (43%) | **58/99 (59%)** |
| BUY YES | ~20 | **141** |
| BUY NO | ~0 | **54** |
| YES:NO ratio | ∞:0 (zero shorts) | **2.6:1** |
| WHALE events | 1 | **7** |
| Max depth | 4 | **11** |
| Branching factor | ~2.5 | ~1.3 (more linear chains) |
| Proved? | YES (OMEGA) | NO |
| Model capability | DeepSeek-V3+R1 | Qwen2.5-7B (weak) |

## Key Findings

### 1. Role Differentiation — VALIDATED
- **Bears actually short**: 43 NO buys (vs zero in Run 2). The "零做空" problem is solved.
- **Bulls concentrate capital**: 91 YES buys, aggressive on promising nodes.
- **Mathematicians build**: 61% of all nodes, with rare but confident investments.
- **Roles are bias, not constraint**: Bear A14 created tx_52 (highest-valued node!) and Bear A11 bought 1000 YES on tx_3. Agents cross role boundaries when evidence is strong.

### 2. Economic Activity — DRAMATICALLY IMPROVED
- **59% nodes traded** (vs 43% in Run 2)
- **195 total trades** (YES+NO) across 300 tx
- **Multi-bet consensus**: tx_111 (14 bets), tx_155 (12 bets), tx_52 (8 bets) — genuine price discovery
- **Contested nodes**: tx_118 (128Y vs 267N), tx_71 (40Y vs 194N) — real bull/bear disagreement

### 3. Whale Behavior — EMERGENT
- tx_1_by_14: **42,432Y / 19,697N** (24 bets!) — root node became a battleground
- tx_52_by_14: **6,019Y / 677N** — Bear-created node became most endorsed
- tx_280_by_8: **10,489Y** — Bull all-in at the deepest chain endpoint
- tx_111_by_1: **586Y / 52N** (14 bets) — strongest consensus node

### 4. Math Quality — WEAK (expected for 7B)
- Proof does NOT reach -1/12. Agents circle around "DCT", "integral", "cos average" without rigorous computation.
- Many duplicate-ish nodes ("Consider the integral representation..." repeated ~15 times)
- No agent uses the hint formula M(m,N) = m * exp(-m/N) * cos(m/N) with complex exponential z = exp((i-1)/N)
- **This is expected**: 7B model lacks the mathematical sophistication. The experiment tests market dynamics, not proof ability.

### 5. DAG Structure — TOO LINEAR
- Only 3 roots (vs 15+ in Run 2). Most of the DAG is long linear chains.
- Max depth = 11 (good for depth), but branching factor ~1.3 (bad for exploration).
- The "dead zone" under tx_49 has 30+ nodes with zero trading — capital doesn't flow to deep branches.
- **Diagnosis**: Boltzmann selection works but agents tend to extend the same chain rather than starting new roots.

### 6. Anti-Zombie — NOT TRIGGERED
- Despite many near-duplicate nodes ("Consider the integral..."), the anti-zombie pruning (threshold=3) didn't fire significantly. 7B model paraphrases just enough to avoid exact-match detection.
