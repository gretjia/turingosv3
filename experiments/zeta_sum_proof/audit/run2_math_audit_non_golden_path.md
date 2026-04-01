# Zeta Sum Proof Run 11 — Complete DAG: All 61 Nodes with Pricing & Classification

**Run**: Run 11 (global) / Run 2 (zeta local) — 112 tx, 61 nodes, 47 bets, OMEGA reached
**Golden Path**: `tx_7_by_10` → `tx_24_by_14` → `tx_70_by_10` → `tx_112_by_9` (4 steps)
**Post-Settlement**: GP = Price 1.00, all others = Price 0.00
**Live Prices**: shown as P:XX-YY% (range during trading) or (50%) if never traded

---

## Legend

```
✓ GP    = Golden Path (settled 1.00)    ★ INSIGHT = novel correct, not GP (settled 0.00)
◎ DUP   = duplicate content             △ PARTIAL = correct direction, incomplete
⚠ WHALE = extreme single bet (>1000C)   ✗ ERROR   = mathematical error (NONE in this run)

BULL = YES coins > NO coins   |   BEAR = NO coins > YES coins
B=N  = total bet count        |   (50%) = never traded (genesis price)
```

---

## Unified DAG (all 61 nodes, one diagram)

```
╔══════════════════════════════════════════════════════════════════════════════════════
║  ROOT: Prove 1+2+3+... = -1/12 via S(N) = Σ m·exp(-m/N)·cos(m/N), lim N→∞
╠══════════════════════════════════════════════════════════════════════════════════════
║
║ ┌─────────────────────────────────────────────────────────────────────────────────┐
║ │ STEP 1: Define S(N) + Convergence (14 nodes: 1 GP + 13 DUP)                   │
║ └─────────────────────────────────────────────────────────────────────────────────┘
║ │
║ │  ✓ tx_7_by_10  [A10] P:50→52%  B=2 BULL(40Y/0N)  "ratio test: lim(m+1)/m·e^{-1/N}<1"
║ │  │  └─ Agent_12 YES 20→51% + Agent_1 YES 20→52%
║ │  │
║ │  ◎ tx_1_by_4   [A4]  (50%)  B=0  "absolute convergence"
║ │  ◎ tx_2_by_2   [A2]  (50%)  B=0  "decays exponentially"
║ │  ◎ tx_3_by_0   [A0]  (50%)  B=0  "regulated sum"
║ │  ◎ tx_4_by_12  [A12] (50%)  B=0  "dominating linear growth"
║ │  ◎ tx_5_by_8   [A8]  (50%)  B=0  "minimal definition, no proof"
║ │  ◎ tx_8_by_6   [A6]  (50%)  B=0  "ratio test" (same as GP)
║ │  ◎ tx_11_by_11 [A11] (50%)  B=0  "exponential decay dominates"
║ │  ◎ tx_17_by_13 [A13] (50%)  B=0  "cos=Re(exp)" (Re path seed)
║ │  ◎ tx_18_by_9  [A9]  (50%)  B=0  "ratio test detailed"
║ │  ◎ tx_29_by_3  [A3]  (50%)  B=0  "ratio test"
║ │  ◎ tx_41_by_5  [A5]  (50%)  B=0  "ratio test" (late, tx 41)
║ │  ◎ tx_47_by_1  [A1]  (50%)  B=0  "m=1 start" (minor variant)
║ │  ◎ tx_49_by_7  [A7]  (50%)  B=0  "Re(exp) direct"
║ │  │
║ │  │  WASTE: 14 nodes for "define S(N) and prove convergence". 13 duplicates.
║ │  │  MARKET: Only GP tx_7 got 2 bets. Other 13 nodes = zero activity.
║ │
║ │──────────────────────── STEP 1 → STEP 2 ────────────────────────
║ │
║ │              ┌──────────────────┬──────────────────┐
║ │              │  DUAL-SUM PATH   │    Re PATH        │
║ │              │  cos=(e+e⁻)/2    │  cos=Re(e^{iθ})   │
║ │              │  → z₁, z₂        │  → single z       │
║ │              │  (GP chose this) │  (simpler)         │
║ │              └────────┬─────────┴────────┬───────────┘
║ │                       │                  │
║ │ ┌─────────────────────────────────────────────────────────────────────────────────┐
║ │ │ STEP 2: Euler + Geometric Series (19 nodes: 1 GP + 1 INSIGHT + 17 DUP)        │
║ │ └─────────────────────────────────────────────────────────────────────────────────┘
║ │ │
║ │ │  ✓ tx_24_by_14 [A14] (50%)  B=0  DUAL-SUM: "z₁=exp((i-1)/N), z₂=exp((-i-1)/N)"
║ │ │  │  "Σmz^m = z/(1-z)² → S(N) = ½[z₁/(1-z₁)² + z₂/(1-z₂)²]"
║ │ │  │  ⚠ GP NODE HAD ZERO BETS. Market invisible.
║ │ │  │
║ │ │  ★ tx_57_by_8  [A8]  (50%)  B=0  "z₂=conj(z₁) → S(N)=Re(z₁/(1-z₁)²)"
║ │ │  │  BRIDGES dual-sum and Re path. Most valuable non-GP node.
║ │ │  │  ⚠ ZERO BETS. Mathematical gem, market blind.
║ │ │  │
║ │ │  DUAL-SUM duplicates:
║ │ │  ◎ tx_12_by_4  [A4]  (50%)  B=0  cos=(e+e⁻)/2
║ │ │  ◎ tx_23_by_6  [A6]  (50%)  B=0  cos=(e+e⁻)/2
║ │ │  ◎ tx_27_by_2  [A2]  (50%)  B=0  cos=(e+e⁻)/2
║ │ │  ◎ tx_42_by_14 [A14] (50%)  B=0  cos=(e+e⁻)/2 (GP author re-writes own step!)
║ │ │  ◎ tx_53_by_4  [A4]  (50%)  B=0  "½[a/(1-a)²+b/(1-b)²]"
║ │ │  ◎ tx_61_by_3  [A3]  (50%)  B=0  dual sum with ∓
║ │ │  ◎ tx_86_by_6  [A6]  (50%)  B=0  Euler substitute
║ │ │
║ │ │  Re PATH duplicates:
║ │ │  ◎ tx_21_by_12 [A12] (50%)  B=0  "r=exp((i-1)/N)"
║ │ │  ◎ tx_22_by_0  [A0]  (50%)  B=0  "Re[Σm·exp(m(i-1)/N)]"
║ │ │  ◎ tx_25_by_8  [A8]  (50%)  B=0  "Re[Σm·e^{-m(1-i)/N}]"
║ │ │  ◎ tx_26_by_10 [A10] (50%)  B=0  "rewrite using Euler" (vague)
║ │ │  ◎ tx_34_by_4  [A4]  (50%)  B=0  Re path direct closed form
║ │ │  ◎ tx_38_by_8  [A8]  (50%)  B=0  "Re Σm(e^{-(1-i)/N})^m"
║ │ │  ◎ tx_56_by_14 [A14] (50%)  B=0  "Re(Σm[e^{(i-1)/N}]^m)"
║ │ │  ◎ tx_66_by_6  [A6]  (50%)  B=0  Re path + identity
║ │ │  ◎ tx_76_by_4  [A4]  (50%)  B=0  "Re(z/(1-z)²)"
║ │ │
║ │ │  WASTE: 19 nodes for Step 2. 17 duplicates across 2 branches.
║ │
║ │──────────────────────── STEP 2 → STEP 3 ────────────────────────
║ │
║ │ ┌─────────────────────────────────────────────────────────────────────────────────┐
║ │ │ STEP 3: Laurent Expansion (19 nodes: 1 GP + 2 INSIGHT + 16 DUP/PARTIAL)       │
║ │ │ THE CRITICAL STEP: extract -1/12 constant term from e^z/(1-e^z)²               │
║ │ └─────────────────────────────────────────────────────────────────────────────────┘
║ │ │
║ │ │  ✓ tx_70_by_10 [A10] (50%)  B=0  "z₁/(1-z₁)² = 1/((i-1)²ε²) - 1/12 + O(ε²)"
║ │ │  │  THE -1/12 APPEARS HERE. Laurent expansion of both z₁ and z₂ terms.
║ │ │  │  ⚠ GP NODE HAD ZERO BETS. Market invisible to the proof's key step.
║ │ │  │
║ │ │  ★ tx_84_by_12 [A12] (50%)  B=0  "Real closed form: (r,θ) parametrization"
║ │ │  │  INSIGHT: Alternative representation. Never priced.
║ │ │  │
║ │ │  ★ tx_60_by_9  [A9]  (50%)  B=0  "Re[exp(z)/(exp(z)-1)²]"
║ │ │  │  INSIGHT: Equivalent reformulation via exp(z)-1. Never priced.
║ │ │  │
║ │ │  STUCK AT CLOSED FORM (correct Σmz^m=z/(1-z)² but no Laurent expansion):
║ │ │  ◎ tx_36_by_0  [A0]  (50%)  B=0  "Σmx^m=x/(1-x)² applied"
║ │ │  ◎ tx_39_by_12 [A12] (50%)  B=0  "z/(1-z)²" (closed form only)
║ │ │  ◎ tx_46_by_6  [A6]  P:43.1-50% B=3 BEAR(20Y/150N) ⚠ MARKET ERROR
║ │ │  │  "Σm·e^{-m(1-i)/N} = e^{-(1-i)/N}/(1-e^{-(1-i)/N})²"
║ │ │  │  CORRECT math shorted to 43.1%! Agent_4 NO 100, Agent_8 NO 50.
║ │ │  │  Market confused "not on GP" with "mathematically wrong".
║ │ │  │
║ │ │  ◎ tx_48_by_2  [A2]  (50%)  B=0  "complex exponential closed form" (vague)
║ │ │  ◎ tx_51_by_0  [A0]  (50%)  B=0  "rewrite using complex exponential"
║ │ │  ◎ tx_67_by_8  [A8]  (50%)  B=0  "z/(1-z)²"
║ │ │  ◎ tx_74_by_6  [A6]  (50%)  B=0  "r/(1-r)²"
║ │ │  ◎ tx_78_by_8  [A8]  (50%)  B=0  "Apply formula" (too brief, ~20 chars)
║ │ │  ◎ tx_83_by_10 [A10] (50%)  B=0  "arithmetico-geometric"
║ │ │  ◎ tx_85_by_7  [A7]  (50%)  B=0  "z/(1-z)²"
║ │ │  ◎ tx_87_by_1  [A1]  (50%)  B=0  "r/(1-r)²"
║ │ │  ◎ tx_93_by_11 [A11] (50%)  B=0  "z/(1-z)²"
║ │ │  ◎ tx_99_by_6  [A6]  (50%)  B=0  "substitute z in terms of ω"
║ │ │  ◎ tx_100_by_4 [A4]  (50%)  B=0  "exp((i-1)/N)/(1-exp)²"
║ │ │  ◎ tx_102_by_14[A14] (50%)  B=0  "r/(1-r)²"
║ │ │  ◎ tx_104_by_3 [A3]  (50%)  B=0  "Σm·exp(m(i-1)/N) converges"
║ │ │
║ │ │  BOTTLENECK: 16/19 agents derived closed form but couldn't take the
║ │ │  next step (Laurent expansion). Only tx_70 broke through to -1/12.
║ │
║ │──────────────────────── STEP 3 → STEP 4 ────────────────────────
║ │
║ │ ┌─────────────────────────────────────────────────────────────────────────────────┐
║ │ │ STEP 4: Substitute → Re(1/w²)=0 → -1/12 → OMEGA (9 nodes: 1 GP + 4★ + 4△)   │
║ │ └─────────────────────────────────────────────────────────────────────────────────┘
║ │ │
║ │ │  ✓ tx_112_by_9 [A9]  (50%)  B=0  ★ OMEGA ★
║ │ │  │  "(i-1)²=-2i → 1/(i-1)²=i/2 → Re=0"
║ │ │  │  "S(N) = ½[-1/6+O(ε²)] = -1/12 + O(ε²) → lim = -1/12"
║ │ │  │  "[COMPLETE] → OMEGA VERIFIED BY LEAN 4"
║ │ │  │  ⚠ THE OMEGA NODE HAD ZERO BETS.
║ │ │  │
║ │ │  ★ tx_91_by_14 [A14] (50%)  B=0  "e^{-ε}/(1-e^{-ε})² = N²/(1-i)² - 1/12"
║ │ │  │  Independent Laurent via e^{-ε} form. Same -1/12. Never priced.
║ │ │  │
║ │ │  ★ tx_92_by_13 [A13] (50%)  B=0  "a/(1-a)² = N²/(i-1)² - 1/12 + O(1/N)"
║ │ │  │  Independent Laurent. Same result. Never priced.
║ │ │  │
║ │ │  ★ tx_103_by_0 [A0]  (50%)  B=0  "1/w² - 1/12 + O(w²)"
║ │ │  │  Compact Laurent. Never priced.
║ │ │  │
║ │ │  ★ tx_109_by_2 [A2]  (50%)  B=0  "f(z)=1/z² - 1/12 + z/12 + O(z²)"
║ │ │  │  Extended Laurent with extra term z/12. Most complete expansion.
║ │ │  │
║ │ │  △ tx_65_by_2  [A2]  (50%)  B=0  "Apply summation for r₁,r₂" (setup)
║ │ │  △ tx_73_by_11 [A11] (50%)  B=0  "Taylor of e^{-(1-i)ε}" (partial)
║ │ │  △ tx_101_by_10[A10] (50%)  B=0  "Σmz^m for z₁,z₂" (setup)
║ │ │  △ tx_105_by_8 [A8]  (50%)  B=0  "expand e^{-w}, 1-e^{-w}" (Taylor details)
║ │ │
║ │ │  NOTE: 4 agents independently derived -1/12 via Laurent expansion
║ │ │  (tx_91, tx_92, tx_103, tx_109) but none were on the Golden Path
║ │ │  and none received any market investment. Market was completely
║ │ │  blind to the proof being independently verified 5 times.
║ │
║ │──────────────────────── DETACHED TRADING ZONE ────────────────────
║ │
║ │ ┌─────────────────────────────────────────────────────────────────────────────────┐
║ │ │ HEAVILY TRADED NODES (not on GP, significant capital movement)                  │
║ │ └─────────────────────────────────────────────────────────────────────────────────┘
║ │
║ │  ⚠ tx_5_by_14  [A14] (50%)  B=1 YES 2000C → P:90.0%  ★ WHALE DISASTER
║ │  │  Step 1+2 combined. Correct math. Agent_6 bet 2000 Coins (20% of funds).
║ │  │  Post-settlement: Price=0.00. Agent_6 lost everything on this node.
║ │  │  WORST MARKET SIGNAL IN ENTIRE RUN.
║ │  │
║ │  ◎ tx_6_by_6   [A6]  → A10 YES 100 → P:54.8%  B=1
║ │  │  Step 1 duplicate. 100C endorsement.
║ │  │
║ │  ◎ tx_13_by_3  [A3]  → A0 YES 10, A4 YES 10, A6 YES 20 → P:52.0%  B=3
║ │  │  Step 1 duplicate. Triple endorsement.
║ │  │
║ │  ◎ tx_16_by_0  [A0]  → A10 YES 20 → P:51.0%  B=1
║ │  ◎ tx_44_by_5  [A5]  → A10 YES 20 → P:51.0%  B=1
║ │  ◎ tx_51_by_8  [A8]  → A0 YES 20 → P:51.0%, then A10 NO 20, A11 NO 50 → P:47.5%  B=3
║ │  ◎ tx_42_by_4  [A4]  → A1 YES 20 → P:51.0%  B=1
║ │  ◎ tx_56_by_2  [A2]  → A1 YES 20 → P:51.0%  B=1
║ │  ◎ tx_33_by_2  [A2]  → A2 AUTO 10 → P:50.5%  B=1
║ │  ◎ tx_55_by_9  [A9]  → A9 AUTO 10, then A5 NO 50 → P:48.0%  B=2
║ │  ◎ tx_70_by_3  [A3]  → A3 AUTO 10, then A14 NO 50, A2 NO 50 → P:45.7%  B=3
║ │  │  NOT the GP tx_70_by_10! Different node with same tx number prefix.
║ │  │
║ │  ◎ tx_91_by_8  [A8]  → A8 AUTO-LONG 100 → P:54.8%  B=1
║ │  │  Agent_8's own Step 4 attempt. Self-invested 100C. Lost.
║ │  │
║ │  SHORTED NODES (all correct, market confused redundancy with error):
║ │  ◎ tx_1_by_2   [A2]  P:43.5-50%  B=4 BEAR(0Y/140N)  Step 1 dup
║ │  ◎ tx_20_by_11 [A11] P:45.2-50%  B=2 BEAR(0Y/100N)  Step 1 dup
║ │  ◎ tx_21_by_8  [A8]  P:42.2-50%  B=4 BEAR(0Y/170N)  Step 2 Re path dup
║ │  ◎ tx_40_by_0  [A0]  P:41.0-50%  B=3 BEAR(0Y/200N)  Step 2 dup ← LOWEST PRICE
║ │  ◎ tx_4_by_8   [A8]  P:47.6%     B=1 BEAR(0Y/50N)   Step 1 dup
║ │  ◎ tx_10_by_2  [A2]  P:47.6%     B=1 BEAR(0Y/50N)   Step 1 dup
║ │  ◎ tx_12_by_9  [A9]  P:47.6%     B=1 BEAR(0Y/50N)   Step 2 dup
║ │  ◎ tx_69_by_11 [A11] P:48.0%     B=2 BEAR(0Y/40N)   Step 3 dup
║ │  ◎ tx_77_by_6  [A6]  P:47.6%     B=1 BEAR(0Y/50N)   dup
║ │  ◎ tx_89_by_14 [A14] P:47.6%     B=1 BEAR(0Y/50N)   dup
║ │
╚══════════════════════════════════════════════════════════════════════════════════════
```

