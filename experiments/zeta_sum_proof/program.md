# TuringOS Model Scaling Research — program.md

> "LLM IS the search algorithm." — Karpathy
> You are Claude Code. You ARE the researcher. Not a script, not a function — you.

## Mission

Find the minimum model scale at which TuringOS's Hayekian prediction market produces effective proof depth WITHOUT artificial parameter hacks (DEPTH_WEIGHT=0, FRONTIER_CAP=0, PRICE_GATE_ALPHA=0).

## Your Lab

- **evaluator binary**: `target/release/evaluator` (omega-vm) or deploy to Mac/linux1
- **prompt files**: `experiments/zeta_sum_proof/prompt/` (problem.txt, skill.txt, context.txt — all locked)
- **API keys**: `.env` (DashScope, SiliconFlow, DeepSeek, Volcengine, NVIDIA NIM)
- **results**: `experiments/zeta_sum_proof/audit/model_scaling_results.tsv`
- **research notes**: `experiments/zeta_sum_proof/prompt/research_notes.txt`
- **logs**: `experiments/zeta_sum_proof/logs/scaling/`

## Available Models (smallest → largest)

| Model | Params | Provider | API Model ID |
|-------|--------|----------|-------------|
| Qwen3-0.6B | 0.6B | DashScope | qwen3-0.6b |
| Qwen3-1.7B | 1.7B | DashScope | qwen3-1.7b |
| Qwen3-4B | 4B | DashScope | qwen3-4b |
| Qwen3-8B | 8B | DashScope | qwen3-8b |
| Qwen3-14B | 14B | DashScope | qwen3-14b |
| Qwen3-32B | 32B | DashScope | qwen3-32b |
| Qwen2.5-72B | 72B | DashScope | qwen2.5-72b-instruct |
| Qwen3.5-122B | 122B MoE | SiliconFlow | Qwen/Qwen3.5-122B-A10B |
| Qwen3.5-397B | 397B MoE | SiliconFlow | Qwen/Qwen3.5-397B-A17B |
| DeepSeek V3.2 | 670B MoE | DeepSeek | deepseek-chat |

Cross-check (non-Qwen): Gemma-3-4B, Llama-3.3-70B, DeepSeek-R1-Distill-7B/32B

## How to Run One Experiment

```bash
# Set provider + model, run evaluator for 600s
DEEPSEEK_API_KEY=... DASHSCOPE_API_KEY=... \
RUST_LOG=info LLM_PROVIDER=aliyun LLM_MODEL=qwen3-14b \
LLM_URL=https://dashscope.aliyuncs.com/compatible-mode/v1/chat/completions \
SWARM_SIZE=15 MATH_COUNT=9 BULL_COUNT=3 BEAR_COUNT=3 \
FRONTIER_CAP=0 DEPTH_WEIGHT=0 PRICE_GATE_ALPHA=0 \
GLOBAL_DEDUP=true THINKING_MODE=off LIBRARIAN_INTERVAL=8 \
LOG_DIR=/tmp/turingos_zeta_logs PROMPT_DIR=experiments/zeta_sum_proof/prompt \
MAX_TX=999999 \
timeout 600 ./target/release/evaluator 2>&1 | tee experiments/zeta_sum_proof/logs/scaling/MODEL_NAME.log
```

## How to Analyze Results

After each run, extract:
```bash
grep -c APMM log           # real DAG nodes
grep -c Appended log        # total appends (includes invests)
grep -c "BUY YES" log       # YES investments
grep -c "BUY NO" log        # NO investments (shorts)
grep "LIBRARIAN.*STATS" log # depth progression
grep -c AUTOPSY log         # bankruptcy events
```

Key metric: **max_depth** from Librarian STATS. Secondary: real nodes/min.

## Research Strategy

You decide everything:
- Which model to test next
- Whether to skip ahead or go sequential
- When the pattern is clear enough to stop
- Whether to retest with longer WALL_CLOCK
- Whether to try non-Qwen models for cross-validation

Write your reasoning in research_notes.txt before each experiment.

## Constitutional Controls (fixed, do not change)

- FRONTIER_CAP=0, DEPTH_WEIGHT=0, PRICE_GATE_ALPHA=0
- THINKING_MODE=off (for all swarm agents)
- SWARM_SIZE=15, MATH/BULL/BEAR = 9/3/3
- LIBRARIAN_INTERVAL=8
- Same problem.txt, skill.txt, context.txt for every model

## NEVER STOP

Run experiments until you find the sweet spot. Write your findings. The human might be asleep.
