#!/usr/bin/env python3
"""
TuringOS AutoResearch Sweep v3 — Self-Tuning Loop
Inspired by karpathy/autoresearch: NEVER STOP. LLM is the search algorithm.
System provides fast, honest feedback.

METRIC: ERS_v2 (depth-emphasized)
  = depth² × novelty × breadth_factor × (1 + 0.5 × proved)
  depth is SQUARED to strongly reward proof depth over breadth.

STRATEGY: Greedy hill-climb with rollback.
  - If ERS improves → keep new params (advance)
  - If ERS same or worse → discard (rollback to best)
  - If a param value fails 3× → abandon it
  - If a mechanism is worse than disabled in 5× → flag for architect

PARAMS: All via env vars (see PARAMS.md)
  FRONTIER_CAP, DEPTH_WEIGHT, PRICE_GATE_ALPHA, GLOBAL_DEDUP,
  MATH_COUNT, BULL_COUNT, BEAR_COUNT, SWARM_SIZE

INFERENCE: Local llama.cpp (no rate limits, no cost)
  LLM_PROVIDER=local, LLM_URLS=mac:18080,win1:18081
"""

import subprocess, re, time, os, sys, random, json
from pathlib import Path
from collections import defaultdict
from datetime import datetime

PROJECT = Path("/home/zephryj/projects/turingosv3")
BINARY = PROJECT / "target/release/evaluator"
RESULTS = PROJECT / "experiments/zeta_sum_proof/audit/autoresearch_v3.tsv"
ABANDON_LOG = PROJECT / "experiments/zeta_sum_proof/audit/abandon_log.tsv"
LOG_DIR = PROJECT / "experiments/zeta_sum_proof/logs/autoresearch_v3"
GT_LOG_DIR = Path("/tmp/turingos_zeta_logs")  # Ground Truth logs

# ── Fixed budget per experiment (Karpathy: comparable experiments) ──
# Source: AUTORESEARCH_PLAN.md — 200 tx, 10 min wall clock max
BUDGET_TX = 200       # tx limit per experiment
TIMEOUT_SECS_BASE = 900    # 15 min for no_think mode
TIMEOUT_SECS_THINK = 3600  # 60 min for thinking mode (12x slower)

# ── Parameter space (all env vars, see PARAMS.md) ──
# Each value list = candidates for sweep. First value = default.
PARAM_SPACE = {
    # Economic mechanisms (actor.rs)
    "FRONTIER_CAP":     [30, 15, 50, 0],           # 0 = unlimited. Source: PARAMS.md
    "DEPTH_WEIGHT":     [1.0, 0.0, 0.5, 1.5, 2.0], # Source: DeepSeek econ audit §6.8
    "PRICE_GATE_ALPHA": [0.05, 0.0, 0.02, 0.10],   # Source: DeepSeek econ audit §6.2
    "GLOBAL_DEDUP":     ["true", "false"],           # Source: DeepSeek econ audit §6.3

    # Role mix (evaluator.rs)
    "MATH_COUNT":       [6, 4, 8, 10],              # Source: sweep_v2 winner = math-heavy
    "BULL_COUNT":       [2, 0, 4],
    "BEAR_COUNT":       [2, 0, 3],

    # Thinking mode (llm_http.rs) — quality vs speed tradeoff
    # Source: 2026-04-03 root cause analysis
    #   "on"        → full thinking, high quality algebra, ~60s/req (9B local)
    #   "off"       → no thinking, terse output, ~5s/req
    #   "budget:N"  → thinking capped at N total tokens (thinking+output share budget)
    # This is a RESEARCH QUESTION: does thinking help proof depth?
    "THINKING_MODE":    ["off", "on", "budget:800", "budget:1500"],

    # Model selection (evaluator.rs) — available on Mac
    # Source: Mac has qwen3.5-9b (5.3GB) and qwen3.5-4b (2.6GB)
    #   9B: stronger math reasoning, slower
    #   4B: weaker but 2x faster → more experiments per hour
    # "LLM_MODEL":      ["qwen3.5-9b", "qwen3.5-4b"],  # uncomment when 4B server available
}

