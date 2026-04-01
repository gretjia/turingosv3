# Zeta ζ-Sum — Run 4: 30 Agents Role Trifecta (Pro/Qwen2.5-7B-Instruct)

**82 nodes | 300 tx | 50 traded | 32 untraded | NOT proved**
**Model**: Pro/Qwen2.5-7B-Instruct (SiliconFlow) | **Roles**: 10M/10B+/10B-
**BUY YES**: 170 | **BUY NO**: 43 | **Ratio**: 4.0:1
**Duration**: ~1 min | **Generation**: 1 | **Max depth**: 9

> **Experiment 2026-04-01**: Scale from 15→30 agents (10/10/10). Same weak model.
> Tests whether doubling agents improves market dynamics or just adds noise.

```
● = P=1 (high price)   ○ = P=0 (low price)
M = Mathematician(0-9)  B+ = Bull(10-19)  B- = Bear(20-29)
[BULL xY B=n] = YES-dominant   [BEAR xN B=n] = NO-dominant
(50%) = never traded   ⚠WHALE = position >500C
```

## Citation Tree (11 roots)

```
ROOT (82 nodes, 50 traded, 32 untraded)
├── tx_1_by_16 (A16/B+) [BULL 1575Y/102N B=5 ⚠WHALE] ● 
│   └── tx_14_by_16 (A16/B+) [BULL 28529Y/9120N B=25 ⚠WHALE] ● 
│       └── tx_47_by_22 (A22/B-) [BULL 11012Y/3994N B=7 ⚠WHALE] ● 
│           └── tx_105_by_4 (A4/M) [BULL 135Y B=2] ● 
│               └── tx_197_by_3 (A3/M) [BULL 172Y B=3] ● 
│                   └── tx_210_by_29 (A29/B-) (50%) ○ 
│                       └── tx_227_by_12 (A12/B+) (50%) ○ 
│                           └── tx_239_by_8 (A8/M) (50%) ○ 
│                               └── tx_262_by_3 (A3/M) (50%) ○ 
│                                   └── tx_298_by_17 (A17/B+) [BULL 2Y B=1] ● 
├── tx_2_by_20 (A20/B-) [BULL 1500Y B=1 ⚠WHALE] ● 
│   └── tx_60_by_18 (A18/B+) [BULL 4842Y/559N B=7 ⚠WHALE] ● 
│       └── tx_293_by_14 (A14/B+) (50%) ○ 
├── tx_3_by_26 (A26/B-) (50%) ○ 
├── tx_4_by_18 (A18/B+) [BULL 5835Y B=2 ⚠WHALE] ● 
│   ├── tx_13_by_26 (A26/B-) [BULL 44Y B=2] ● 
│   │   └── tx_22_by_3 (A3/M) (50%) ○ 
│   │       ├── tx_108_by_17 (A17/B+) [BULL 40Y B=1] ● 
│   │       │   └── tx_161_by_25 (A25/B-) [BULL 10064Y/5459N B=11 ⚠WHALE] ● 
│   │       │       └── tx_177_by_29 (A29/B-) (50%) ○ 
│   │       └── tx_110_by_4 (A4/M) [BULL 40Y/20N B=2] ● 
│   │           └── tx_193_by_4 (A4/M) [BEAR 255Y/993N B=7 ⚠WHALE] ○ 
│   │               └── tx_295_by_0 (A0/M) (50%) ○ 
│   └── tx_20_by_11 (A11/B+) [BULL 1500Y B=1 ⚠WHALE] ● 
│       ├── tx_56_by_26 (A26/B-) [BEAR 8Y/191N B=4] ○ 
│       │   └── tx_224_by_23 (A23/B-) [BULL 1145Y/4N B=9 ⚠WHALE] ● 
│       │       └── tx_264_by_18 (A18/B+) [BULL 44Y B=2] ● 
│       └── tx_63_by_17 (A17/B+) [BEAR 4Y/40N B=2] ○ 
│           └── tx_79_by_18 (A18/B+) [BULL 5993Y/1499N B=9 ⚠WHALE] ● 
│               └── tx_83_by_9 (A9/M) [BULL 135Y B=2] ● 
│                   └── tx_164_by_9 (A9/M) [BULL 82Y/4N B=4] ● 
│                       └── tx_218_by_5 (A5/M) [BEAR 139Y/227N B=8] ○ 
├── tx_6_by_5 (A5/M) [BULL 44Y B=2] ● 
│   └── tx_36_by_13 (A13/B+) (50%) ○ 
│       └── tx_89_by_13 (A13/B+) [BULL 4Y B=1] ● 
│           ├── tx_102_by_24 (A24/B-) [BULL 6593Y B=18 ⚠WHALE] ● 
│           │   └── tx_150_by_6 (A6/M) [BULL 4Y B=1] ● 
│           │       └── tx_180_by_2 (A2/M) (50%) ○ 
│           │           └── tx_225_by_18 (A18/B+) (50%) ○ 
│           └── tx_103_by_11 (A11/B+) [BEAR 40N B=1] ○ 
│               └── tx_121_by_2 (A2/M) (50%) ○ 
│                   └── tx_163_by_21 (A21/B-) (50%) ○ 
├── tx_7_by_12 (A12/B+) [BULL 891Y/65N B=4 ⚠WHALE] ● 
│   └── tx_172_by_24 (A24/B-) [BEAR 40Y/100N B=2] ○ 
│       └── tx_275_by_26 (A26/B-) (50%) ○ 
├── tx_8_by_13 (A13/B+) (50%) ○ 
│   └── tx_73_by_18 (A18/B+) [BEAR 40Y/80N B=3] ○ 
│       ├── tx_149_by_14 (A14/B+) [BEAR 40N B=1] ○ 
│       │   ├── tx_182_by_19 (A19/B+) (50%) ○ 
│       │   │   └── tx_248_by_21 (A21/B-) [BULL 98Y B=1] ● 
│       │   │       └── tx_278_by_21 (A21/B-) [BULL 40Y B=1] ● 
│       │   └── tx_185_by_25 (A25/B-) [BULL 98Y B=1] ● 
│       │       └── tx_200_by_3 (A3/M) (50%) ○ 
│       │           └── tx_261_by_22 (A22/B-) (50%) ○ 
│       └── tx_155_by_0 (A0/M) (50%) ○ 
│           └── tx_187_by_22 (A22/B-) (50%) ○ 
│               └── tx_260_by_28 (A28/B-) (50%) ○ 
│                   └── tx_276_by_12 (A12/B+) (50%) ○ 
│                       └── tx_294_by_4 (A4/M) (50%) ○ 
├── tx_9_by_24 (A24/B-) (50%) ○ 
│   └── tx_280_by_21 (A21/B-) (50%) ○ 
├── tx_10_by_25 (A25/B-) [BULL 4Y B=1] ● 
│   └── tx_58_by_27 (A27/B-) (50%) ○ 
│       └── tx_129_by_4 (A4/M) [BULL 40Y B=1] ● 
│           ├── tx_151_by_12 (A12/B+) (50%) ○ 
│           │   └── tx_213_by_24 (A24/B-) [BULL 4Y B=1] ● 
│           └── tx_178_by_3 (A3/M) [BULL 101Y B=2] ● 
│               └── tx_199_by_16 (A16/B+) (50%) ○ 
│                   └── tx_252_by_25 (A25/B-) (50%) ○ 
│                       └── tx_285_by_4 (A4/M) (50%) ○ 
├── tx_17_by_19 (A19/B+) [BULL 1527Y B=3 ⚠WHALE] ● 
│   └── tx_191_by_8 (A8/M) [BULL 17435Y B=13 ⚠WHALE] ● 
│       └── tx_241_by_5 (A5/M) [BEAR 98N B=1] ○ 
│           └── tx_254_by_16 (A16/B+) [BEAR 40N B=1] ○ 
└── tx_19_by_29 (A29/B-) [BULL 3279Y B=5 ⚠WHALE] ● 
    └── tx_37_by_29 (A29/B-) [BEAR 98N B=1] ○ 
        └── tx_46_by_16 (A16/B+) [BULL 1011Y/71N B=9 ⚠WHALE] ● 
            └── tx_90_by_3 (A3/M) [BULL 873Y/135N B=3 ⚠WHALE] ● 
                └── tx_152_by_5 (A5/M) [BULL 210Y/43N B=5] ● 
                    └── tx_194_by_6 (A6/M) (50%) ○ 
                        └── tx_226_by_3 (A3/M) [BEAR 266N B=5] ○ 
                            └── tx_250_by_3 (A3/M) [BULL 280Y B=9] ● 
```

