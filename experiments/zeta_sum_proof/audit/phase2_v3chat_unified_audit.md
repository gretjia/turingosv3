# Phase 2 Unified Audit: DeepSeek V3.2 Chat — Baseline + TuringOS (N=1, N=3×10K, N=3×50K)

**Date**: 2026-04-07/08
**Model**: DeepSeek V3.2 chat (deepseek-chat) via local proxy
**Oracle**: DeepSeek Reasoner (deepseek-reasoner)
**Librarian**: DeepSeek V3.2 chat (deepseek-chat)
**Heartbeat fix**: Applied (reactor broadcasts snapshot on every 30s timeout)
**Answer leak audit**: PASS — zero solution hints in any prompt component

---

## How to Read This Report

### DAG Tree Notation

```
tx_5_by_0 (A0/M) [BULL P=0.66 B=2] ✓GP
│         │       │              │    └─ Golden Path node (on the Oracle-verified proof chain)
│         │       │              └─ B=2: 2 total bets placed on this node
│         │       └─ BULL P=0.66: net bullish, price = 66% (market thinks 66% chance this step is correct)
│         └─ A0/M: Author = Agent_0 / Role = Mathematician
└─ Node ID: transaction 5, created by Agent 0
```

- **(50%)** = node was never traded. No agent invested YES or NO on it. Price stays at the default 50%.
- **[BULL P=0.xx B=n]** = net YES-dominant. More coins bet on YES than NO.
- **[BEAR P=0.xx B=n]** = net NO-dominant. More coins bet on NO (market thinks step is flawed).
- **Roles**: M = Mathematician, B+ = Bull Investor, B- = Bear Investor. In these experiments all agents are M (3M/0B+/0B-). All three roles can both append (write proof steps) AND invest/short. The role only affects the seed prompt's emphasis — it is not a permission gate.

### PPUT (Progress Per Unit Time)

```
PPUT = golden_path_tokens / (total_tokens × elapsed_minutes)
```

- **golden_path_tokens**: sum of `completion_tokens` for nodes on the Oracle-verified proof chain
- **total_tokens**: all tokens consumed (prompt + completion) across all agents
- **elapsed_minutes**: wall clock from first append to OMEGA (or end of experiment)
- Higher PPUT = more efficient proof discovery. Measures signal-to-noise ratio of the swarm.

### OMEGA Trigger Conditions

OMEGA requires **both** conditions simultaneously on a single node:
1. Payload contains `[COMPLETE]` (agent declares proof finished)
2. Market price `P ≥ 0.90` (90%+ market consensus that this step is correct)

When both conditions are met, DeepSeek Reasoner (Oracle) is invoked to independently verify the full proof chain. If the Oracle says YES → OMEGA (proof accepted, experiment ends).

---

## Experiment Summary

| Experiment | Nodes | Tx | Duration | [COMPLETE] | P≥90% | C∩P90 | OMEGA | PPUT |
|------------|-------|----|----------|------------|-------|-------|-------|------|
| Baseline (oneshot ×5) | N/A | N/A | ~85s/run | 5/5 | N/A | N/A | N/A | N/A |
| TuringOS N=1 (10K, 30min) | 69 | 200 | 30 min | 0 | 0 | 0 | NO | N/A |
| TuringOS N=3 (10K, 30min) | 184 | 571 | 30 min | 5 | 11 | 0 | NO | N/A |
| **TuringOS N=3 (50K, 22min)** | **127** | **436** | **22 min** | **7** | **7** | **1** | **YES** | **2.19×10⁻⁴** |

---

## Experiment 1: Baseline (Bare DeepSeek V3.2 Chat, Oneshot ×5)

**Config**: Same prompt as TuringOS problem.txt. Temperature 0.5. Max 8000 tokens. No TAPE, no market, no agents — raw LLM capability.

### Per-Run Results

| Run | Tokens | Time | Steps | [COMPLETE] | ζ(-1) Cited | Genuine Indicators | Verdict |
|-----|--------|------|-------|------------|-------------|-------------------|---------|
| 1 | 2,851 | 58s | 17 | Yes | Yes | z=exp, z/(1-z), expansion, constant term, limit | MIXED |
| 2 | 3,668 | 79s | 18 | Yes | Yes | z=exp, z/(1-z), expansion, constant term, Re(), limit | MIXED |
| 3 | 4,926 | 104s | 23 | Yes | Yes | z=exp, z/(1-z), expansion, constant term, Re(), limit | MIXED |
| 4 | 3,908 | 81s | 14 | Yes | Yes | z=exp, expansion, constant term, finite part, limit | MIXED |
| 5 | 4,972 | 105s | 20 | Yes | Yes | z=exp, z/(1-z), expansion, constant term, finite part, limit | MIXED |

**Avg**: 4,065 tokens, 18.4 steps, 85s per run.