# ── Fixed params (not swept) ──
FIXED = {
    "SWARM_SIZE": "10",
    "LLM_PROVIDER": "local",
    "LLM_MODEL": "qwen3.5-9b",
    "LIBRARIAN_INTERVAL": "50",
    "LOG_DIR": str(GT_LOG_DIR),
    "RUST_LOG": "info",
    "MAX_TX": str(BUDGET_TX),
}


def load_env():
    env = os.environ.copy()
    env_file = PROJECT / ".env"
    for line in env_file.read_text().splitlines():
        line = line.strip()
        if '=' in line and not line.startswith('#'):
            k, v = line.split('=', 1)
            env[k.strip()] = v.strip()
    return env


def run_experiment(label, params, base_env):
    """Run one evaluator experiment. Returns (log_text, elapsed_secs)."""
    # Clear Ground Truth logs for fresh experiment
    for f in GT_LOG_DIR.glob("*.jsonl"):
        f.unlink()

    env = base_env.copy()
    env.update(FIXED)
    env.update(params)

    # Dynamic timeout: thinking mode needs much longer
    # Source: 2026-04-03 — thinking ON = ~60s/req vs OFF = ~5s/req
    thinking = params.get("THINKING_MODE", "off")
    timeout = TIMEOUT_SECS_THINK if thinking != "off" else TIMEOUT_SECS_BASE

    # Detect available LLM endpoints
    urls = []
    for port in [18080, 18081]:
        try:
            import urllib.request
            r = urllib.request.urlopen(f"http://127.0.0.1:{port}/health", timeout=2)
            if r.status == 200:
                urls.append(f"http://127.0.0.1:{port}")
        except:
            pass
    if not urls:
        print("  ERROR: No llama-server endpoints available!", flush=True)
        return "", 0
    if len(urls) > 1:
        env["LLM_URLS"] = ",".join(urls)
    else:
        env["LLM_URL"] = f"{urls[0]}/v1/chat/completions"
    print(f"  Endpoints: {urls}", flush=True)

    start = time.time()
    try:
        result = subprocess.run(
            [str(BINARY)], cwd=str(PROJECT),
            capture_output=True, text=True, timeout=timeout, env=env
        )
        output = result.stderr
    except subprocess.TimeoutExpired as e:
        output = (e.stderr or b"").decode(errors="replace")
        output += "\n[TIMEOUT]"

    elapsed = time.time() - start

    # Save log
    LOG_DIR.mkdir(parents=True, exist_ok=True)
    log_file = LOG_DIR / f"{label}_{int(time.time())}.log"
    log_file.write_text(output)
    return output, elapsed


def parse_ground_truth():
    """Parse Ground Truth logs (success.jsonl + failure.jsonl) → metrics."""
    success_file = GT_LOG_DIR / "success.jsonl"
    failure_file = GT_LOG_DIR / "failure.jsonl"

    nodes = {}  # node_id → payload
    if success_file.exists():
        for line in success_file.read_text().splitlines():
            try:
                d = json.loads(line)
                nodes[d["node_id"]] = d.get("payload", "")
            except:
                pass

    failures = defaultdict(int)
    if failure_file.exists():
        for line in failure_file.read_text().splitlines():
            try:
                d = json.loads(line)
                reason = d.get("reason", "unknown")
                if "too short" in reason:
                    failures["too_short"] += 1
                elif "Duplicate" in reason:
                    failures["dedup"] += 1
                elif "Bankrupt" in reason:
                    failures["bankrupt"] += 1
                else:
                    failures["other"] += 1
            except:
                pass

    return nodes, failures


