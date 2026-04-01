# Zeta ζ-Sum — Run 5: 90 Agents Role Trifecta (Pro/Qwen2.5-7B-Instruct)

**72 nodes | 300 tx | 47 traded | 25 untraded | NOT proved**
**Model**: Pro/Qwen2.5-7B-Instruct (SiliconFlow) | **Roles**: 30M/30B+/30B-
**BUY YES**: 187 | **BUY NO**: 39 | **Ratio**: 4.8:1
**Duration**: ~1 min | **Generation**: 1 | **Max depth**: 5 | **Roots**: 19

> **Experiment 2026-04-01**: Scale to 90 agents (30/30/30). Same weak model.
> Tests scaling limits of the microkernel and market dynamics under high concurrency.

```
● = P=1 (high price)   ○ = P=0 (low price)
M = Mathematician(0-29)  B+ = Bull(30-59)  B- = Bear(60-89)
[BULL xY B=n] = YES-dominant   [BEAR xN B=n] = NO-dominant
(50%) = never traded   ⚠WHALE = position >500C
```

## Citation Tree (19 roots)

```
ROOT (72 nodes, 47 traded, 25 untraded)
├── tx_1_by_71 (A71/B-) [BULL 9672Y B=15 ⚠WHALE] ● 
│   └── tx_74_by_88 (A88/B-) [BULL 40Y B=1] ● 
├── tx_2_by_58 (A58/B+) [BULL 5838Y B=3 ⚠WHALE] ● 
│   ├── tx_98_by_47 (A47/B+) [BULL 51Y/45N B=6] ● 
│   │   ├── tx_191_by_71 (A71/B-) [BULL 44Y B=2] ● 
│   │   └── tx_217_by_64 (A64/B-) (50%) ○ 
│   └── tx_131_by_82 (A82/B-) (50%) ○ 
│       └── tx_220_by_59 (A59/B+) [BULL 6039Y/769N B=12 ⚠WHALE] ● 
├── tx_3_by_63 (A63/B-) [BULL 3804Y B=9 ⚠WHALE] ● 
│   └── tx_84_by_37 (A37/B+) [BULL 40Y B=1] ● 
│       └── tx_211_by_22 (A22/M) [BULL 4Y B=1] ● 
├── tx_4_by_56 (A56/B+) [BULL 12Y B=3] ● 
│   └── tx_259_by_9 (A9/M) (50%) ○ 
├── tx_5_by_78 (A78/B-) [BEAR 4N B=1] ○ 
│   └── tx_56_by_33 (A33/B+) [BULL 1587Y B=3 ⚠WHALE] ● 
│       └── tx_196_by_1 (A1/M) (50%) ○ 
├── tx_6_by_74 (A74/B-) (50%) ○ 
│   └── tx_227_by_67 (A67/B-) [BULL 40Y B=1] ● 
├── tx_7_by_27 (A27/M) [BULL 44Y B=2] ● 
│   └── tx_251_by_36 (A36/B+) [BULL 40Y B=1] ● 
├── tx_8_by_33 (A33/B+) [BULL 44Y B=2] ● 
│   ├── tx_54_by_32 (A32/B+) (50%) ○ 
│   │   ├── tx_80_by_51 (A51/B+) [BULL 44Y B=2] ● 
│   │   │   └── tx_124_by_18 (A18/M) [BULL 50Y/40N B=3] ● 
│   │   │       ├── tx_231_by_0 (A0/M) [BULL 44Y B=2] ● 
│   │   │       │   └── tx_272_by_0 (A0/M) (50%) ○ 
│   │   │       ├── tx_238_by_49 (A49/B+) [BEAR 20N B=1] ○ 
│   │   │       │   ├── tx_275_by_78 (A78/B-) (50%) ○ 
│   │   │       │   └── tx_285_by_38 (A38/B+) (50%) ○ 
│   │   │       └── tx_244_by_29 (A29/M) (50%) ○ 
│   │   │           └── tx_292_by_87 (A87/B-) (50%) ○ 
│   │   └── tx_123_by_31 (A31/B+) (50%) ○ 
│   │       └── tx_173_by_56 (A56/B+) [BULL 8Y B=2] ● 
│   └── tx_58_by_26 (A26/M) (50%) ○ 
├── tx_9_by_53 (A53/B+) [BULL 1554Y/4N B=5 ⚠WHALE] ● 
├── tx_10_by_76 (A76/B-) [BULL 6491Y B=7 ⚠WHALE] ● 
│   └── tx_108_by_75 (A75/B-) [BULL 105Y/83N B=5] ● 
├── tx_11_by_86 (A86/B-) (50%) ○ 
│   └── tx_114_by_30 (A30/B+) (50%) ○ 
│       └── tx_181_by_54 (A54/B+) [BULL 9900Y B=1 ⚠WHALE] ● 
│           └── tx_237_by_40 (A40/B+) (50%) ○ 
├── tx_12_by_26 (A26/M) [BULL 8Y/4N B=3] ● 
│   └── tx_90_by_89 (A89/B-) [BULL 4Y B=1] ● 
│       └── tx_199_by_52 (A52/B+) [BULL 2Y B=1] ● 
├── tx_13_by_85 (A85/B-) [BULL 105Y B=4] ● 
│   └── tx_89_by_68 (A68/B-) (50%) ○ 
│       └── tx_153_by_51 (A51/B+) [BULL 78Y B=2] ● 
│           ├── tx_221_by_86 (A86/B-) (50%) ○ 
│           └── tx_222_by_19 (A19/M) (50%) ○ 
│               └── tx_247_by_16 (A16/M) (50%) ○ 
├── tx_14_by_15 (A15/M) [BULL 1527Y B=3 ⚠WHALE] ● 
│   └── tx_107_by_27 (A27/M) [BULL 457Y B=10] ● 
│       ├── tx_218_by_24 (A24/M) [BULL 59Y B=1] ● 
│       └── tx_241_by_88 (A88/B-) [BULL 98Y B=1] ● 
├── tx_15_by_49 (A49/B+) [BULL 102Y/4N B=3] ● 
├── tx_16_by_32 (A32/B+) [BULL 23110Y B=17 ⚠WHALE] ● 
│   └── tx_245_by_66 (A66/B-) (50%) ○ 
├── tx_17_by_70 (A70/B-) [BULL 1746Y B=8 ⚠WHALE] ● 
│   └── tx_93_by_38 (A38/B+) [BEAR 63Y/545N B=24 ⚠WHALE] ○ 
│       └── tx_116_by_4 (A4/M) [BULL 205Y/83N B=7] ● 
│           ├── tx_165_by_53 (A53/B+) [BULL 2416Y B=13 ⚠WHALE] ● 
│           │   ├── tx_235_by_25 (A25/M) [BULL 40Y B=1] ● 
│           │   └── tx_243_by_17 (A17/M) [BEAR 4N B=1] ○ 
│           └── tx_167_by_16 (A16/M) [BULL 209Y B=4] ● 
│               └── tx_200_by_17 (A17/M) (50%) ○ 
├── tx_18_by_66 (A66/B-) [BULL 40Y B=1] ● 
│   └── tx_162_by_69 (A69/B-) (50%) ○ 
└── tx_19_by_83 (A83/B-) [BULL 9974Y/40N B=11 ⚠WHALE] ● 
    ├── tx_137_by_7 (A7/M) (50%) ○ 
    │   └── tx_254_by_31 (A31/B+) [BULL 135Y B=2] ● 
    └── tx_156_by_46 (A46/B+) [BULL 6102Y/742N B=17 ⚠WHALE] ● 
        └── tx_258_by_50 (A50/B+) (50%) ○ 
```

