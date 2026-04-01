# Zeta Sum Proof Run 11 — Visualized DAG (All 61 Nodes, Corrected Pricing)

**61 nodes | 112 tx | 26 nodes traded | 35 untraded | OMEGA reached**

> **CORRECTION 2026-04-01**: Previous version had incorrect node ID mapping
> (tape IDs ≠ log IDs). Pricing now verified via tape→log agent-order matching.

```
✓ = Golden Path (settled 1.00)   ★ = Insight (correct, not GP)   ◎ = Duplicate
→1.00 = settled on GP            ⚠WHALE = single bet >1000C
[lo-hi% BULL/BEAR xY/xN B=n] = live price range, direction, YES/NO coins, bet count
(50%) = truly never traded
```

## Complete Citation Tree with Verified Live Pricing

```
ROOT (61 nodes, 26 traded, 35 untraded)
├── tx_1_by_4 (A4) [(50%)] ◎ Define the regularized sum S(N) = Σ_{m=0}^∞ m
│   └── tx_12_by_4 (A4) [(50%)] ◎ Step 2: Express cos(m/N) using Euler's formul
│       └── tx_61_by_3 (A3) [46-50% BEAR 10Y/100N B=3] ◎ Step 3: For each geometric-like series, use t
├── tx_2_by_2 (A2) [44-50% BEAR 0Y/140N B=4] ◎ Define the regularized sum S(N) = Σ_{m=0}^∞ m
│   └── tx_27_by_2 (A2) [48-50% BEAR 0Y/50N B=1] ◎ Rewrite the cosine factor using Euler's formu
│       └── tx_53_by_4 (A4) [(50%)] ◎ Step 3: Apply the identity Σ_{m=0}^∞ m z^m = 
│           ├── tx_92_by_13 (A13) [(50%)] ★ Step 4: To evaluate the limit N→∞, expand a a
│           └── tx_103_by_0 (A0) [49-50% BEAR 0Y/20N B=1] ★ Step 4: Using Taylor expansion for small w = 
├── tx_3_by_0 (A0) [(50%)] ◎ Step 1: For each fixed positive integer N, de
│   └── tx_22_by_0 (A0) [50-51% BULL 20Y/0N B=1] ◎ Step 2: Using Euler's formula, cos(m/N) = Re[
├── tx_4_by_12 (A12) [(50%)] ◎ Define S(N) = Σ_{m=0}^∞ m exp(-m/N) cos(m/N).
│   ├── tx_21_by_12 (A12) [50-50% BULL 10Y/0N B=1] ◎ Rewrite S(N) using complex exponential: since
│   │   └── tx_74_by_6 (A6) [(50%)] ◎ Step 3: Since |r| = exp(-1/N) < 1 for any N >
│   ├── tx_25_by_8 (A8) [42-50% BEAR 0Y/170N B=4] ◎ Step 2: Since the series converges absolutely
│   │   ├── tx_39_by_12 (A12) [(50%)] ◎ Step 3: Let z = e^{-(1-i)/N}. Since |z| = e^{
│   │   │   └── tx_105_by_8 (A8) [(50%)] ★ Step 4: Set ε = 1/N and w = (1-i)ε. For small
│   │   └── tx_46_by_6 (A6) [43-50% BEAR 20Y/150N B=3] ◎ Step 3: Using the formula for the sum of a ge
│   │       ├── tx_73_by_11 (A11) [48-50% BEAR 0Y/40N B=2] ★ Step 4: Let ε = 1/N. For small ε, expand e^{-
│   │       └── tx_91_by_14 (A14) [(50%)] ★ Step 4: For large N, using the Taylor expansi
│   └── tx_26_by_10 (A10) [(50%)] ◎ Rewrite S(N) using Euler's formula to express
│       └── tx_42_by_14 (A14) [50-50% BULL 10Y/0N B=1] ◎ Step 3: Substituting cos(m/N) = (e^{i m/N} + 
│           └── tx_65_by_2 (A2) [50-51% BULL 20Y/0N B=1] ◎ Step 4: Apply the summation formula for arith
│               └── tx_109_by_2 (A2) [(50%)] ★ Step 5: Using the Laurent expansion of f(z)=e
├── tx_5_by_8 (A8) [48-50% BEAR 0Y/50N B=1] ◎ Define for each positive integer N the regula
│   └── tx_56_by_14 (A14) [48-50% BEAR 0Y/50N B=1] ◎ Rewrite the sum using Euler's formula: cos(m/
│       ├── tx_78_by_8 (A8) [(50%)] ◎ Apply the formula for the sum of m*z^m.
│       └── tx_85_by_7 (A7) [(50%)] ◎ Step 3: Apply the formula for the sum of a ge
├── tx_6_by_14 (A14) [90% ⚠WHALE 2000Y] ★ Define S(N) = Σ_{m=0}^∞ m exp(-m/N) cos(m/N).
├── tx_7_by_10 (A10) [50-52% BULL 40Y/0N B=2 →1.00] ✓ Step 1: For each fixed N > 0, consider the se
│   └── tx_24_by_14 (A14) [(50%) →1.00] ✓ Step 2: Using Euler's formula, cos(m/N) = (e^
│       ├── tx_57_by_8 (A8) [48-51% BEAR 20Y/70N B=3] ★ Step 3: Since z2 = \overline{z1}, the two ter
│       ├── tx_70_by_10 (A10) [(50%) →1.00] ✓ Step 3: For large N, set ε = 1/N → 0. Using t
│       │   └── tx_112_by_9 (A9) [(50%) →1.00] ✓ Step 4: Substituting the asymptotic expansion
│       └── tx_84_by_12 (A12) [(50%)] ★ Step 3: Let a = 1/N. Then z1 = e^{-a} e^{ia} 
├── tx_8_by_6 (A6) [50-55% BULL 100Y/0N B=1] ◎ Define S(N) = Σ_{m=0}^∞ m * exp(-m/N) * cos(m
│   ├── tx_23_by_6 (A6) [(50%)] ◎ Step 2: Express cos(m/N) in terms of complex 
│   │   ├── tx_83_by_10 (A10) [(50%)] ◎ Then S(N) can be evaluated using the formula 
│   │   └── tx_87_by_1 (A1) [(50%)] ◎ Step 3: Recognize that for |r| < 1, Σ_{m=0}∞ 
│   └── tx_34_by_4 (A4) [50-51% BULL 20Y/0N B=1] ◎ Step 2: Express S(N) using complex exponentia
├── tx_11_by_11 (A11) [45-50% BEAR 0Y/100N B=2] ◎ Define the regularized sum S(N) = Σ_{m=0}^∞ m
│   └── tx_38_by_8 (A8) [(50%)] ◎ Step 2: Using Euler's formula, cos(m/N) = Re(
│       └── tx_67_by_8 (A8) [50-55% BULL 100Y/0N B=1] ◎ Step 3: Recognize that for |z| < 1, the sum Σ
├── tx_17_by_13 (A13) [(50%)] ◎ Define S(N) = Σ_{m=0}^∞ m exp(-m/N) cos(m/N).
│   └── tx_36_by_0 (A0) [41-50% BEAR 0Y/200N B=3] ◎ Step 2: Using the identity Σ_{m=0}∞ m x^m = x
│       └── tx_60_by_9 (A9) [48-50% BEAR 10Y/50N B=2] ★ Step 3: Using the identity 1 - exp(-z) = exp(
├── tx_18_by_9 (A9) [48-50% BEAR 0Y/50N B=1] ◎ Define S(N) = Σ_{m=0}^∞ m e^{-m/N} cos(m/N). 
│   └── tx_48_by_2 (A2) [50-50% BULL 10Y/0N B=1] ◎ Using complex exponential representation and 
│       └── tx_93_by_11 (A11) [(50%)] ◎ Step 3: Set z = exp((-1+i)/N). Then S(N) = Re
├── tx_29_by_3 (A3) [50-52% BULL 40Y/0N B=3] ◎ Define S(N) = Σ_{m=0}^∞ m exp(-m/N) cos(m/N).
│   └── tx_51_by_0 (A0) [(50%)] ◎ Rewrite S(N) using complex exponential to fac
│       └── tx_86_by_6 (A6) [(50%)] ◎ Step 3: Using Euler's formula, cos(m/N) = (e^
│           └── tx_101_by_10 (A10) [(50%)] ◎ Step 4: Using the formula Σ_{m=0}^∞ m z^m = z
├── tx_41_by_5 (A5) [(50%)] ◎ Step 1: Define the regularized sum S(N) = Σ_{
│   └── tx_66_by_6 (A6) [48-50% BEAR 0Y/50N B=1] ◎ Step 2: Recognize that cos(m/N) = Re(e^{i m/N
│       └── tx_102_by_14 (A14) [(50%)] ◎ Step 3: Using the identity Σ_{m=0}^∞ m r^m = 
├── tx_47_by_1 (A1) [(50%)] ◎ Step 1: For any fixed N > 0, define S(N) = Σ_
│   └── tx_76_by_4 (A4) [(50%)] ◎ Step 2: Express S(N) using complex exponentia
│       └── tx_99_by_6 (A6) [(50%)] ◎ Continue the computation by substituting z in
└── tx_49_by_7 (A7) [(50%)] ◎ Define the regularized sum S(N) = Σ_{m=0}^∞ M
    ├── tx_100_by_4 (A4) [(50%)] ◎ Step 2: Since |exp((i-1)/N)| = exp(-1/N) < 1,
    └── tx_104_by_3 (A3) [(50%)] ◎ Since |exp((i-1)/N)| = exp(-1/N) < 1, the ser
```

