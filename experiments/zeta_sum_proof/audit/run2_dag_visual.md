# Zeta Sum Proof Run 11 — Visualized DAG (All 61 Nodes)

**61 nodes | 112 tx | 47 bets on 27 nodes | OMEGA reached**

```
✓ = Golden Path (settled 1.00)   ★ = Insight (correct, not GP, settled 0.00)
◎ = Duplicate (settled 0.00)     [XX%] = live peak price   (50%) = never traded
→ 1.00 = post-settlement GP      B=N = bet count           ⚠ = market anomaly
```

## Complete Citation Tree with Live Pricing

```
ROOT
├── tx_1_by_4 (A4) [(50%)] ◎ "Define S(N), absolute convergence"
│   └── tx_12_by_4 (A4) [(50%)] ◎ "Step 2: cos via Euler's formula"
│       └── tx_61_by_3 (A3) [(50%)] ◎ "Step 3: Σmr^m = r/(1-r)² applied"
│
├── tx_2_by_2 (A2) [(50%)] ◎ "Define S(N), decays exponentially"
│   └── tx_27_by_2 (A2) [(50%)] ◎ "Step 2: Euler → dual sum"
│       └── tx_53_by_4 (A4) [(50%)] ◎ "Step 3: ½[a/(1-a)²+b/(1-b)²]"
│           ├── tx_92_by_13 (A13) [(50%)] ★ "Step 4: Laurent a/(1-a)²=N²/(i-1)²−1/12"
│           └── tx_103_by_0 (A0) [(50%)] ★ "Step 4: Laurent 1/w²−1/12+O(w²)"
│
├── tx_3_by_0 (A0) [(50%)] ◎ "Step 1: regulated sum"
│   └── tx_22_by_0 (A0) [(50%)] ◎ "Step 2: cos=Re[exp], Re path"
│
├── tx_4_by_12 (A12) [(50%)] ◎ "Define S(N), dominating linear growth"
│   ├── tx_21_by_12 (A12) [(50%)] ◎ "Step 2: Re path, r=exp((i-1)/N)"
│   │   └── tx_74_by_6 (A6) [(50%)] ◎ "Step 3: r/(1-r)² converges"
│   │
│   ├── tx_25_by_8 (A8) [(50%)] ◎ "Step 2: Re path, absolute conv"
│   │   ├── tx_39_by_12 (A12) [(50%)] ◎ "Step 3: z/(1-z)² closed form"
│   │   │   └── tx_105_by_8 (A8) [(50%)] ★ "Step 4: Taylor e^{-w}, 1-e^{-w}"
│   │   │
│   │   └── tx_46_by_6 (A6) [43% BEAR B=3] ◎ "Step 3: Σme^{-m(1-i)/N}" ⚠ MARKET ERROR
│   │       │                                    CORRECT node shorted! A4 NO 100, A8 NO 50
│   │       ├── tx_73_by_11 (A11) [(50%)] ★ "Step 4: Taylor e^{-(1-i)ε} expansion"
│   │       └── tx_91_by_14 (A14) [(50%)] ★ "Step 4: e^{-ε}/(1-e^{-ε})²=N²/(1-i)²−1/12"
│   │
│   └── tx_26_by_10 (A10) [(50%)] ◎ "Rewrite using Euler's formula"
│       └── tx_42_by_14 (A14) [(50%)] ◎ "Step 3: cos substituted"
│           └── tx_65_by_2 (A2) [(50%)] ◎ "Step 4: arithmetico-geometric"
│               └── tx_109_by_2 (A2) [(50%)] ★ "Step 5: Laurent f(z)=1/z²−1/12+z/12+O(z²)"
│                                                  Extended Laurent (extra z/12 term!)
│
├── tx_5_by_8 (A8) [(50%)] ◎ "Define S(N), minimal"
│   └── tx_56_by_14 (A14) [(50%)] ◎ "Step 2: Re path, cos=Re(e^{iθ})"
│       ├── tx_78_by_8 (A8) [(50%)] ◎ "Apply formula for Σmz^m" (too brief)
│       └── tx_85_by_7 (A7) [(50%)] ◎ "Step 3: z/(1-z)²"
│
├── tx_6_by_14 (A14) [(50%)] ★ "Step 1+2 combined: define+Euler+dual-sum in one node"
│
├── tx_7_by_10 (A10) [52% → 1.00] ✓ "Step 1: ratio test lim(m+1)/m·e^{-1/N}<1"  ← GP
│   │   B=2 BULL: A12 YES 20→51%, A1 YES 20→52%
│   │   ONLY GP NODE THAT RECEIVED MARKET INVESTMENT
│   │
│   └── tx_24_by_14 (A14) [(50%) → 1.00] ✓ "Step 2: cos=(e+e⁻)/2, z₁/(1-z₁)²+z₂/(1-z₂)²"  ← GP
│       │   DUAL-SUM PATH. Zero bets on this GP node!
│       │
│       ├── tx_57_by_8 (A8) [(50%)] ★ "z₂=conj(z₁) → S=Re(z₁/(1-z₁)²)"
│       │                               BRIDGES dual-sum ↔ Re path. Most valuable non-GP.
│       │                               ZERO BETS. Mathematical gem, market blind.
│       │
│       ├── tx_70_by_10 (A10) [(50%) → 1.00] ✓ "Step 3: 1/((i-1)²ε²)−1/12+O(ε²)"  ← GP
│       │   │   THE −1/12 CONSTANT TERM EMERGES HERE.
│       │   │   Zero bets on this critical GP node!
│       │   │
│       │   └── tx_112_by_9 (A9) [(50%) → 1.00] ✓ "Step 4: (i-1)²=−2i, Re(i/2)=0, lim=−1/12"
│       │                                            [COMPLETE] → OMEGA VERIFIED BY LEAN 4
│       │                                            Zero bets on OMEGA node!
│       │
│       └── tx_84_by_12 (A12) [(50%)] ★ "Step 3: real closed form via (r,θ) parametrize"
│                                          Alternative representation. Never priced.
│
├── tx_8_by_6 (A6) [(50%)] ◎ "Define S(N), ratio test"
│   ├── tx_23_by_6 (A6) [(50%)] ◎ "Step 2: cos=(e+e⁻)/2, dual sum"
│   │   ├── tx_83_by_10 (A10) [(50%)] ◎ "Step 3: arithmetico-geometric formula"
│   │   └── tx_87_by_1 (A1) [(50%)] ◎ "Step 3: Σmr^m=r/(1-r)²"
│   └── tx_34_by_4 (A4) [(50%)] ◎ "Step 2: Re path, direct closed form"
│
├── tx_11_by_11 (A11) [(50%)] ◎ "Define S(N), exp decay dominates"
│   └── tx_38_by_8 (A8) [(50%)] ◎ "Step 2: Re path"
│       └── tx_67_by_8 (A8) [(50%)] ◎ "Step 3: z/(1-z)²"
│
├── tx_17_by_13 (A13) [(50%)] ◎ "Define S(N), cos=Re(exp) seed"
│   └── tx_36_by_0 (A0) [(50%)] ◎ "Step 2: Σmx^m=x/(1-x)² applied"
│       └── tx_60_by_9 (A9) [(50%)] ★ "Step 3: Re[exp(z)/(exp(z)-1)²]"
│                                        Alternative formulation. Never priced.
│
├── tx_18_by_9 (A9) [(50%)] ◎ "Define S(N), ratio test detailed"
│   └── tx_48_by_2 (A2) [(50%)] ◎ "complex exponential closed form" (vague)
│       └── tx_93_by_11 (A11) [(50%)] ◎ "Step 3: z/(1-z)²"
│
├── tx_29_by_3 (A3) [(50%)] ◎ "Define S(N), ratio test"
│   └── tx_51_by_0 (A0) [(50%)] ◎ "Rewrite using complex exponential"
│       └── tx_86_by_6 (A6) [(50%)] ◎ "Step 3: Euler substitute"
│           └── tx_101_by_10 (A10) [(50%)] ◎ "Step 4: Σmz^m for z₁,z₂"
│
├── tx_41_by_5 (A5) [(50%)] ◎ "Step 1: ratio test" (late, tx 41)
│   └── tx_66_by_6 (A6) [(50%)] ◎ "Step 2: Re path + identity"
│       └── tx_102_by_14 (A14) [(50%)] ◎ "Step 3: r/(1-r)²"
│
├── tx_47_by_1 (A1) [(50%)] ◎ "Step 1: m=1 start (not m=0)"
│   └── tx_76_by_4 (A4) [(50%)] ◎ "Step 2: Re path, z/(1-z)²"
│       └── tx_99_by_6 (A6) [(50%)] ◎ "Step 3: substitute z in ω=1/N"
│
└── tx_49_by_7 (A7) [(50%)] ◎ "Define S(N), Re(exp) direct"
    ├── tx_100_by_4 (A4) [(50%)] ◎ "Step 2: exp((i-1)/N)/(1-exp)²"
    └── tx_104_by_3 (A3) [(50%)] ◎ "Step 2: Σm·exp converges, closed form"
```

