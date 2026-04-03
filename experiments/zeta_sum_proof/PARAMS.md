# ζ-Sum Proof — Tunable Parameters (AutoResearch)

All parameters configurable via environment variables for `sweep.py` tuning.

## Boltzmann Selection (actor.rs)

| Param | Env Var | Default | Source | Rationale |
|-------|---------|---------|--------|-----------|
| Frontier cap | `FRONTIER_CAP` | 30 | DeepSeek econ audit 2026-04-02 §6.9 | 190 nodes = over-dilution. Cap forces focus. 30 ≈ 2×swarm_size |
| Depth weight exponent | `DEPTH_WEIGHT` | 1.0 | DeepSeek econ audit §6.8 | score × log(depth+1)^DEPTH_WEIGHT. 0=no depth bias, 1=log, 2=log² |
| Price Gate alpha | `PRICE_GATE_ALPHA` | 0.05 | DeepSeek econ audit §6.2 | child must exceed parent×(1+α/depth) to mask. Higher α = stickier parents |
| Boltzmann temperature | (per-agent in evaluator) | 0.2-0.8 | CLAUDE.md Layer 2 §7 | T=0→greedy, T=∞→uniform |

## Deduplication (bus.rs)

| Param | Env Var | Default | Source | Rationale |
|-------|---------|---------|--------|-----------|
| Branch dedup prefix | (hardcoded) | 40 chars | bus.rs Phase 3b, Gemini audit 2026-03-30 | 40 char normalized prefix for same-branch |
| Global dedup | `GLOBAL_DEDUP` | true | DeepSeek econ audit §6.3 + architect insight 2026-04-02 | Cross-branch exact-match rejection. 754 branch dedup + unknown cross-branch waste |

## Librarian (librarian.rs)

| Param | Env Var | Default | Source | Rationale |
|-------|---------|---------|--------|-----------|
| Compress interval | `LIBRARIAN_INTERVAL` | 100 | architect directive 2026-04-02 | Appends between compressions. 50 for testing |
| Log directory | `LOG_DIR` | /tmp/turingos_zeta_logs | architect "Ground Truth" directive | Persistent JSONL logs |

## Thinking Mode (llm_http.rs)

| Param | Env Var | Default | Source | Rationale |
|-------|---------|---------|--------|-----------|
| Thinking mode | `THINKING_MODE` | off | Root cause analysis 2026-04-03 | "on"=full thinking (high quality, ~60s/req), "off"=/no_think (terse, ~5s/req), "budget:N"=capped |

Research question: Qwen3.5 thinking tokens are the model's "scratch pad" for algebra.
- ON: generates 1000+ hidden tokens of derivation → output has detailed equations
- OFF: skips thinking → output is terse descriptions ("substitute w and expand")
- BUDGET:N: thinking + output share N total tokens → partial thinking

Tradeoff: quality (depth of proof) vs speed (experiments per hour for AutoResearch).
This parameter lets AutoResearch empirically determine the optimal balance.

## Swarm (evaluator.rs)

| Param | Env Var | Default | Source | Rationale |
|-------|---------|---------|--------|-----------|
| Swarm size | `SWARM_SIZE` | 15 | CLAUDE.md Layer 2 §6 | Total agents |
| Math/Bull/Bear split | `MATH_COUNT` / `BULL_COUNT` / `BEAR_COUNT` | 5/5/5 | architect 2026-04-01 | Role trifecta |
| LLM provider | `LLM_PROVIDER` | aliyun | Layer 2 | aliyun / siliconflow |
| LLM model | `LLM_MODEL` | qwen3-8b | Layer 2 §9 | Worker model |
| Max transactions | `MAX_TX` | ∞ | sweep.py budget | 0 = unlimited |
