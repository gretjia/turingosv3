# AutoResearch Self-Tuning Plan for ζ-Sum Proof

Inspired by [karpathy/autoresearch](https://github.com/karpathy/autoresearch).
Core insight: LLM IS the search algorithm. System provides fast, honest feedback loops.

## Architecture

```
omega-vm (orchestrator)
  ├── sweep.py           — AutoResearch loop
  ├── evaluator binary   — runs each experiment
  ├── results.tsv        — append-only experiment log (Ground Truth)
  └── ssh → local LLMs
        ├── zephrymac-studio:11434  — Ollama (qwen2.5:7b, ~50 tok/s)
        └── windows1-w1:11434      — Ollama (qwen2.5:7b, to install)
```

## Single Scalar Metric: ERS (Effective Reasoning Score)

From existing sweep.py, enhanced with depth emphasis:

```
ERS = depth² × novelty × breadth_factor × (1 + proved_bonus)

where:
  depth = max_chain_depth / 15          # normalized, SQUARED to reward depth
  novelty = unique_40char / total_nodes  # global dedup effectiveness
  breadth_factor = min(roots, 5) / 5    # multiple proof strategies
  proved_bonus = 0.5 if [OMEGA]         # 50% bonus for completion
```

Depth is squared (not linear) because our primary goal is increasing proof depth.

## Experiment Loop (Karpathy-style)

```python
# sweep_v3.py — AutoResearch self-tuning loop
# Each experiment = one evaluator run with fixed budget

BUDGET_TX = 200          # fixed budget per experiment (comparable)
TIMEOUT_SECS = 600       # 10 min wall clock max

# Parameter space (all env vars, see PARAMS.md)
PARAMS = {
    "FRONTIER_CAP":      [10, 20, 30, 50, 0],       # 0 = unlimited
    "DEPTH_WEIGHT":      [0.0, 0.5, 1.0, 1.5, 2.0],
    "PRICE_GATE_ALPHA":  [0.0, 0.02, 0.05, 0.10],
    "GLOBAL_DEDUP":      ["true", "false"],
    "MATH_COUNT":        [5, 8, 10, 15],
    "BULL_COUNT":        [0, 2, 5],
    "BEAR_COUNT":        [0, 2, 5],
    "SWARM_SIZE":        [10, 15, 20],
}

for experiment in generate_experiments():
    # 1. Set env vars
    # 2. Run evaluator with MAX_TX=200
    # 3. Parse tape → compute ERS
    # 4. Append to results.tsv (Ground Truth, never delete)
    # 5. If ERS > best_ers: advance (keep params)
    #    Else: discard (revert to previous best)
```

## Abandon Strategy

### Parameter-level abandon (automated)
If a parameter value produces ERS < baseline in 3 consecutive experiments
involving that value, mark it as "dead" and exclude from future sweeps.

### Mechanism-level abandon (requires architect review)
If a mechanism (e.g., Global Dedup, Price Gate) produces WORSE results
than its disabled variant in 5+ experiments, flag for architect review:
  "Global Dedup appears harmful — ERS=X with vs ERS=Y without. Disable?"

Results logged to `audit/abandon_log.tsv` with full evidence.

## Local LLM Setup

### Phase 1: Mac Studio (immediate)
```bash
# On omega-vm, SSH tunnel to Mac's Ollama
ssh -L 11434:127.0.0.1:11434 zephrymac-studio "ollama serve"

# Evaluator config
LLM_PROVIDER=ollama
LLM_URL=http://127.0.0.1:11434/v1/chat/completions
LLM_MODEL=qwen2.5:7b
```

Advantage: No rate limits. ~50 tok/s. ~3x faster than DashScope with 429s.

### Phase 2: Windows1 (requires setup)
```bash
# Install Ollama on Windows1
ssh windows1-w1 "winget install Ollama.Ollama"
ssh windows1-w1 "ollama pull qwen2.5:7b"

# Second tunnel
ssh -L 11435:127.0.0.1:11434 windows1-w1 "ollama serve"

# Use both endpoints for heterogeneous swarm
LLM_PROVIDER=multi
LLM_URLS=http://127.0.0.1:11434,http://127.0.0.1:11435
```

128GB can run 70B models — potential for stronger Mathematician agents.

### Phase 3: Heterogeneous swarm
- Mathematicians → windows1 (qwen2.5:72b or deepseek-v3 local)
- Bulls/Bears → mac (qwen2.5:7b, fast + cheap)
- Librarian → DeepSeek API (cloud, strongest model, low frequency)

## Implementation Priority

1. **Start Ollama on Mac** (5 min) — immediate 429-free local inference
2. **Wire evaluator to Ollama endpoint** (code change) — add `ollama` provider
3. **Run baseline ERS** with current params on local inference
4. **Implement sweep_v3.py** with parameter grid + abandon logic
5. **Install Ollama on Windows1** (when Mac baseline stable)
6. **Heterogeneous swarm** (Phase 3)

## Files to Create/Modify

| File | Change |
|------|--------|
| `src/drivers/llm_http.rs` | Add Ollama endpoint support (if needed) |
| `experiments/zeta_sum_proof/sweep_v3.py` | AutoResearch self-tuning loop |
| `experiments/zeta_sum_proof/PARAMS.md` | Already created |
| `experiments/zeta_sum_proof/audit/results.tsv` | Append-only experiment log |
| `experiments/zeta_sum_proof/audit/abandon_log.tsv` | Mechanism abandon decisions |
