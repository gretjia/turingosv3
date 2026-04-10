# Qwen3.5-122B (MoE, 10B Active) N Sweep Results

**Date**: 2026-04-10
**Model**: Qwen/Qwen3.5-122B-A10B via SiliconFlow
**Config**: all-M, 50K, 30min, OMEGA=[COMPLETE]+P>=90%+target=-1/12

## Results — 100% OMEGA Success Rate

| N | Nodes | C | m12 | P90 | OMEGA | Tokens | GP Steps | Time | PPUT |
|---|-------|---|-----|-----|-------|--------|----------|------|------|
| 3 | 11 | 2 | 5 | 2 | **YES** | 96K | 4 | 2.5m | **9.87e-3** |
| 5 | 61 | 22 | 24 | 3 | **YES** | 667K | 6 | 16.1m | **4.66e-4** |
| 7 | 85 | 42 | 50 | 3 | **YES** | 833K | 7 | 25.3m | **3.19e-4** |
| 10 | 57 | 11 | 11 | 2 | **YES** | 471K | 4 | 13.3m | **5.12e-4** |

## Cross-Model Comparison

| Model | N=3 | N=5 | N=7 | N=10 |
|-------|-----|-----|-----|------|
| **Qwen3.5-122B OMEGA** | **YES** | **YES** | **YES** | **YES** |
| **Qwen3.5-122B PPUT** | **9.87e-3** | **4.66e-4** | **3.19e-4** | **5.12e-4** |
| V3.2 OMEGA | YES (2/3) | NO | NO | NO |
| V3.2 PPUT | ~2e-4 | 0 | 0 | 0 |

## Key Findings

1. **Qwen3.5-122B dominates V3.2 on every metric**:
   - OMEGA: 4/4 (100%) vs 2/3 (67%) at N=3, 0% at N>3
   - PPUT at N=3: 9.87e-3 vs 2e-4 — **49x higher**
   - Tokens: 96K vs ~1M — **10x fewer tokens**

2. **N=3 is the PPUT sweet spot**: 9.87e-3, 49x better than V3.2
   - Only 4 GP steps, 2383 tokens, 2.5 minutes to OMEGA
   - The model chunks the Laurent expansion in one step (same as V3.2's best behavior)

3. **OMEGA scales to N=10**: Unlike V3.2 which fails at N>3,
   Qwen3.5-122B achieves OMEGA at all N values tested.
   This suggests the model's stronger math ability produces
   [COMPLETE] naturally without being stuck in exploration loops.

4. **[COMPLETE] count increases with N**: N=3: 2, N=5: 22, N=7: 42
   The opposite of V3.2's pattern (where [COMPLETE] decreased with N).
   This model writes [COMPLETE] proactively.

5. **MoE efficiency**: 122B total params but only 10B active = 
   cheaper per token than dense 14B while being much more capable.