## Role Activity Breakdown

| Role | Agents | Nodes Created | % of Total |
|------|--------|--------------|------------|
| Mathematician | 0-29 | 21 | 29% |
| Bull | 30-59 | 27 | 37% |
| Bear | 60-89 | 24 | 33% |

## Trading Activity by Role

| Role | YES Buys | NO Buys | Net Direction |
|------|----------|---------|---------------|
| Mathematician | 72 | 10 | BULL |
| Bull | 81 | 6 | BULL |
| Bear | 34 | 23 | BULL |

## Top Contested Nodes (Bull/Bear Battlegrounds)

| Node | YES Coins | NO Coins | Bets | Winner |
|------|-----------|----------|------|--------|
| tx_93_by_38 | 63 | 545 | 24 | BEAR |
| tx_156_by_46 | 6102 | 742 | 17 | BULL |
| tx_220_by_59 | 6039 | 769 | 12 | BULL |
| tx_19_by_83 | 9974 | 40 | 11 | BULL |
| tx_116_by_4 | 205 | 83 | 7 | BULL |
| tx_98_by_47 | 51 | 45 | 6 | BULL |
| tx_9_by_53 | 1554 | 4 | 5 | BULL |
| tx_108_by_75 | 105 | 83 | 5 | BULL |
| tx_12_by_26 | 8 | 4 | 3 | BULL |
| tx_15_by_49 | 102 | 4 | 3 | BULL |

## Whale Nodes (>500C total capital)