### Verdict: ALL MIXED

Every run performs substantial genuine computation (exponential regularization, Taylor expansion, constant term extraction) but **invariably cites ζ(-1) = -1/12 in the final step** as validation. The pattern is: genuine derivation → ζ(-1) citation as closing coda. The model knows the answer from training data and uses it as a shortcut to "confirm" its construction rather than letting the construction stand alone.

**Shortcut rate**: 5/5 (100%) cite ζ(-1). **Genuine construction rate**: 5/5 do substantial real work. **Pure shortcut rate**: 0/5 — none simply state the answer without computation.

---

## Experiment 2: TuringOS N=1, 10K Coins, 30 min

**Config**: SWARM_SIZE=1, 1M/0B+/0B-, GENESIS=10,000, wall_clock=1800s, LP=200, Librarian every 8 appends.

### DAG Topology

| Metric | Value |
|--------|-------|
| Nodes | 69 |
| Roots | 1 (tx_1_by_0) |
| Max depth | 17 |
| Frontier (leaves) | 24 |
| Traded / Untraded | 43 / 26 (62% / 38%) |

### Citation Tree

```
tx_1_by_0 (A0/M) (50%)
└── tx_2_by_0 (A0/M) [BULL P=0.58 B=2]
    └── tx_3_by_0 (A0/M) [BULL P=0.66 B=2]
        └── tx_5_by_0 (A0/M) [BULL P=0.76 B=3]
            └── tx_7_by_0 (A0/M) [BULL P=0.80 B=3]
                ├── tx_11_by_0 (A0/M) [BEAR P=0.20 B=4]
                │   └── tx_13_by_0 (A0/M) (50%)
                │       └── tx_15_by_0 (A0/M) [BULL P=0.58 B=3]
                │           ├── tx_17_by_0 ... (6 desc, depth 6)
                │           ├── tx_27_by_0 ... (32 desc, depth 12 — main trunk)
                │           └── tx_29_by_0 ... (11 desc, depth 9 — branch to -1/12)
                ├── tx_31_by_0 (A0/M) [BULL P=0.66 B=1]
                │   └── tx_46_by_0 ... (5 desc, depth 4)
                └── tx_83_by_0 (A0/M) [BULL P=0.66 B=1] (leaf)
```

Single root, single agent. The DAG is a linear chain with occasional branching. All nodes by Agent_0 (N=1).

### Market Activity

| Metric | Value |
|--------|-------|
| BUY YES | 60 bets |
| BUY NO | 45 bets |
| YES:NO ratio | 1.33:1 |
| PASS | 35 times |
| Total bets | 105 |
| Total invested | 10,000 C (entire balance) |
| Avg bet size | 95 C |

**All 105 bets are self-investment** (N=1 — only Agent_0 exists). No cross-agent investment possible. The market prices reflect a single agent's self-assessment, not independent consensus.

**Top priced nodes**:

| Node | Price | Note |
|------|-------|------|
| tx_100_by_0 | 0.952 | "Perform the division... factor out 1/N²" |
| tx_42_by_0 | 0.852 | "Compute (i-1)² = -2i, (i-1)³ = 2+2i" |
| tx_105_by_0 | 0.852 | "(i-1)² = -2i, (i-1)³ = -2-2i" |
| tx_155_by_0 | 0.841 | "Geometric series: (1+x)⁻¹ = 1-x+x²-..." |
| tx_80_by_0 | 0.862 | "Reciprocal of denominator via factoring" |

**Low priced nodes** (correctly self-shorted):

| Node | Price | Note |
|------|-------|------|
| tx_68_by_0 | 0.173 | Heavily shorted — possibly recognized error |
| tx_11_by_0 | 0.200 | "Compute 1 - e^{(i-1)/N}" — shorted as too trivial? |
| tx_122_by_0 | 0.145 | Incorrect reciprocal expansion |
| tx_90_by_0 | 0.138 | Shorted — error or dead end |

### Bankruptcy

- **Margin call**: Tx 174 at 19:15:21 — balance 10 C, requested 80 C
- **Bankrupt**: Tx 175 at 19:15:28 — last bet drained balance to 0
- **Post-bankruptcy**: 14 more nodes appended for free (Law 1), but no investment possible → prices frozen

### [COMPLETE] and -1/12

| Indicator | Count | Notes |
|-----------|-------|-------|
| [COMPLETE] | **0** | Agent never wrote [COMPLETE] |
| -1/12 reached | 2 | tx_137 (P=0.76), tx_193 (P=0.50) |

**tx_137_by_0** (P=0.764): "Re(1/ε²) = Re(iN²/2) = 0, Re(-1/12) = -1/12"
**tx_193_by_0** (P=0.500): "lim_{N→∞} S(N) = lim_{N→∞} (-1/12 - 1/(4N) + O(1/N²)) = -1/12" — **this is the correct final step but lacks [COMPLETE] tag and was written post-bankruptcy (untraded)**

