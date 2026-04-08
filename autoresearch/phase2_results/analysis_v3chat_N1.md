# Phase 2: DeepSeek V3.2 Chat — N=1 TuringOS vs Baseline

## Date: 2026-04-07

## Experiment Config
- Model: deepseek-chat (V3.2)
- TuringOS: N=1, 1M/0B+/0B-, 30min wall clock, LP=200, Librarian interval=8
- Baseline: same prompt, oneshot x5, temperature=0.5

## Results

### Baseline (bare oneshot)
| Run | Tokens | Time | [COMPLETE] | Verdict |
|-----|--------|------|------------|---------|
| 1   | 2851   | 58s  | Yes        | MIXED (cites ζ(-1)) |
| 2   | 3668   | 79s  | Yes        | MIXED (cites ζ(-1)) |
| 3   | 4926   | 104s | Yes        | MIXED (cites ζ(-1)) |
| 4   | 3908   | 81s  | Yes        | MIXED (cites ζ(-1)) |
| 5   | 4972   | 105s | Yes        | MIXED (cites ζ(-1)) |

All 5 runs do partial construction (complex exponential, Taylor expansion) but cite ζ(-1)=-1/12 at the end.

### TuringOS N=1
- **Nodes**: 69, **Transactions**: 200, **Deepest chain**: 18 steps
- **[COMPLETE]**: 0, **-1/12 reached**: 2 nodes (tx_137, tx_193)
- **Tokens**: 86,741 completion / 535,840 total
- **ζ(-1) shortcut**: NO — genuine construction throughout
- **Agent bankrupted** at tx 175 (105 bets, avg 95 Coins, total 10,000)
- **0 stagnation events** (heartbeat fix confirmed working)

### Golden Path (18 steps)
1. Define regularized sum S(N)
2. Euler's formula: cos = Re(e^{im/N})
3. Geometric series: Σmr^m = r/(1-r)²
4. Substitute r = e^{(i-1)/N}
5. Taylor expand e^{(i-1)/N} to O(1/N⁴)
6. Compute 1 - e^{(i-1)/N}
7-11. Square the denominator, substitute (i-1)²=-2i, (i-1)³=2+2i
12-15. Divide numerator by denominator, factor 1/N², geometric series for reciprocal
16-17. Multiply expansions
18. Extract real part → asymptotic formula

### Key Node: tx_193
> "Take the limit as N → ∞ of the asymptotic expression for S(N): lim_{N→∞} S(N) = lim_{N→∞} (-1/12 - 1/(4N) + O(1/N²)) = -1/12."

### Key Node: tx_137
> "S(N) = Re( ((i-1)/N)^{-2} - 1/12 + (1/4)((i-1)/N) + O(1/N²) ). Re(iN²/2) = 0, Re(-1/12) = -1/12"

## Diagnosis: Why No OMEGA

1. Agent went bankrupt (10,000 Coins spent on 105 bets)
2. In N=1, no market counterparty → all investments are pure cost
3. Bankrupt agent can append (free) but cannot invest
4. Without investment, no node reaches P≥90% for Oracle trigger
5. tx_193 has the correct answer but was never validated

## Librarian Quality: EXCELLENT
- Correctly identifies what works (Taylor expansion, geometric series)
- Correctly identifies anti-patterns (premature factorization, truncation errors)
- Recommends next steps aligned with proof direction

## Heartbeat Fix: CONFIRMED
- 0 stagnation, 0 timeout events in 30 minutes
- Previous run: 49 timeouts, 14 stagnation events, 1 append
- Fix: reactor broadcasts snapshot on every 30s timeout

## Next: N=3 experiment (3 math agents, market counterparty enabled)