def parse_log_metrics(log):
    """Extract metrics from evaluator log."""
    m = {}
    m['appends'] = len(re.findall(r"Appended", log))
    m['buy_yes'] = len(re.findall(r"BUY YES\]", log))
    m['buy_no'] = len(re.findall(r"BUY NO\]", log))
    m['rejected'] = len(re.findall(r"REJECTED", log))
    m['dedup'] = len(re.findall(r"DEDUP\]", log))
    m['global_dedup'] = len(re.findall(r"GLOBAL-DEDUP\]", log))
    m['proved'] = bool(re.search(r"Proof chain COMPLETE|OMEGA", log))
    m['librarian'] = len(re.findall(r"Memory written", log))

    # Max frontier size
    frontiers = re.findall(r"from (\d+) frontier", log)
    m['max_frontier'] = max((int(f) for f in frontiers), default=0)

    # Depth from librarian memory
    depth_match = re.findall(r"deepest chain.*?(\d+) steps", log)
    m['depth'] = max((int(d) for d in depth_match), default=0)

    # Bankrupt count
    bankrupt = set(re.findall(r"Bankrupt.*?(Agent_\d+)", log))
    m['bankrupt'] = len(bankrupt)

    return m


def compute_ers_v2(log_metrics):
    """
    ERS v2 = depth² × novelty × breadth × proved_bonus

    depth² because our PRIMARY goal is increasing proof depth.
    Source: DeepSeek math audit — "11 steps is circular, not progressive"
    """
    depth = min(log_metrics.get('depth', 0), 20) / 20.0
    appends = max(log_metrics.get('appends', 0), 1)
    dedup_total = log_metrics.get('dedup', 0) + log_metrics.get('global_dedup', 0)
    novelty = max(0, appends - dedup_total) / appends  # how much is truly new
    proved_bonus = 1.5 if log_metrics.get('proved') else 1.0
    # Breadth approximation from max frontier (smaller = more focused = better)
    max_f = max(log_metrics.get('max_frontier', 1), 1)
    focus = min(30.0 / max_f, 1.0)  # penalize frontier > 30

    ers = (depth ** 2) * novelty * focus * proved_bonus
    return round(ers, 5)


def default_params():
    """First value in each PARAM_SPACE list = default."""
    return {k: str(v[0]) for k, v in PARAM_SPACE.items()}


def mutate_params(current, dead_values):
    """Mutate one random param to a random non-dead value."""
    mutable = list(PARAM_SPACE.keys())
    random.shuffle(mutable)

    for key in mutable:
        candidates = [str(v) for v in PARAM_SPACE[key]]
        alive = [v for v in candidates
                 if (key, v) not in dead_values and v != current.get(key)]
        if alive:
            new_val = random.choice(alive)
            new_params = current.copy()
            new_params[key] = new_val
            return new_params, key, new_val

    # All params exhausted — try double mutation
    return current.copy(), None, None