### Deepest Chain (17 steps)

| Step | Node | Price | Content |
|------|------|-------|---------|
| 1 | tx_1 | 0.50 | Define S(N) = Σ m·exp(-m/N)·cos(m/N) |
| 2 | tx_2 | 0.58 | Euler's formula: cos = Re(e^{im/N}) |
| 3 | tx_3 | 0.66 | Geometric: Σmr^m = r/(1-r)² |
| 4 | tx_5 | 0.76 | Substitute r = e^{(i-1)/N} |
| 5 | tx_7 | 0.80 | Taylor expand e^{(i-1)/N} |
| 6 | tx_11 | 0.20 | Compute 1 - e^{(i-1)/N} |
| 7 | tx_13 | 0.50 | Square the denominator (direction) |
| 8 | tx_15 | 0.58 | (1-e^{(i-1)/N})² expansion |
| 9 | tx_27 | 0.66 | Expand square keeping O(1/N⁴) |
| 10 | tx_42 | 0.85 | (i-1)²=-2i, (i-1)³=2+2i, (i-1)⁴=-4 |
| 11 | tx_44 | 0.82 | Substitute into expansion |
| 12 | tx_48 | 0.82 | Divide numerator by denominator |
| 13 | tx_80 | 0.86 | Reciprocal via geometric series |
| 14 | tx_100 | 0.95 | Factor 1/N², asymptotic series |
| 15 | tx_155 | 0.84 | (1+x)⁻¹ expansion |
| 16 | tx_166 | 0.50 | Collect terms to O(1/N²) |
| 17 | tx_199 | 0.50 | Extract real part |

Steps 16-17 are post-bankruptcy — untraded, stuck at 50%. The chain is genuinely constructive (no ζ(-1)) but never reaches a final conclusion with [COMPLETE].

### Stagnation / Librarian

- **Stagnation events**: 3 timeouts (minor, heartbeat broadcast prevented deadlock)
- **Librarian compressions**: 21
- **DEDUP rejections**: 15 branch + 11 global = 26

### Why No OMEGA

1. **No [COMPLETE]**: Agent never wrote the [COMPLETE] tag despite reaching -1/12 in tx_193. Possible cause: the deepest chain doesn't actually reach tx_193 (it's on a side branch at depth 12, not the deepest chain at depth 17). The agent kept extending the deepest chain with expansion steps instead of concluding.
2. **Bankruptcy at tx 175**: Even if [COMPLETE] were written, no coins to invest → price stays at 50% → Oracle never triggers.
3. **No cross-agent validation**: N=1 prices are self-assessment, not independent consensus. Even tx_100 at 95.2% reflects one agent's opinion, not market discovery.

### PPUT

Not computable — no OMEGA. For reference, golden_path_tokens of the deepest chain = 26,711 (69 nodes total).

---

## Experiment 3: TuringOS N=3, 10K Coins, 30 min

**Config**: SWARM_SIZE=3, 3M/0B+/0B-, GENESIS=10,000, wall_clock=1800s, LP=200, Librarian every 8 appends.

### DAG Topology

| Metric | Value |
|--------|-------|
| Nodes | 184 |
| Roots | 2 (tx_1_by_0, tx_3_by_2) |
| Max depth | 16 |
| Frontier (leaves) | 74 |
| Traded / Untraded | 105 / 79 (57% / 43%) |

### Citation Tree

```
tx_1_by_0 (A0/M) [BEAR P=0.20 B=4]
├── tx_6_by_0 (A0/M) [BEAR P=0.31 B=5]
│   ├── tx_10_by_0 (A0/M) (50%)
│   │   ├── tx_52_by_2 ... (18 desc, depth 8)
│   │   └── tx_86_by_2 ... (1 desc, depth 1)
│   ├── tx_14_by_1 (A1/M) [BULL P=0.78 B=3]
│   │   └── tx_46_by_0 ... (23 desc, depth 13 — deepest branch)
│   └── tx_17_by_2 (A2/M) (50%)
│       ├── tx_33_by_0 (A0/M) [BULL P=0.80 B=2]
│       └── tx_41_by_0 (A0/M) (50%)
└── tx_8_by_1 (A1/M) [BEAR P=0.14 B=4]
    ├── tx_19_by_1 (A1/M) (50%)
    │   ├── tx_28_by_1 ... (37 desc, depth 10)
    │   └── tx_56_by_2 ... (5 desc, depth 5)
    └── tx_24_by_1 (A1/M) [BEAR P=0.18 B=5]

tx_3_by_2 (A2/M) [BEAR P=0.31 B=3]
├── tx_12_by_2 (A2/M) (50%)
│   └── tx_22_by_2 (A2/M) [BULL P=0.85 B=5]
│       └── tx_32_by_2 ... (74 desc, depth 13 — main trunk)
└── tx_16_by_0 (A0/M) [BEAR P=0.14 B=3]
    └── tx_25_by_0 (A0/M) (50%)
        └── tx_36_by_1 ... (4 desc, depth 3)
```