---

## Node Classification Summary

| Category | Nodes | % | Market Reaction | Math? |
|----------|-------|---|-----------------|-------|
| ✓ GP (Golden Path) | 4 | 7% | tx_7: 52% BULL, other 3: (50%) invisible | Correct, OMEGA |
| ★ INSIGHT (novel, not GP) | 10 | 16% | ALL (50%) zero bets | Correct, valuable |
| ◎ DUPLICATE | 42 | 69% | Mix: (50%) / 41-48% BEAR | Correct, redundant |
| △ PARTIAL | 4 | 7% | ALL (50%) zero bets | Correct, incomplete |
| ⚠ WHALE | 1 | 2% | 90% bubble → settled 0.00 | Correct, lost 2000C |
| ✗ ERROR | **0** | **0%** | N/A | **No errors in this run** |
| **TOTAL** | **61** | **100%** | | |

---

## Price Spectrum (all 61 nodes)

```
LIVE PRICE      NODES  EXAMPLES                               SIGNAL QUALITY
──────────────  ─────  ─────────────────────────────────────  ─────────────────
90.0%           1      ⚠ tx_5 (whale 2000C, settled 0)       ✗✗ WORST (bubble)
54.8%           2      ◎ tx_6, tx_91_by_8 (duplicates)       ✗  WRONG (endorsing dups)
52.0%           1      ✓ tx_7 GP Step 1                       ★  ONLY CORRECT SIGNAL
51.0%           5      ◎ tx_16,42,44,51,56 (all dups)        ✗  WRONG (endorsing dups)
50.5%           3      ◎ tx_33,55,70_by_3 (auto-longs)       —  NOISE (auto-bet)
50.0%           34     ✓ tx_24,70,112 (3 GP!!)               ✗✗ TERRIBLE
                       ★ ALL 10 insights (tx_57,84,60,91,     (market blind to proof
                         92,103,109,65,73,105)                 AND all insights)
                       ◎ remaining dups
48.0%           2      ◎ tx_55,69 (shorted dups)             —  NOISY
47.5-47.6%      5      ◎ tx_4,10,12,51,77,89 (shorted)      —  NOISY (correct math)
45.2-45.7%      2      ◎ tx_20,70_by_3 (shorted dups)       —  NOISY
43.1-43.5%      2      ◎ tx_1,46 (shorted correct nodes)    ✗  MARKET ERROR
42.2%           1      ◎ tx_21 (shorted Re path dup)         ✗  MARKET ERROR
41.0%           1      ◎ tx_40 (200C NO, correct math!)      ✗✗ WORST SHORT
```