## Detached Heavily-Traded Nodes (not in citation tree, but significant capital)

```
⚠ tx_5_by_14 (A14) [90%!!] ◎  ← Agent_6 bet 2000 Coins. Settled 0. WHALE DISASTER.
  tx_6_by_6  (A6)  [55%]   ◎  ← A10 YES 100C. Endorsed a duplicate.
  tx_13_by_3 (A3)  [52%]   ◎  ← A0+A4+A6 total 40C YES. Triple endorsement on dup.
  tx_40_by_0 (A0)  [41%]   ◎  ← A4+A5+A2 total 200C NO. MARKET ERROR: correct math killed.
  tx_21_by_8 (A8)  [42%]   ◎  ← A8+A12+A14 total 170C NO. MARKET ERROR: correct killed.
  tx_1_by_2  (A2)  [44%]   ◎  ← A0+A8+A4+A2 total 140C NO. Correct math shorted.
  tx_20_by_11(A11) [45%]   ◎  ← A8+A2 total 100C NO. Correct math shorted.
  tx_91_by_8 (A8)  [55%]   ◎  ← A8 AUTO-LONG 100C on own Step 4 attempt. Lost.
```

## Summary Statistics

```
 ┌──────────────┬───────┬─────┬──────────────┬────────────────────────────────────┐
 │ Category     │ Nodes │  %  │ Market Price │ Note                               │
 ├──────────────┼───────┼─────┼──────────────┼────────────────────────────────────┤
 │ ✓ GP         │   4   │  7% │ 52% or (50%) │ only tx_7 got bets, 3 invisible   │
 │ ★ Insight    │  10   │ 16% │ ALL (50%)    │ all 10 insights = ZERO bets       │
 │ ◎ Duplicate  │  42   │ 69% │ 41-55%       │ some endorsed, some wrongly killed │
 │ △ Partial    │   4   │  7% │ ALL (50%)    │ correct direction, incomplete      │
 │ ⚠ Whale      │   1   │  2% │ 90% bubble   │ 2000C lost, worst signal          │
 │ ✗ Error      │   0   │  0% │ N/A          │ NO errors in this run             │
 ├──────────────┼───────┼─────┼──────────────┼────────────────────────────────────┤
 │ Market Score │       │     │   1/10       │ Proof found by exploration alone   │
 └──────────────┴───────┴─────┴──────────────┴────────────────────────────────────┘
```