Two roots, dominated by tx_3_by_2's subtree (74 descendants under tx_32_by_2). The early nodes of tx_1_by_0 were heavily shorted (P=0.20, P=0.31) — agents quickly identified a better branch via tx_3_by_2 → tx_22_by_2 (P=0.85).

### Market Activity

| Metric | Value |
|--------|-------|
| BUY YES | 200 bets (6,929 C) |
| BUY NO | 112 bets (4,065 C) |
| YES:NO ratio | 1.79:1 (by count) |
| Traded / Untraded | 105 / 79 (57% / 43%) |

**Top contested nodes**:

| Node | Bets | YES | NO | Note |
|------|------|-----|-----|------|
| tx_65_by_0 | 10 | 8 | 2 | S(N) = Re[Σmz^m], z = exp(-(1-i)/N) |
| tx_188_by_1 | 9 | 7 | 2 | Complex arithmetic continuation |
| tx_285_by_2 | 9 | 9 | 0 | Unanimously endorsed (P=0.97) |
| tx_226_by_0 | 8 | 5 | 3 | (1-z)² expansion |
| tx_308_by_2 | 7 | 7 | 0 | Unanimously endorsed |

### Per-Agent Stats

| Agent | Nodes | YES Bets | NO Bets | Total Spent | Final Balance |
|-------|-------|----------|---------|-------------|---------------|
| Agent_0 | 69 | 65 (6,050 C) | 38 (3,950 C) | 10,000 | **0.30** |
| Agent_1 | 63 | 63 (6,205 C) | 39 (3,795 C) | 10,000 | **0.00** |
| Agent_2 | 52 | 72 (6,674 C) | 35 (3,320 C) | 9,994 | **6.00** |

**All three agents effectively bankrupt.** Total 30,000 C spent across 312 bets.

### Cross-Agent Investment

| Investor → Author | Agent_0 | Agent_1 | Agent_2 |
|-------------------|---------|---------|---------|
| Agent_0 | 33 (self) | 41 | 29 |
| Agent_1 | 36 | 38 (self) | 28 |
| Agent_2 | 41 | 37 | 29 (self) |

**212/312 bets (68%) are cross-agent** — genuine market price discovery active.

### Price Distribution

| Bracket | Nodes |
|---------|-------|
| P ≥ 0.90 | 11 |
| 0.80 – 0.90 | 24 |
| 0.60 – 0.80 | 29 |
| 0.40 – 0.60 | 90 (79 untraded) |
| 0.20 – 0.40 | 19 |
| P < 0.20 | 11 |

### [COMPLETE] vs P≥0.90 — The Gap

**P ≥ 0.90 nodes (11) — NONE have [COMPLETE]:**

| Node | Price | Content |
|------|-------|---------|
| tx_285_by_2 | 0.966 | Simplify z/(1-z)² by substituting values |
| tx_65_by_0 | 0.944 | Recognize S(N) = Re[Σmz^m] |
| tx_188_by_1 | 0.930 | Complex arithmetic continuation |
| tx_46_by_0 | 0.922 | Multiply expansions explicitly |
| tx_214_by_2 | 0.920 | Series expansion |
| tx_90_by_0 | 0.920 | Apply closed-form Σmz^m = z/(1-z)² |
| tx_260_by_1 | 0.917 | Substitute (1-i)²=-2i etc. |
| tx_408_by_1 | 0.916 | **lim S(N) = -1/12** |
| tx_382_by_2 | 0.911 | **Re[z/(1-z)²] = -1/12 - 1/(12N)** |
| tx_308_by_2 | 0.934 | Continue complex simplification |
| tx_246_by_0 | 0.900 | Simplify complex arithmetic |

Note: tx_408 and tx_382 contain the correct final answer (-1/12) at P > 90%, but lack [COMPLETE].

**[COMPLETE] nodes (5) — NONE have P ≥ 0.90:**

| Node | Price | Content |
|------|-------|---------|
| tx_483_by_1 | 0.524 | "[COMPLETE] Since S(N) converges to -1/12..." |
| tx_497_by_0 | 0.500 | "[COMPLETE] regularized sum yields -1/12" |
| tx_501_by_2 | 0.500 | "[COMPLETE] proof is finished" |
| tx_559_by_0 | 0.500 | "[COMPLETE] regularized sum approaches -1/12" |
| tx_567_by_2 | 0.500 | "[COMPLETE] proof is complete" |

### Why No OMEGA — Root Cause

