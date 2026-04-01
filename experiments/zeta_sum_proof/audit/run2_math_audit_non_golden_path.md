# Zeta Sum Proof Run 11 — Complete DAG: All 61 Nodes with Pricing & Classification

**Run**: Run 11 (global) / Run 2 (zeta local) — 112 tx, 61 nodes, 47 bets, OMEGA reached
**Golden Path**: `tx_7_by_10` → `tx_24_by_14` → `tx_70_by_10` → `tx_112_by_9` (4 steps)
**Post-Settlement**: GP = Price 1.00, all others = Price 0.00
**Live Prices**: shown as P:XX% (peak during trading) or (50%) if never traded

---

## Legend

```
✓ = Golden Path (settled 1.00)   ★ = novel insight, not GP    ◎ = duplicate
△ = correct but incomplete       ⚠ = whale bet (>1000C)       ✗ = error (NONE in run)
Price shown = LIVE peak during trading. Post-settlement: GP→1.00, rest→0.00.
```

---

## Horizontal DAG: Proof Flows Left → Right

```
 STEP 1: Define S(N)          STEP 2: Euler + Σmz^m           STEP 3: Laurent Expansion        STEP 4: Limit → OMEGA
 ═══════════════════          ═══════════════════════          ══════════════════════════        ══════════════════════

 ✓ tx_7  [A10] 52%  ──────→  ✓ tx_24 [A14] (50%) ─────────→  ✓ tx_70 [A10] (50%) ──────────→  ✓ tx_112 [A9] (50%) ★ OMEGA
   "ratio test proof"           "cos=(e+e⁻)/2, z₁,z₂"          "1/((i-1)²ε²) - 1/12"           "Re=0 → lim = -1/12"
   B=2 (A12 YES, A1 YES)       DUAL-SUM PATH ◄ GP              THE -1/12 EMERGES HERE           [COMPLETE] → Lean 4 ✓
   only GP node with bets!      zero bets on GP node!           zero bets on GP node!            zero bets on OMEGA node!

                              ┌─────────────────────────────┐
                              │ TWO METHOD BRANCHES FOUND    │
                              │                             │
                              │ DUAL-SUM: cos=(e+e⁻)/2     │
                              │ → z₁, z₂ (GP chose this)   │
                              │                             │
                              │ Re PATH: cos=Re(e^{iθ})    │
                              │ → single z (simpler)        │
                              │                             │
                              │ ★ tx_57 [A8] (50%)         │
                              │ PROVED EQUIVALENCE          │
                              │ z₂=conj(z₁) → Re(...)      │
                              │ Most valuable non-GP node   │
                              │ ZERO BETS. Gem ignored.     │
                              └─────────────────────────────┘

                                                               INDEPENDENT LAURENT DERIVATIONS
                                                               (4 agents found -1/12 on their own):

                                                               ★ tx_91  [A14] (50%) "N²/(1-i)²-1/12"
                                                               ★ tx_92  [A13] (50%) "N²/(i-1)²-1/12"
                                                               ★ tx_103 [A0]  (50%) "1/w²-1/12+O(w²)"
                                                               ★ tx_109 [A2]  (50%) "1/z²-1/12+z/12"
                                                               ALL AT 50%. NONE RECEIVED BETS.
                                                               Proof verified 5× independently.
                                                               Market saw none of it.
```

### Duplicate Chains (42 nodes, all ◎, showing representative samples)

```
 STEP 1 DUPLICATES (13)       STEP 2 DUPLICATES (17)         STEP 3 DUPLICATES (16)           STEP 4 PARTIALS (4)
 ─────────────────────        ────────────────────────        ─────────────────────────        ──────────────────────

 ◎ tx_1  [A4]  (50%)         DUAL-SUM branch:                STUCK AT CLOSED FORM:            △ tx_65  [A2]  (50%)
 ◎ tx_2  [A2]  (50%)         ◎ tx_12 [A4]  (50%)            ◎ tx_36 [A0]  (50%)              △ tx_73  [A11] (50%)
 ◎ tx_3  [A0]  (50%)         ◎ tx_23 [A6]  (50%)            ◎ tx_39 [A12] (50%)              △ tx_101 [A10] (50%)
 ◎ tx_4  [A12] (50%)         ◎ tx_27 [A2]  (50%)            ◎ tx_67 [A8]  (50%)              △ tx_105 [A8]  (50%)
 ◎ tx_5  [A8]  (50%)         ◎ tx_42 [A14] (50%)            ◎ tx_74 [A6]  (50%)
 ◎ tx_6  [A14] (50%)         ◎ tx_53 [A4]  (50%)            ◎ tx_85 [A7]  (50%)              ALL △ nodes had correct
 ◎ tx_8  [A6]  (50%)         ◎ tx_61 [A3]  (50%)            ◎ tx_87 [A1]  (50%)              direction but stopped
 ◎ tx_11 [A11] (50%)         ◎ tx_86 [A6]  (50%)            ◎ tx_93 [A11] (50%)              before reaching -1/12.
 ◎ tx_18 [A9]  (50%)                                         ◎ tx_100[A4]  (50%)
 ◎ tx_29 [A3]  (50%)         Re PATH branch:                 ◎ tx_102[A14] (50%)
 ◎ tx_41 [A5]  (50%)         ◎ tx_21 [A12] (50%)            ◎ tx_104[A3]  (50%)
 ◎ tx_47 [A1]  (50%)         ◎ tx_22 [A0]  (50%)            ◎ ... +5 more
 ◎ tx_49 [A7]  (50%)         ◎ tx_25 [A8]  (50%)
                              ◎ tx_34 [A4]  (50%)            INSIGHT buried in dups:
 ALL 50%. Zero bets.          ◎ tx_38 [A8]  (50%)            ★ tx_84 [A12] (50%) real(r,θ)
 12 agents wrote the          ◎ tx_56 [A14] (50%)            ★ tx_60 [A9]  (50%) exp/(exp-1)²
 exact same Step 1.           ◎ tx_66 [A6]  (50%)
                              ◎ tx_76 [A4]  (50%)            MARKET ERROR:
                                                              ◎ tx_46 [A6] 43% BEAR ⚠
                              ALL 50%. Zero bets.             CORRECT node shorted to 43%
                                                              (A4 NO 100, A8 NO 50)
```

