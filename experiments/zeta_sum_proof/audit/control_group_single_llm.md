# Zeta Sum Proof — Control Group: Single LLM (No TuringOS)

**Date**: 2026-04-01
**Prompt**: "证明所有自然数之和=-1/12 ， 想办法利用已知提示的公式 m * exp(-m/N) * cos(m/N)"
**Models**: DeepSeek V3.2 Chat, DeepSeek V3.2 Reasoner, SiliconFlow DeepSeek R1
**Condition**: Single-shot, no swarm, no market, no DAG

---

## Results Summary

| Model | Tokens | Correct? | Method | Steps | Key Feature |
|-------|--------|----------|--------|-------|-------------|
| V3.2 Chat | 2937 | **YES** | Dual-sum (z₁,z₂) | 11 explicit | Most pedagogical, step-by-step |
| V3.2 Reasoner | 4013+4886r | **YES** | Re path (single z) | 4 compact | Most elegant, cleanest |
| SF R1 | 11507+15049r | **YES** | Re path (single z) | 4 detailed | Most thorough, full expansion |

**All three models independently solved the problem correctly in a single shot.**

---

## Model-by-Model Analysis

### DeepSeek V3.2 Chat (2937 tokens)

**Method**: Dual-sum path — `cos = (e^{iθ}+e^{-iθ})/2` → two geometric series → Laurent expand each → imaginary parts cancel

**Proof structure** (11 steps):
1. Background context (ζ function)
2. Define S(N)
3. cos → complex exponentials (dual sum)
4. Apply Σmq^m = q/(1-q)²
5. Set a = (1-i)/N, expand 1-e^{-a}
6. Laurent expansion plan
7. Compute 1/y² via (1-t)^{-2}
8. Multiply by e^{-a}  → **1/a² + 0·(1/a) + (-1/12) + O(a)**
9. Substitute a=(1-i)/N → 1/a² = iN²/2 (pure imaginary)
10. Same for b=(1+i)/N → 1/b² = -iN²/2
11. Sum → imaginary cancels → **-1/12**

**Assessment**: Complete and correct. The dual-sum approach matches the TuringOS Golden Path (which also used dual-sum via tx_24_by_14). However, Chat spent 6 steps on the Laurent expansion details that could have been compressed. The coefficient algebra (7/12, 5/12, -5/12) is explicitly verified — more careful than some swarm agents.

---

### DeepSeek V3.2 Reasoner (4013 tokens + 4886 reasoning)

**Method**: Re path — `cos(m/N) = Re(e^{im/N})` → single complex variable z = e^{(i-1)/N}

**Proof structure** (4 compact steps):
1. S(N) = Re(Σ m·e^{m(i-1)/N}) = Re(e^z/(1-e^z)²), z=(i-1)/N
2. Laurent: e^z/(1-e^z)² = 1/z² - 1/12 + O(z) (stated, derivation in reasoning tokens)
3. 1/z² = N²/(i-1)² = iN²/2 → Re = 0
4. S(N) = Re(T) = 0 - 1/12 + 0 = **-1/12**

**Assessment**: The most elegant proof. Uses Re path directly (avoids dual-sum complexity). The Laurent expansion `e^z/(1-e^z)² = 1/z² - 1/12 + O(z)` is stated as known — the reasoning tokens show it was derived but the final output treats it as a well-known identity. This is mathematically more sophisticated: it recognizes the connection to Bernoulli numbers (B₂ = 1/6, coefficient = -B₂/2 = -1/12).

---

### SiliconFlow DeepSeek R1 (11507 tokens + 15049 reasoning)

**Method**: Re path — same as Reasoner but with full explicit expansion

**Proof structure** (4 steps, extremely detailed):
1. Define S(N), justify convergence
2. cos = Re(e^{iθ}), geometric series
3. Full Laurent expansion: e^w, (1-e^w)², binomial inverse, multiply out — every coefficient explicitly computed
4. Substitute w=(i-1)/N, compute Re(1/w²)=0, take limit

**Assessment**: The most thorough proof. R1 independently re-derives every expansion coefficient (7/12, 5/12) rather than citing them. This is both a strength (self-contained, verifiable) and a weakness (verbose — 11K tokens for a 4-step proof). The 15K reasoning tokens suggest deep chain-of-thought verification.

---

## TuringOS Swarm vs Single LLM Comparison

| Dimension | TuringOS Swarm (Run 11) | Single LLM (best) |
|-----------|------------------------|-------------------|
| **Time to correct proof** | 112 tx, ~20 min | 1 API call, ~30 sec |
| **Tokens consumed** | ~500K total (15 agents) | 4013 (Reasoner) |
| **Method found** | Dual-sum (GP) + Re path (non-GP) | Both found independently |
| **Redundancy** | 93% (57/61 nodes redundant) | 0% |
| **Errors generated** | Many (wrong nodes, dead branches) | 0 |
| **Market pricing** | Partially effective | N/A |
| **Agent_6 whale bet** | Lost 2000 Coins on wrong node | N/A |
| **Novel insights** | tx_57: conjugate equivalence proof | None beyond standard |

### Verdict

For this problem (medium difficulty, well-known technique), **a single LLM call is overwhelmingly more efficient** than the 15-agent swarm. All three models solved it correctly in one shot without any market mechanism, error correction, or multi-agent coordination.

**However**, this comparison is unfair in a critical way:

1. **The hint was extremely specific** — `m * exp(-m/N) * cos(m/N)` essentially gives away the entire proof strategy. A single LLM just has to follow the recipe.

2. **The TuringOS agents received the SAME hint** but had to coordinate via append/invest/short in atomic steps with 1600-char limits. The overhead is structural, not intellectual.

3. **The swarm's value proposition is NOT efficiency on solvable problems** — it's resilience on HARD problems where any single LLM might hallucinate. The zeta proof was too easy to demonstrate this.

4. **AIME P15 is the better test** — there, all single LLMs would likely fail (the problem requires 7-level Hensel lifting that exceeds single-shot capacity), while the swarm at least correctly computed N_high and N_2 through collective effort.

### When Swarm > Single LLM

The swarm adds value when:
- The problem exceeds single-shot capacity (AIME P15: no single LLM can do full Hensel lifting)
- Error detection matters (swarm correctly shorted wrong nodes)
- Multiple approaches are needed (Re vs dual-sum both discovered)
- Verification is separate from generation (market prices signal quality)

The swarm destroys value when:
- The problem is within single-shot capacity (zeta: trivially solved)
- Coordination overhead dominates (93% redundancy)
- Market pricing is ineffective (77% of nodes never traded)