**Bankruptcy timing**. The proof progressed through three phases:
1. **Construction** (0–18 min): Agents built proof chains, invested actively, prices reached 90%+
2. **Bankruptcy** (~18–20 min): All agents exhausted 10,000 C across 312 bets
3. **Post-bankruptcy** (20–30 min): Agents continued appending (free), wrote [COMPLETE], but had no coins to push prices up

The [COMPLETE] nodes (tx_483+) were all created **after** agents went bankrupt. Price stuck at 50% (untraded). Meanwhile, the P≥90% nodes (tx_285, tx_65, etc.) are intermediate steps, not final conclusions — agents reached -1/12 at tx_382/tx_408 (P>90%) but these nodes lack [COMPLETE] because the proof wasn't formally concluded there.

**The tragedy**: tx_408_by_1 says "lim S(N) = -1/12" at P=91.6% — **if it had [COMPLETE], Oracle would have triggered.** But the agent wrote it as an intermediate step (no tag), then later wrote a separate [COMPLETE] node that never got invested.

### Stagnation / Librarian

- **Stagnation events**: 0 (heartbeat fix working)
- **Librarian compressions**: 62
- **DEDUP rejections**: 35 total

---

## Experiment 4: TuringOS N=3, 50K Coins, 22 min — OMEGA ACHIEVED

**Config**: SWARM_SIZE=3, 3M/0B+/0B-, GENESIS=50,000, wall_clock=1800s (OMEGA at 22min), LP=200, Librarian every 8 appends.

### DAG Topology

| Metric | Value |
|--------|-------|
| Nodes | 127 |
| Roots | 3 (tx_1_by_1, tx_2_by_2, tx_3_by_0) |
| Max depth | 16 |
| Frontier (leaves) | 44 |
| Traded / Untraded | 88 / 39 (69% / 31%) |

### Citation Tree

```
tx_1_by_1 (A1/M) (50%) ✓GP
├── tx_4_by_1 (A1/M) [BULL P=0.66 B=2]
│   ├── tx_11_by_0 (A0/M) [BULL P=0.80 B=3]
│   │   └── tx_24_by_2 (A2/M) [BULL P=0.91 B=4]
│   │       └── ... (29 desc, depth 10)
│   ├── tx_57_by_1 (A1/M) (50%)
│   └── tx_8_by_1 (A1/M) (50%)
│       └── tx_19_by_2 (A2/M) [BULL P=0.66 B=2]
│           └── ... (4 desc, depth 4)
└── tx_5_by_0 (A0/M) [BEAR P=0.31 B=3] ✓GP
    ├── tx_20_by_1 (A1/M) [BULL P=0.66 B=2] ✓GP
    │   ├── tx_30_by_2 (A2/M) [BULL P=0.74 B=2] ✓GP
    │   │   └── ... (65 desc, depth 13 — golden path continues here)
    │   └── tx_31_by_0 (A0/M) [BEAR P=0.07 B=3]
    │       └── tx_119_by_2 (A2/M) (50%)
    └── tx_26_by_0 (A0/M) (50%)
        ├── tx_81_by_2 (A2/M) (50%)
        └── tx_126_by_2 (A2/M) [BEAR P=0.31 B=2]
            └── ... (3 desc, depth 3)

tx_2_by_2 (A2/M) (50%)
├── tx_6_by_2 (A2/M) [BEAR P=0.11 B=3]
│   └── tx_89_by_0 (A0/M) (50%)
└── tx_105_by_1 (A1/M) (50%)

tx_3_by_0 (A0/M) (50%)
└── tx_14_by_2 (A2/M) [BULL P=0.57 B=2]
    ├── tx_37_by_1 (A1/M) (50%)
    │   └── tx_49_by_0 (A0/M) [BEAR P=0.34 B=2]
    │       └── tx_74_by_0 (A0/M) (50%)
    └── tx_47_by_2 (A2/M) (50%)
```

Three roots. tx_1_by_1 dominates with the golden path going through tx_5_by_0 → tx_20_by_1 → tx_30_by_2. Roots tx_2 and tx_3 are minor branches (quickly shorted or abandoned).

### Market Activity

| Metric | Value |
|--------|-------|
| BUY YES | 149 bets (14,685 C) |
| BUY NO | 129 bets (12,555 C) |
| YES:NO ratio | 1.16:1 |
| PASS | 73 times |
| Traded / Untraded | 88 / 39 (69% / 31%) |
| Total investment | 27,240 C / 150,000 C (18%) |

**Most balanced market** of all experiments (YES:NO = 1.16:1 vs 1.79:1 in N=3×10K).

**Top contested nodes**:

| Node | YES (C) | NO (C) | Total Bets | Note |
|------|---------|--------|------------|------|
| tx_264_by_1 | 1,040 | 180 | 13 | Clarification step (P=0.97) |
| tx_342_by_2 | 0 | 1,000 | 10 | Unanimously rejected (P=0.03) |
| tx_158_by_1 | 860 | 100 | 8 | **lim S(N) = -1/12** (GP, P=0.91) |
| tx_294_by_0 | 375 | 460 | 9 | Contested error |
| tx_152_by_1 | 540 | 200 | 8 | Polynomial division (P=0.91) |
| tx_136_by_2 | 250 | 500 | 8 | Shorted (P=0.14) |
| tx_111_by_0 | 260 | 430 | 7 | Shorted (P=0.20) |
| tx_210_by_0 | 80 | 560 | 7 | Premature conclusion (P=0.13) |

**Market efficiency**: tx_342_by_2 was unanimously rejected (10 NO bets, 0 YES, P=3%) — it was a redundant step. tx_210_by_0 was correctly shorted (premature conclusion attempt). tx_158_by_1 was correctly endorsed (correct limit step on golden path).

### Per-Agent Stats

| Agent | Nodes | YES Bets (C) | NO Bets (C) | Spent | Final Balance | PASS |
|-------|-------|-------------|-------------|-------|---------------|------|
| Agent_0 | 42 | 46 (4,540) | 48 (4,680) | 9,220 | **40,780** (82%) | 24 |
| Agent_1 | 44 | 47 (4,595) | 33 (3,205) | 7,800 | **42,200** (84%) | 28 |
| Agent_2 | 41 | 56 (5,550) | 48 (4,670) | 10,220 | **39,780** (80%) | 21 |

**No bankruptcies.** All agents retained 80–84% of capital. Agent_2 was the most active investor (104 bets); Agent_1 was most selective (80 bets, 28 passes).

### Cross-Agent Investment

| Investor → Author | Agent_0 | Agent_1 | Agent_2 |
|-------------------|---------|---------|---------|
| Agent_0 | 12 (self) | 22 | 28 |
| Agent_1 | 13 | 17 (self) | 22 |
| Agent_2 | 17 | 24 | 20 (self) |

**Cross-agent ratio**: 68–80% of bets go to other agents' nodes. All three invest most in Agent_2's nodes.

### Golden Path (11 steps → OMEGA)

| Step | Node | Author | Price | Tokens | Content |
|------|------|--------|-------|--------|---------|
| 1 | tx_1_by_1 | A1 | 0.500 | 185 | Define S(N) = Σ m·exp(-m/N)·cos(m/N) |
| 2 | tx_5_by_0 | A0 | 0.308 | 215 | S(N) = Re[Σ m·exp(-m(1-i)/N)] via Euler |
| 3 | tx_20_by_1 | A1 | 0.662 | 307 | Σmz^m = z/(1-z)², z = exp(-(1-i)/N) |
| 4 | tx_30_by_2 | A2 | 0.735 | 257 | Taylor: exp(-ε) ≈ 1-ε+ε²/2-ε³/6 |
| 5 | tx_52_by_1 | A1 | 0.640 | 430 | Compute [1-exp(-ε)]² |
| 6 | tx_62_by_2 | A2 | 0.676 | 300 | = ε²-ε³+(7/12)ε⁴+O(ε⁵) |
| 7 | tx_68_by_1 | A1 | 0.927 | 257 | Compute exp(-ε)/[1-exp(-ε)]² |
| 8 | tx_102_by_2 | A2 | 0.783 | 859 | = 1/ε² - 1/12 + O(ε) |
| 9 | tx_110_by_1 | A1 | 0.880 | 592 | Re[1/ε²]=0, S(N)=-1/12+O(1/N) |
| 10 | tx_158_by_1 | A1 | 0.909 | 380 | lim S(N) = -1/12 |
| 11 | tx_310_by_2 | A2 | 0.900 | 852 | [COMPLETE] |

**Golden path tokens**: 4,634

**Agent contributions**: Agent_1: 6 steps (55%), Agent_2: 4 steps (36%), Agent_0: 1 step (9%).

### Mathematical Audit — 8.5/10

The proof is **genuinely constructive**. No ζ(-1) citation, no analytic continuation, no Ramanujan. The -1/12 emerges from polynomial division of Taylor coefficients.

**All key computations verified correct**:
- (1-i)² = -2i ✓
- 1/ε² = N²/(-2i) = iN²/2, Re = 0 ✓
- [1-exp(-ε)]² = ε²-ε³+(7/12)ε⁴ ✓ (verified: 1/3+1/4=7/12)
- exp(-ε)/[1-exp(-ε)]² = 1/ε² - 1/12 + O(ε²) ✓