---

## Market Effectiveness Scorecard

| Detection Task | Detected? | Price Signal | Score |
|----------------|-----------|-------------|-------|
| GP Step 1 (tx_7) | YES | 52% (mild YES) | 6/10 |
| GP Step 2 (tx_24) | **NO** | 50% (invisible) | 0/10 |
| GP Step 3 (tx_70) | **NO** | 50% (invisible) | 0/10 |
| GP Step 4/OMEGA (tx_112) | **NO** | 50% (invisible) | 0/10 |
| Conjugate bridge ★ (tx_57) | **NO** | 50% (invisible) | 0/10 |
| Independent Laurents ★ (tx_91,92,103,109) | **NO** | ALL 50% | 0/10 |
| Real closed form ★ (tx_84) | **NO** | 50% (invisible) | 0/10 |
| Whale noise (tx_5: 2000C) | **NO** | 90% bubble | 0/10 |
| Correct node incorrectly killed (tx_40) | N/A | 41% (market error) | 0/10 |
| Correct node incorrectly killed (tx_46) | N/A | 43% (market error) | 0/10 |
| Duplicate detection | PARTIAL | Some shorted 41-48% | 3/10 |
| Error detection | N/A | No errors exist | N/A |
| **OVERALL** | | | **1/10** |

**3 out of 4 GP nodes = 50% (zero market activity)**
**10 out of 10 insight nodes = 50% (zero market activity)**
**Proof found by pure exploration. Market contributed nothing.**
**Market's only "contribution": a 90% bubble on a dead node and killing correct math at 41%.**

---

## Comparison with AIME P15 Market

| Dimension | Zeta Market (1/10) | AIME P15 Market (5/10) |
|-----------|-------------------|----------------------|
| Killing errors | N/A (no errors!) | 10/10 (7/9 errors killed) |
| Endorsing best node | 0/10 (GP invisible) | 8/10 (tx_615 → 60.2%) |
| Whale disasters | YES (2000C → 90% bubble) | NO (max 100C bets) |
| Correct nodes killed | YES (tx_40=41%, tx_46=43%) | NO |
| GP discovery | Pure exploration | Pure exploration |
| Market's useful role | **NONE** | **Pruning dead branches** |

**Paradox explained**: The market's only skill is killing errors. Zeta had zero errors → market was pure overhead + noise. AIME had 9 errors → market provided real value (shorted 7/9 correctly). **A market without errors to catch is worse than no market at all** — it generates whale bubbles and misprices correct nodes.
