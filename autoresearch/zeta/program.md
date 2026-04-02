# TuringOS AutoResearch — Zeta ζ-Sum

## Goal

Find the optimal swarm configuration for **Pro/Qwen2.5-7B-Instruct** on the
ζ regularization proof (1+2+3+...= -1/12).

"Optimal" = the economic system (Polymarket + role differentiation) constrains
agents to produce **deep, novel, multi-strategy reasoning** rather than shallow
repetition or noise.

## Setup

1. Ensure the evaluator binary exists: `target/release/evaluator`
   (built from `experiments/zeta_sum_proof/`).
2. Ensure `.env` has `SILICONFLOW_API_KEY`.
3. Work in `autoresearch/zeta/`.
4. First run should always be the **baseline** (`config.json` as-is).

## What you CAN modify

`config.json` — the ONLY file you edit. Fields:

```json
{
  "swarm_size": 15,
  "math_count": 5,
  "bull_count": 5,
  "bear_count": 5,
  "max_tx": 300,
  "description": "what this experiment tests and why"
}
```

Constraint: `math_count + bull_count + bear_count == swarm_size`.

## What you CANNOT modify

- `run_experiment.py` — fixed evaluation harness. Read-only.
- The evaluator binary or any Rust source code.
- The model (always Pro/Qwen2.5-7B-Instruct).

## Metric: Effective Reasoning Score (ERS)

```
ERS = depth_norm × novelty × breadth_factor × proved_bonus
```

- **depth_norm** = min(max_depth, 15) / 15 — deeper chains = better
- **novelty** = unique_content_ratio — fraction of nodes with unique 40-char prefix
- **breadth_factor** = min(roots, 5) / 5 — multiple proof strategies
- **proved_bonus** = 1.5 if OMEGA reached, else 1.0

**Secondary metrics** (diagnostic, not in score):
- YES:NO ratio — market balance
- traded/nodes — market engagement
- capital_at_depth — where money flows

Run the experiment:

```
python3 run_experiment.py
```

It reads `config.json`, runs the evaluator, and prints:

```
---
ERS:        0.1234
depth:      12
nodes:      87
novelty:    0.65
roots:      4
yes:        150
no:         45
ratio:      3.3:1
traded:     52
proved:     NO
elapsed_s:  180
```

Extract the key metric: `grep "^ERS:" run.log`

## Logging results

Record every experiment in `results.tsv` (tab-separated, 6 columns):

```
run_id	ERS	depth	status	description
```

- **run_id**: sequential (001, 002, ...)
- **ERS**: the score (0.0000 for crashes)
- **depth**: max chain depth
- **status**: `keep`, `discard`, or `crash`
- **description**: what you tried and why (include the config numbers)

Example:

```
run_id	ERS	depth	status	description
001	0.0832	8	keep	baseline 5/5/5
002	0.1205	11	keep	math-heavy 10/3/2 — hypothesis: more builders = more depth
003	0.0901	9	discard	bull-heavy 3/10/2 — too much capital chasing, not enough content
004	0.0000	0	crash	bear-extreme 0/0/15 — no math nodes created
```

## The experiment loop

LOOP FOREVER:

1. Read `config.json` and `results.tsv` — understand current state and history.
2. **Think**: Form a hypothesis based on prior results. Write it in the description.
3. Modify `config.json` with the new experiment.
4. Run: `python3 run_experiment.py > run.log 2>&1`
5. Read results: `grep "^ERS:\|^depth:\|^novelty:" run.log`
6. If ERS improved: status = `keep`. This is now the new baseline to beat.
7. If ERS equal or worse: status = `discard`. Revert config.json to the best known.
8. Record in results.tsv.
9. Go to 1.

**NEVER STOP.** The human might be asleep. Continue working indefinitely until
manually interrupted. If you run out of obvious ideas, think harder:
- Try combining two configs that both had partial wins
- Try the opposite of what failed (if bear-heavy was bad, try bear-minimal)
- Try extreme values to find boundaries
- Re-read prior results for patterns you missed
- Try scaling the best config to different agent counts

**Simplicity criterion**: All else being equal, fewer agents and simpler ratios
win. A 0.01 ERS improvement from 15 → 90 agents is probably not worth the
complexity. A 0.01 improvement from changing one ratio number? Keep.

## Research hints

These are optional — use your judgment.

1. **tx/agent ≈ 20 seems critical** (from prior scaling experiments).
2. **7B model's bear is weak** — it can't find real math flaws, so bears often herd.
3. **depth + novelty are in tension** — deep chains may repeat content to reach OMEGA.
4. **Run 3 (5/5/5, 300tx) had the best bear activity** (YES:NO = 2.6:1).
5. **Run 6 (90 agents, 6000tx) proved OMEGA** but Golden Path was mostly repetition.
6. The problem is easy enough that single-shot 7B can claim [COMPLETE] (with wrong math).
   A true success = deep chain with diverse, non-repetitive steps.