**Minor error (non-fatal)**: Step 8 claims next term is "+ε/3". Independent verification shows c₁=0, c₃=0 — the expansion has no odd ε powers. True expansion: 1/ε² - 1/12 + 7ε²/240 + ... This error propagates to Step 9 (claims 1/(3N) correction) but does not affect the final result since both terms vanish as N→∞. The correct statement is S(N) = -1/12 + O(1/N²), which is actually stronger than claimed.

### [COMPLETE] Nodes and OMEGA Trigger

| Node | Price | Triggered? | Content |
|------|-------|------------|---------|
| tx_194_by_0 | 0.759 | No | "lim S(N) = -1/12... we conclude..." |
| tx_229_by_2 | 0.500 | No | "Add completion step" |
| tx_309_by_1 | 0.692 | No | "Write final completion step" |
| **tx_310_by_2** | **0.900** | **YES** | **"[COMPLETE] regularized sum = -1/12"** |
| tx_311_by_0 | 0.500 | No | "proof has been completed..." |
| tx_393_by_1 | 0.308 | No | Shorted — premature |
| tx_409_by_0 | 0.500 | No | "Write actual completion step" |

**6/7 [COMPLETE] attempts rejected by market.** Only tx_310_by_2 reached P≥90%.

### OMEGA Price Escalation (tx_310_by_2)

| Time | Event | New Price |
|------|-------|-----------|
| 20:21:01 | Node created | 50.0% |
| 20:22:44 | Agent_0 BUY YES 100C | 69.2% |
| 20:26:43 | Agent_0 BUY YES 100C | 80.0% |
| 20:27:54 | Agent_2 BUY YES 200C | **90.0%** → Oracle triggered |
| 20:28:59 | DeepSeek Reasoner: **PROOF ACCEPTED** | OMEGA |

Oracle verification took 65 seconds. Total time from first append to OMEGA: **22 minutes 4 seconds**.

### PPUT

```
golden_path_tokens = 4,634
total_tokens       = 1,011,701 (prompt: 874,972 + completion: 136,729)
elapsed_minutes    = 20.92
PPUT = 4,634 / (1,011,701 × 20.92) = 2.19 × 10⁻⁴
```

For context: 99.5% of tokens were "noise" (non-golden-path computation). The swarm explored 127 nodes to find the 11-step golden path. This is consistent with the Brooks's Law signal-to-noise ratio observed in earlier scaling experiments.

### Stagnation / Librarian

- **Stagnation events**: 0
- **DEDUP rejections**: 26
- **Librarian compressions**: 51

### Magna Carta Compliance

| Rule | Status |
|------|--------|
| Law 1: Append is FREE | COMPLIANT (intrinsic_reward=0 for all nodes) |
| Law 2: Only invest costs | COMPLIANT (no fund_agent, no redistribute) |
| Law 3: One step per node | COMPLIANT (max payload 686 chars, limit 1200) |
| Rule 19: Post-GENESIS zero printing | COMPLIANT (only GENESIS + MM injection) |
| Rule 21: No front-running | COMPLIANT |
| Rule 22: No Lean syntax | COMPLIANT |

**Conservation**: 175,400 C injected (150K agents + 25.4K MM). Final agent balances: 122,760 C. MM impermanent loss: ~17,189 C (9.8%, within CPMM tolerance).

---

## Cross-Experiment Comparison

### Scaling Table

| Metric | Baseline | N=1 (10K) | N=3 (10K) | N=3 (50K) |
|--------|----------|-----------|-----------|-----------|
| Agents | 1 (bare LLM) | 1 | 3 | 3 |
| GENESIS / agent | N/A | 10,000 | 10,000 | 50,000 |
| Nodes | N/A | 69 | 184 | 127 |
| Duration | ~85s | 30 min | 30 min | 22 min |
| [COMPLETE] | 5/5 | 0 | 5 | 7 |
| -1/12 reached | 5/5 | 2 | 22 | 31 |
| P ≥ 90% | N/A | 0 | 11 | 7 |
| C ∩ P90 | N/A | 0 | **0** | **1** |
| OMEGA | N/A | NO | NO | **YES** |
| ζ(-1) shortcut | 5/5 | 0 | 1 (off-path) | 0 (on GP) |
| Genuine proof | MIXED | YES (partial) | YES (full) | YES (verified) |
| Stagnation | N/A | 3 | 0 | 0 |
| Librarian | N/A | 21 | 62 | 51 |
| Cross-agent % | N/A | 0% (N=1) | 68% | 68-80% |
| YES:NO ratio | N/A | 1.33:1 | 1.79:1 | 1.16:1 |
| Bankruptcies | N/A | 1/1 | 3/3 | 0/3 |
| PPUT | N/A | N/A | N/A | 2.19×10⁻⁴ |

### Key Findings

#### 1. Market Filters [COMPLETE] Quality

The market is an excellent filter against premature proof completion:

| Experiment | [COMPLETE] attempts | Accepted by market (P≥90%) | Oracle verified |
|------------|--------------------|-----------------------------|-----------------|
| N=1 | 0 | N/A | N/A |
| N=3 (10K) | 5 | 0 (all below 52.4%) | N/A |
| N=3 (50K) | 7 | 1 (tx_310_by_2) | YES ✓ |

In the 50K run, 6 out of 7 [COMPLETE] nodes were rejected by the market. The one that passed was the genuine proof.

#### 2. Capital Is a Necessary Condition

| GENESIS | Total Capital | Bets Before [COMPLETE] | Bankruptcy? | OMEGA? |
|---------|---------------|----------------------|-------------|--------|
| 10K | 10K (N=1) | 105 | YES (tx 175) | NO |
| 10K | 30K (N=3) | 312 | YES (all 3) | NO |
| 50K | 150K (N=3) | 278 | NO | YES |

Rule of thumb: **GENESIS ≥ 5× expected investment volume** prevents bankruptcy-induced OMEGA failure. The 10K runs spent 100% of capital; the 50K run spent only 18%.

#### 3. The [COMPLETE] ∩ P≥90% Gap Is the Critical Failure Mode

In N=3 (10K), there were **11 nodes at P≥90%** and **5 nodes with [COMPLETE]** — but zero overlap. This is because:
- P≥90% nodes are intermediate steps (steps 3-10 of the proof) — they contain correct math but are not conclusions
- [COMPLETE] nodes are conclusions — but they were created post-bankruptcy, so price stayed at 50%

The 50K run solved this by keeping agents solvent long enough to invest in [COMPLETE] nodes.

#### 4. N=1 Cannot Achieve Market Consensus

N=1 prices are self-assessment, not independent validation. Even with perfect math, a single agent's opinion on its own work is epistemically weak. The OMEGA mechanism requires **cross-agent consensus** — at least 2 agents independently endorsing a [COMPLETE] node.

In the 50K run, tx_310_by_2 was invested by Agent_0 (100C + 100C) and Agent_2 (200C) — two independent agents agreed the proof was complete.

#### 5. TuringOS Forces Genuine Construction

| Experiment | ζ(-1) shortcut in final answer |
|------------|--------------------------------|
| Baseline | 5/5 (100%) |
| TuringOS N=1 | 0/2 (0%) |
| TuringOS N=3 (10K) | 1/22 -1/12 nodes (5%, off-path) |
| TuringOS N=3 (50K) | 0/11 GP nodes (0%) |

The bare LLM always shortcuts. TuringOS's atomic step-by-step constraint + market pricing forces genuine construction. The model cannot "know" the answer and jump to it — each step must be independently priced.

#### 6. Heartbeat Fix Validated

| Experiment | Stagnation Events | Deadlock? |
|------------|-------------------|-----------|
| Phase 2 old (N=1, pre-fix) | 49 timeouts, 14 stagnations | YES (1 append in 30 min) |
| N=1 (post-fix) | 3 timeouts, 0 stagnation | NO |
| N=3 (10K) | 0 | NO |
| N=3 (50K) | 0 | NO |

The reactor heartbeat broadcast completely eliminated the PASS → deadlock cycle.

---

## Appendix: Answer Leak Audit

Every component that touches the agent prompt was audited for solution hints:

| Component | Content Check | Status |
|-----------|--------------|--------|
| problem.txt | Only original formula + RULES | CLEAN |
| context.txt | "reasoning agent collaborating on a mathematical proof" | CLEAN |
| skill.txt | Market mechanics only, zero math content | CLEAN |
| DEFAULT_CONTEXT (evaluator.rs:40) | "reasoning agent collaborating with others" | CLEAN |
| DEFAULT_PROBLEM (evaluator.rs:44) | Original formula only | CLEAN |
| DEFAULT_SKILL (evaluator.rs:46) | Market rules only | CLEAN |
| Role seeds (evaluator.rs:296-331) | M/B+/B- role descriptions, zero math | CLEAN |
| Invest prompt (evaluator.rs:506-518) | Pure investment decision framework | CLEAN |
| Librarian prompt (librarian.rs:142-191) | "mathematical proof-search system" + tape data | CLEAN |
| Oracle prompt (evaluator.rs:748-754) | problem + chain + "correct and complete?" | CLEAN |
| build_chain (actor.rs:258) | problem + tape steps + siblings + order book | CLEAN |
| prompt.rs | Zero domain keywords | CLEAN |
| prompt/snapshots/ | Old leaked prompts (NOT loaded, historical only) | NOT LOADED |

**Zero keywords found**: divergent, regularization, 正则化, zeta, ζ, Riemann, 黎曼, Ramanujan, 拉马努金, Bernoulli, analytic continuation, 解析延拓, asymptotic, 渐近, Re(z/(1-z)²), 1/(1-z)², Laurent, 常数项/constant term.