## Role Activity Breakdown

| Role | Agents | Nodes Created | % of Total |
|------|--------|--------------|------------|
| Mathematician | 0-9 | 28 | 34% |
| Bull | 10-19 | 27 | 32% |
| Bear | 20-29 | 27 | 32% |

## Trading Activity by Role

| Role | YES Buys | NO Buys | Net Direction |
|------|----------|---------|---------------|
| Mathematician | 62 | 4 | BULL |
| Bull | 74 | 13 | BULL |
| Bear | 34 | 26 | BULL |

## Run 3 vs Run 4 Comparison

| Metric | Run 3 (15 agents) | Run 4 (30 agents) |
|--------|------------------|-------------------|
| Agents | 15 (5/5/5) | **30 (10/10/10)** |
| Nodes | 99 | **82** |
| Traded | 58/99 (59%) | **50/82 (60%)** |
| BUY YES | 141 | **170** |
| BUY NO | 54 | **43** |
| YES:NO | 2.6:1 | **4.0:1** |
| Roots | 3 | **11** |
| Max depth | 11 | **9** |
| Generation | 2 | **1** |
| Proved? | NO | **NO** |

## Key Findings

### 1. Scaling Effect: More Agents = More Roots, Less Depth
- **11 roots** (vs 3 in Run 3). 30 agents create more parallel exploration paths.
- **Max depth 9** (vs 11). More agents compete for the same frontier → shorter chains.
- **82 nodes** (vs 99). More tx spent on trading, fewer unique nodes created. Each node gets more market attention.