### Heavily Traded Detached Nodes (not on GP, significant capital)

```
 NODE              AGENT   LIVE PRICE    BETS        MATH STATUS    MARKET VERDICT
 ─────────────     ─────   ──────────    ────        ───────────    ──────────────
 ⚠ tx_5  [A14]    A6      → 90.0%      2000C YES    ◎ Step 1+2     WORST: bubble → 0
 ◎ tx_6  [A6]     A10     → 54.8%      100C YES     ◎ Step 1       endorsing dup
 ◎ tx_13 [A3]     A0,4,6  → 52.0%      40C YES      ◎ Step 1       endorsing dup
 ◎ tx_91b [A8]    A8      → 54.8%      100C AUTO    ◎ Step 4 try   self-invest, lost

 ◎ tx_40 [A0]     A4,5,2  → 41.0%      200C NO      ◎ Step 2       MARKET ERROR: killed correct
 ◎ tx_21 [A8]     A8,12,14→ 42.2%      170C NO      ◎ Step 2       MARKET ERROR: killed correct
 ◎ tx_1  [A2]     A0,8,4,2→ 43.5%      140C NO      ◎ Step 1       MARKET ERROR: killed correct
 ◎ tx_46 [A6]     A8,4,A0 → 43.1%      150N/20Y     ◎ Step 3       MARKET ERROR: killed correct
```

---

## Node Classification Summary

| Category | Nodes | % | Market Price | Math |
|----------|-------|---|-------------|------|
| ✓ GP | 4 | 7% | tx_7: 52%, other 3: (50%) | Correct, OMEGA |
| ★ INSIGHT | 10 | 16% | ALL (50%) zero bets | Correct, valuable, ignored |
| ◎ DUPLICATE | 42 | 69% | (50%) or 41-48% shorted | Correct, redundant |
| △ PARTIAL | 4 | 7% | ALL (50%) | Correct, incomplete |
| ⚠ WHALE | 1 | 2% | 90% bubble → settled 0 | Correct, lost 2000C |
| ✗ ERROR | **0** | **0%** | N/A | **No errors in run** |

## Price Spectrum

```
 90%  ┃ ⚠ tx_5 (whale 2000C)                      ← WORST SIGNAL (bubble on dead node)
      ┃
 55%  ┃ ◎ tx_6, tx_91_by_8 (duplicates)            ← endorsing wrong nodes
 52%  ┃ ✓ tx_7 GP Step 1                            ← ONLY correct market signal
 51%  ┃ ◎ tx_16,42,44,51,56 (duplicates)            ← noise
 50%  ┃████████████████████████████████████████████  ← 34 nodes: 3 GP(!), 10 insights(!!), dups
      ┃  3/4 GP = 50% (invisible!)
      ┃  ALL 10 insights = 50% (invisible!)
 48%  ┃ ◎ tx_55,69 (shorted dups)                   ← noise
 47%  ┃ ◎ tx_4,10,12,77,89 (shorted dups)           ← noise
 45%  ┃ ◎ tx_20,70_by_3 (shorted dups)              ← killing correct math
 43%  ┃ ◎ tx_1,46 (shorted correct nodes)           ← MARKET ERROR
 42%  ┃ ◎ tx_21 (170C short on correct math)        ← MARKET ERROR
 41%  ┃ ◎ tx_40 (200C short on correct math)        ← WORST SHORT (correct node killed)
```

## Market Scorecard: 1/10

| Task | Score | Detail |
|------|-------|--------|
| Identify GP Step 1 | 6/10 | tx_7 got 2 mild YES bets (52%) |
| Identify GP Step 2-4 | **0/10** | All three at 50%, zero activity |
| Identify 10 insight nodes | **0/10** | ALL at 50%, zero activity |
| Whale prevention | **0/10** | 2000C bubble at 90% on dead node |
| Correct nodes NOT shorted | **0/10** | tx_40 killed to 41%, tx_21 to 42% |
| Duplicate detection | 3/10 | Some shorted but can't tell dup from error |
| Error detection | N/A | No errors exist in this run |
| **OVERALL** | **1/10** | **Proof found by exploration, not market** |

**Paradox**: Market was WORSE on easy problem (Zeta 1/10) vs hard problem (AIME 5/10). Without errors to catch, the market is pure overhead + noise.