def main():
    LOG_DIR.mkdir(parents=True, exist_ok=True)
    GT_LOG_DIR.mkdir(parents=True, exist_ok=True)
    base_env = load_env()

    if not BINARY.exists():
        print(f"ERROR: {BINARY} not found. Run: cargo build --release", file=sys.stderr)
        sys.exit(1)

    # ── State ──
    best_ers = -1.0
    best_params = default_params()
    dead_values = set()   # (param_name, value) → abandoned
    fail_counts = defaultdict(int)  # (param_name, value) → consecutive failures
    experiment_num = 0

    # TSV header
    header = ["num", "timestamp", "label", "ers", "depth", "appends", "rejected",
              "dedup", "global_dedup", "frontier", "bankrupt", "librarian",
              "proved", "elapsed_s", "verdict", "mutated_param", "mutated_value",
              "params_json"]
    if not RESULTS.exists():
        RESULTS.parent.mkdir(parents=True, exist_ok=True)
        with open(RESULTS, "w") as f:
            f.write("\t".join(header) + "\n")

    print("=" * 70)
    print("TuringOS AutoResearch v3 — Self-Tuning Loop")
    print(f"Metric: ERS_v2 = depth² × novelty × focus × proved_bonus")
    print(f"Budget: {BUDGET_TX} tx, {TIMEOUT_SECS_BASE}s/{TIMEOUT_SECS_THINK}s timeout (off/think)")
    print(f"Params: {len(PARAM_SPACE)} tunable, {sum(len(v) for v in PARAM_SPACE.values())} values")
    print(f"Strategy: greedy hill-climb + rollback + abandon")
    print("=" * 70, flush=True)

    # ── BASELINE ──
    print(f"\n[0] BASELINE — default params", flush=True)
    log, elapsed = run_experiment("baseline", best_params, base_env)
    log_m = parse_log_metrics(log)
    best_ers = compute_ers_v2(log_m)
    print(f"  → ERS={best_ers:.5f} depth={log_m['depth']} appends={log_m['appends']} "
          f"frontier={log_m['max_frontier']} bankrupt={log_m['bankrupt']} "
          f"elapsed={elapsed:.0f}s", flush=True)

    with open(RESULTS, "a") as f:
        row = [0, datetime.now().isoformat(), "baseline", best_ers, log_m['depth'],
               log_m['appends'], log_m['rejected'], log_m['dedup'], log_m['global_dedup'],
               log_m['max_frontier'], log_m['bankrupt'], log_m['librarian'],
               log_m['proved'], f"{elapsed:.0f}", "BASELINE", "", "",
               json.dumps(best_params)]
        f.write("\t".join(str(x) for x in row) + "\n")

    # ── SWEEP LOOP — NEVER STOP ──
    while True:
        experiment_num += 1
        new_params, mutated_key, mutated_val = mutate_params(best_params, dead_values)

        if mutated_key is None:
            print(f"\n[{experiment_num}] All param values exhausted. Resetting dead list.", flush=True)
            dead_values.clear()
            fail_counts.clear()
            continue

        label = f"exp{experiment_num}_{mutated_key}={mutated_val}"
        print(f"\n[{experiment_num}] {label}", flush=True)
        print(f"  Mutated: {mutated_key} = {best_params.get(mutated_key)} → {mutated_val}", flush=True)

        log, elapsed = run_experiment(label, new_params, base_env)
        log_m = parse_log_metrics(log)
        ers = compute_ers_v2(log_m)

        # ── KEEP / DISCARD ──
        if ers > best_ers:
            verdict = "KEEP"
            improvement = ers - best_ers
            best_ers = ers
            best_params = new_params
            # Reset fail count for this value
            fail_counts[(mutated_key, mutated_val)] = 0
            print(f"  → ERS={ers:.5f} (+{improvement:.5f}) KEEP ✓", flush=True)
        else:
            verdict = "DISCARD"
            fail_counts[(mutated_key, mutated_val)] += 1
            fc = fail_counts[(mutated_key, mutated_val)]
            print(f"  → ERS={ers:.5f} (best={best_ers:.5f}) DISCARD (fail #{fc})", flush=True)

            # ── ABANDON CHECK ──
            if fc >= 3:
                dead_values.add((mutated_key, mutated_val))
                print(f"  ☠ ABANDONED: {mutated_key}={mutated_val} (failed 3× consecutive)", flush=True)
                with open(ABANDON_LOG, "a") as af:
                    af.write(f"{datetime.now().isoformat()}\t{mutated_key}\t{mutated_val}\t"
                             f"failed_3x\ters={ers:.5f}\tbest={best_ers:.5f}\n")

        # ── LOG ──
        print(f"  depth={log_m['depth']} appends={log_m['appends']} "
              f"frontier={log_m['max_frontier']} bankrupt={log_m['bankrupt']} "
              f"dedup={log_m['dedup']}+{log_m['global_dedup']}g elapsed={elapsed:.0f}s", flush=True)

        with open(RESULTS, "a") as f:
            row = [experiment_num, datetime.now().isoformat(), label, ers, log_m['depth'],
                   log_m['appends'], log_m['rejected'], log_m['dedup'], log_m['global_dedup'],
                   log_m['max_frontier'], log_m['bankrupt'], log_m['librarian'],
                   log_m['proved'], f"{elapsed:.0f}", verdict, mutated_key, mutated_val,
                   json.dumps(new_params)]
            f.write("\t".join(str(x) for x in row) + "\n")

        # Status
        alive_count = sum(len(v) for v in PARAM_SPACE.values()) - len(dead_values)
        print(f"  [STATE] best_ers={best_ers:.5f} alive_values={alive_count} "
              f"dead={len(dead_values)}", flush=True)


if __name__ == "__main__":
    main()