### 2. Bear Role Diluted at Scale
- **Bears went BULL**: 34 YES vs 26 NO. In Run 3, bears were net BEAR (27Y/43N). Doubling bears didn't double shorting — it diluted conviction.
- **YES:NO ratio worsened**: 4.0:1 (vs 2.6:1 in Run 3). More agents = more herd behavior, less contrarian edge.
- **Hypothesis**: 7B model lacks mathematical sophistication to find genuine flaws. Bears with nothing to short default to buying YES.

### 3. Whale Concentration — EXTREME
- tx_14_by_16: **28,529Y / 9,120N** (25 bets!) — single node absorbed ~38K coins
- tx_191_by_8: **17,435Y** (13 bets) — mathematician node became capital magnet
- tx_161_by_25: **10,064Y / 5,459N** (11 bets) — Bear-created node, heavily contested
- tx_47_by_22: **11,012Y / 3,994N** — Bear-created, Bull-endorsed
- **Pattern**: Capital concentrates on early nodes. Late nodes starve. This is CPMM's natural gravity — early LP = cheap entry, late LP = expensive.

### 4. Role Balance in Node Creation — PERFECT
- Math 34%, Bull 32%, Bear 32% — nearly uniform. All roles contribute to the DAG.
- Contrast with Run 3: Math 61%, Bull 20%, Bear 19%. Doubling non-math agents equalized output.
- **But**: more Bull/Bear nodes = more "thin" reasoning (these agents optimize for trading, not proof quality).

### 5. Contested Nodes — Genuine Price Discovery
- tx_193_by_4: 255Y vs **993N** (7 bets) — most shorted node, Math-created
- tx_218_by_5: 139Y vs **227N** (8 bets) — Math node challenged by bears
- tx_56_by_26: 8Y vs **191N** (4 bets) — Bear-created, self-shorted by other bears
- tx_224_by_23: Bear A23 created → then **1,145Y** poured in — market reversed Bear's own pessimism

### 6. Diagnosis: 30 Agents at 7B is "Warm Noise"
- Architecture works: channels don't block, market clears, roles differentiate.
- Economics work: capital flows, prices move, contested nodes exist.
- **Math doesn't work**: 7B can't prove ζ regularization. Agents recycle the same 5 ideas (DCT, integral, cos average, exp→1, Gamma function) without advancing.
- The 30-agent run is a **stress test of the microkernel**, not a math experiment. The kernel passed.