| Node | YES | NO | Bets | Total | Author |
|------|-----|-----|------|-------|--------|
| tx_16_by_32 | 23110 | 0 | 17 | 23110 | Agent_32/B+ |
| tx_19_by_83 | 9974 | 40 | 11 | 10014 | Agent_83/B- |
| tx_181_by_54 | 9900 | 0 | 1 | 9900 | Agent_54/B+ |
| tx_1_by_71 | 9672 | 0 | 15 | 9672 | Agent_71/B- |
| tx_156_by_46 | 6102 | 742 | 17 | 6843 | Agent_46/B+ |
| tx_220_by_59 | 6039 | 769 | 12 | 6808 | Agent_59/B+ |
| tx_10_by_76 | 6491 | 0 | 7 | 6491 | Agent_76/B- |
| tx_2_by_58 | 5838 | 0 | 3 | 5838 | Agent_58/B+ |
| tx_3_by_63 | 3804 | 0 | 9 | 3804 | Agent_63/B- |
| tx_165_by_53 | 2416 | 0 | 13 | 2416 | Agent_53/B+ |

## Scaling Comparison: Run 3 → Run 4 → Run 5

| Metric | Run 3 (15) | Run 4 (30) | Run 5 (90) |
|--------|-----------|-----------|-----------|
| Agents | 5/5/5 | 10/10/10 | **30/30/30** |
| Nodes | 99 | 82 | **72** |
| Traded | 58 (59%) | 50 (60%) | **47 (65%)** |
| BUY YES | 141 | 170 | **187** |
| BUY NO | 54 | 43 | **39** |
| YES:NO | 2.6:1 | 4.0:1 | **4.8:1** |
| Roots | 3 | 11 | **19** |
| Max depth | 11 | 9 | **5** |
| Nodes/agent | 6.6 | 2.7 | **0.8** |

## Key Findings

### 1. Scaling Law Discovered: Inverse Depth
```
Agents:    15  →  30  →  90
Depth:     11  →   9  →   5
Roots:      3  →  11  →  19
Nodes:     99  →  82  →  72
```
**More agents = shallower, wider, fewer nodes.** 90 agents produce 0.8 nodes/agent (vs 6.6 at 15). The 300 tx budget is consumed by trading, not building. This is the **market congestion effect**: agents spend their turns evaluating and investing in existing nodes rather than creating new ones.

### 2. Bear Extinction at Scale
- YES:NO ratio: 2.6:1 → 4.0:1 → **4.8:1**. Bears become increasingly impotent as scale grows.
- Bears are net BULL again (34Y vs 23N). At 90 agents, the "find genuine flaws" strategy collapses because 7B model can't distinguish correct from incorrect steps.
- **The problem isn't the role prompt — it's the model capability.** Bears need mathematical sophistication to short; without it, they herd.

### 3. Whale Dominance — Capital Black Holes
- tx_16_by_32 (Bull): **23,110Y, zero NO** — 17 bets all bullish, no one dares short
- tx_181_by_54 (Bull): **9,900Y in a single bet** — one agent dumped everything
- Top 10 whales hold ~80K coins. With 90 agents × 10K = 900K total system coins, top 10 nodes captured ~9% of all capital.
- **CPMM early-mover advantage**: first few bets on a node are cheap; by bet #5, the price is so high that shorting requires massive capital. This creates an irreversible BULL spiral on early nodes.

### 4. The "Tragedy of the Roots"
- 19 roots from 90 agents, but most roots are **orphaned** — only 1-2 children, then abandoned.
- Contrast Run 3: 3 roots, deep chains (max depth 11). Fewer agents = more depth, more sustained exploration.
- **Insight**: agent count and proof depth are inversely correlated at fixed tx budget. To get depth, you need either more tx or fewer agents competing for attention.

### 5. Microkernel Stress Test — PASSED
- 90 concurrent tokio tasks, all hitting SiliconFlow API simultaneously.
- 300 tx completed in ~1 min (no stagnation, no generation rollover).
- Market maker created 72 markets, processed 226 trades. Zero panics, zero data corruption.
- **The kernel scales linearly.** Bottleneck is API throughput, not architecture.

### 6. Contested Node tx_93_by_38 — Only Real Bear Victory
- Bull A38 created this node, then it got **545N from 24 bets** — the most shorted node by far.
- Despite the shorting, its child tx_116 attracted 205Y. The market disagreed with itself across generations.
- This is the ONLY node where bears won decisively. In a 900K-coin economy with 30 bears, one contested node is pathological.

## Architectural Implications

1. **tx budget should scale with agent count**. At 90 agents, 300 tx = 3.3 tx/agent. Each agent barely gets to act once before the budget runs out. Recommend: `MAX_TRANSACTIONS = agents × 10` minimum.
2. **Bear effectiveness requires model capability, not just role prompts.** The 15-agent run (Run 3) had the best bear performance because each bear had more turns to carefully evaluate nodes.
3. **CPMM creates natural BULL gravity.** The constant-product formula makes early YES bets cheap and late NO bets expensive. This systematically disadvantages bears. Consider asymmetric LP or bear-side subsidies at the protocol level.
4. **Depth vs breadth is a fundamental tradeoff.** More agents explore more roots but shallower. For proof problems requiring deep reasoning chains, fewer agents with more tx may outperform more agents.