## Corrected Market Scorecard

With corrected pricing, the picture is different from the previous (flawed) analysis:

| Metric | Previous (wrong) | Corrected |
|--------|-----------------|-----------|
| Nodes traded | "only tx_7" | **26/61 (43%)** |
| GP tx_7 bets | 2 (correct) | 2 (confirmed) |
| GP tx_24,70,112 bets | "zero" | **zero** (confirmed) |
| tx_57 "gem, zero bets" | "zero" | **3 bets, BEAR 20Y/70N** (shorted!) |
| Whale node | "tx_5_by_14" | **tx_6_by_14** (same node, corrected ID) |
| Strongest short | "tx_40 at 41%" | **tx_36_by_0 at 41%** (same node, corrected ID) |

**Key correction**: tx_57 (conjugate bridge ★) was NOT ignored — it was **actively shorted** (BEAR 20Y/70N). The market evaluated it and disagreed. This changes the narrative from "market blind to insights" to "**market actively rejected the most valuable insight**".

| Task | Previous Score | Corrected Score |
|------|---------------|----------------|
| GP Step 1 | 6/10 | 6/10 (confirmed) |
| GP Steps 2-4 | 0/10 | **0/10** (confirmed: truly zero bets) |
| Insight tx_57 | "0/10 (invisible)" | **2/10** (seen but shorted — worse!) |
| Insight tx_103 | "0/10 (invisible)" | **1/10** (1 short, 20N) |
| Whale prevention | 0/10 | 0/10 (confirmed) |
| Duplicate shorting | "some" | **extensive** (multiple dups at 41-48%) |
| **OVERALL** | 1/10 | **2/10** |

The corrected data shows the market was slightly MORE active than previously thought (26 vs ~5 traded nodes), but the core finding holds: **3/4 GP nodes had zero bets, and the most valuable insight (tx_57) was actively shorted rather than endorsed.**
