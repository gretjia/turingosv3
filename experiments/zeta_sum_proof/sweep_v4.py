#!/usr/bin/env python3
"""
TuringOS AutoResearch v4 — Karpathy-Faithful Agent Loop

"LLM IS the search algorithm." — Karpathy

Architecture:
  1. evaluator reads prompt from files (problem.txt, skill.txt, context.txt)
  2. DeepSeek V3 reads last experiment results + tape samples
  3. DeepSeek decides: edit a prompt file OR change a parameter
  4. Run experiment for FIXED 10 min wall clock
  5. ERS improves → keep (git-like advance), else → discard (rollback)
  6. NEVER STOP

The prompt files are our "train.py" — the single mutable artifact.
DeepSeek is our "researcher" — it reads results and proposes changes.
ERS is our "val_bpb" — single scalar truth.
"""

import subprocess, re, time, os, sys, json, shutil
from pathlib import Path
from datetime import datetime

PROJECT = Path("/home/zephryj/projects/turingosv3")
BINARY = PROJECT / "target/release/evaluator"
PROMPT_DIR = PROJECT / "experiments/zeta_sum_proof/prompt"
RESULTS = PROJECT / "experiments/zeta_sum_proof/audit/autoresearch_v4.tsv"
LOG_DIR = PROJECT / "experiments/zeta_sum_proof/logs/v4"
GT_LOG_DIR = Path("/tmp/turingos_zeta_logs")

# ── FIXED BUDGET (Karpathy: comparable) ──
WALL_CLOCK_SECS = 600  # 10 minutes. Fixed. No exceptions.

# ── DEFAULT CONFIG (params, NOT prompts — prompts are in files) ──
DEFAULT_CONFIG = {
    "SWARM_SIZE": 10, "MATH_COUNT": 6, "BULL_COUNT": 2, "BEAR_COUNT": 2,
    "FRONTIER_CAP": 30, "DEPTH_WEIGHT": 1.0, "PRICE_GATE_ALPHA": 0.05,
    "GLOBAL_DEDUP": "true", "THINKING_MODE": "off",
    "LLM_MODEL": "qwen3.5-9b",
    "LIBRARIAN_INTERVAL": 20,  # Must run within 10 min budget. Librarian = Ground Truth authority for depth.
}


def load_env():
    env = os.environ.copy()
    for line in (PROJECT / ".env").read_text().splitlines():
        line = line.strip()
        if '=' in line and not line.startswith('#'):
            k, v = line.split('=', 1)
            env[k.strip()] = v.strip()
    return env


def detect_endpoints():
    import urllib.request
    urls = []
    for port in [18080, 18081]:
        try:
            urllib.request.urlopen(f"http://127.0.0.1:{port}/health", timeout=2)
            urls.append(f"http://127.0.0.1:{port}")
        except: pass
    return urls


def run_experiment(label, config, base_env):
    """Run evaluator for exactly WALL_CLOCK_SECS."""
    for f in GT_LOG_DIR.glob("*.jsonl"):
        f.unlink()

    env = base_env.copy()
    env.update({"RUST_LOG": "info", "LLM_PROVIDER": "local",
                "LOG_DIR": str(GT_LOG_DIR), "MAX_TX": "999999",
                "PROMPT_DIR": str(PROMPT_DIR)})
    for k, v in config.items():
        env[k] = str(v)

    urls = detect_endpoints()
    if not urls: return "", 0
    env["LLM_URLS" if len(urls) > 1 else "LLM_URL"] = (
        ",".join(urls) if len(urls) > 1 else f"{urls[0]}/v1/chat/completions")

    start = time.time()
    try:
        r = subprocess.run([str(BINARY)], cwd=str(PROJECT),
            capture_output=True, text=True, timeout=WALL_CLOCK_SECS, env=env)
        output = r.stderr
    except subprocess.TimeoutExpired as e:
        output = (e.stderr or b"").decode(errors="replace")

    elapsed = time.time() - start
    LOG_DIR.mkdir(parents=True, exist_ok=True)
    (LOG_DIR / f"{label}.log").write_text(output)
    return output, elapsed


def compute_depth_from_log(log):
    """Extract depth. Librarian "deepest chain" is the AUTHORITATIVE source (Ground Truth).
    Fallback sources exist but are degraded — if Librarian didn't run, that's a config bug."""

    # PRIMARY: Librarian output (Ground Truth authority)
    depths = [int(d) for d in re.findall(r"deepest chain.*?(\d+) steps", log)]
    if depths:
        return max(depths)

    # FALLBACK 1: "Step N" in proof chain display (from Boltzmann chain)
    steps = [int(s) for s in re.findall(r"Step (\d+) \[Price:", log)]
    if steps:
        return max(steps)

    # FALLBACK 2: if nothing found, depth=0 (Librarian likely didn't run — check LIBRARIAN_INTERVAL)
    return 0


def compute_ers(log):
    """ERS = (depth/20)² × novelty × focus. Single scalar."""
    depth = compute_depth_from_log(log)
    appends = len(re.findall(r"Appended", log))
    dedup = len(re.findall(r"DEDUP\]|GLOBAL-DEDUP", log))
    novelty = max(0, appends - dedup) / max(appends, 1)
    frontiers = [int(f) for f in re.findall(r"from (\d+) frontier", log)]
    focus = min(30.0 / max(max(frontiers, default=1), 1), 1.0)
    proved = 1.5 if re.search(r"OMEGA|COMPLETE", log) else 1.0
    return round((min(depth, 20) / 20.0) ** 2 * novelty * focus * proved, 5)


def extract_summary(log):
    """Concise summary for LLM search agent."""
    appends = len(re.findall(r"Appended", log))
    frontiers = [int(f) for f in re.findall(r"from (\d+) frontier", log)]

    # Actual math content from tape
    samples = []
    sf = GT_LOG_DIR / "success.jsonl"
    if sf.exists():
        for line in sf.read_text().splitlines()[-30:]:
            try:
                d = json.loads(line)
                p = d.get("payload", "")
                if "Tool" not in p and len(p) > 30:
                    samples.append(p[:200])
            except: pass

    return {
        "appends": appends,
        "depth": compute_depth_from_log(log),
        "max_frontier": max(frontiers, default=0),
        "dedup": len(re.findall(r"DEDUP\]|GLOBAL-DEDUP", log)),
        "bankrupt": len(set(re.findall(r"Bankrupt.*?(Agent_\d+)", log))),
        "too_short": len(re.findall(r"too short", log)),
        "math_samples": samples[-5:],
    }


def save_prompt_snapshot(label):
    """Save current prompt files as snapshot (for rollback)."""
    snap = PROMPT_DIR / "snapshots" / label
    snap.mkdir(parents=True, exist_ok=True)
    for f in ["problem.txt", "skill.txt", "context.txt"]:
        src = PROMPT_DIR / f
        if src.exists():
            shutil.copy2(src, snap / f)


def rollback_prompt(label):
    """Restore prompt files from snapshot."""
    snap = PROMPT_DIR / "snapshots" / label
    if snap.exists():
        for f in ["problem.txt", "skill.txt", "context.txt"]:
            src = snap / f
            if src.exists():
                shutil.copy2(src, PROMPT_DIR / f)


def llm_research_step(history, config, base_env):
    """
    THE CORE: LLM IS the search algorithm.

    DeepSeek reads:
      - Last 8 experiment results (ERS, depth, samples)
      - Current prompt files (the artifact it can edit)
      - Current config params

    DeepSeek outputs:
      - EITHER a param change: {"action": "param", "param": "X", "value": Y, "reason": "..."}
      - OR a prompt edit: {"action": "edit", "file": "problem.txt", "content": "...", "reason": "..."}
    """
    ds_key = base_env.get("DEEPSEEK_API_KEY", "")
    if not ds_key:
        return "no_key", None

    # Read current prompt files
    prompts = {}
    for f in ["problem.txt", "skill.txt", "context.txt"]:
        p = PROMPT_DIR / f
        prompts[f] = p.read_text() if p.exists() else "(using default)"

    prompt = f"""You are an AutoResearch agent optimizing a multi-agent proof system.
The system uses 10 LLM agents to collaboratively prove 1+2+3+...=-1/12 via regularization.

YOUR METRIC: ERS = (depth/20)² × novelty × focus. MAXIMIZE THIS.
  depth² means: doubling proof depth is 4x more valuable than doubling anything else.

YOUR BUDGET: Each experiment runs for exactly 10 minutes. No exceptions.

CURRENT PROMPT FILES (you can EDIT these — they are the main lever):

--- problem.txt ---
{prompts['problem.txt'][:800]}

--- skill.txt ---
{prompts['skill.txt'][:500]}

--- context.txt ---
{prompts['context.txt'][:300]}

CURRENT PARAMS: {json.dumps(config)}

LAST {min(len(history), 8)} EXPERIMENTS:
"""
    for h in reversed(history[-8:]):
        s = h['summary']
        prompt += f"  [ERS={h['ers']:.5f}] depth={s['depth']} appends={s['appends']} "
        prompt += f"frontier={s['max_frontier']} bankrupt={s['bankrupt']} "
        prompt += f"dedup={s['dedup']} verdict={h['verdict']} change={h.get('change','baseline')}\n"
        if s.get('math_samples'):
            prompt += f"    math sample: \"{s['math_samples'][-1][:120]}\"\n"

    prompt += """
WHAT TO DO:
You must output ONE action as JSON. Choose the highest-impact change.

Option A — Edit a prompt file (HIGHEST IMPACT: directly changes agent behavior):
  {"action":"edit","file":"problem.txt","content":"FULL NEW CONTENT HERE","reason":"..."}

Option B — Change a parameter:
  {"action":"param","param":"THINKING_MODE","value":"budget:800","reason":"..."}

Tunable params: SWARM_SIZE, MATH_COUNT, BULL_COUNT, BEAR_COUNT, FRONTIER_CAP,
  DEPTH_WEIGHT, PRICE_GATE_ALPHA, GLOBAL_DEDUP, THINKING_MODE, LIBRARIAN_INTERVAL

RULES:
- Change ONE thing only.
- If depth is low, consider: are agents just restating conclusions? Edit problem.txt to demand computation.
- If too many duplicates, consider: is global dedup too strict? Or is the prompt too narrow?
- If agents bankrupt fast, consider: adjust role ratio or investment rules in skill.txt.
- Explain your reasoning in "reason".
- Output ONLY valid JSON.
"""

    import urllib.request
    req = urllib.request.Request(
        "https://api.deepseek.com/chat/completions",
        data=json.dumps({
            "model": "deepseek-chat", "temperature": 0.4, "max_tokens": 4000,
            "messages": [{"role": "user", "content": prompt}]
        }).encode(),
        headers={"Content-Type": "application/json",
                 "Authorization": f"Bearer {ds_key}"}
    )

    try:
        with urllib.request.urlopen(req, timeout=60) as resp:
            body = json.loads(resp.read())
            content = body["choices"][0]["message"]["content"]
            # Strip markdown code blocks
            content = re.sub(r'```json\s*', '', content)
            content = re.sub(r'```\s*', '', content)
            match = re.search(r'\{.*\}', content, re.DOTALL)
            if match:
                action = json.loads(match.group())
                return action.get("action", "param"), action
    except Exception as e:
        print(f"  LLM error: {e}", flush=True)

    return "error", None


def main():
    LOG_DIR.mkdir(parents=True, exist_ok=True)
    GT_LOG_DIR.mkdir(parents=True, exist_ok=True)
    PROMPT_DIR.mkdir(parents=True, exist_ok=True)
    base_env = load_env()

    if not BINARY.exists():
        print(f"ERROR: {BINARY} not found", file=sys.stderr); sys.exit(1)

    header = ["num", "time", "ers", "depth", "appends", "dedup", "frontier",
              "bankrupt", "elapsed", "verdict", "change", "config"]
    if not RESULTS.exists():
        RESULTS.parent.mkdir(parents=True, exist_ok=True)
        with open(RESULTS, "w") as f:
            f.write("\t".join(header) + "\n")

    best_ers = -1.0
    best_config = DEFAULT_CONFIG.copy()
    best_prompt_label = "baseline"
    history = []
    endpoints = detect_endpoints()

    print("=" * 60)
    print("TuringOS AutoResearch v4")
    print(f"  Metric: ERS = (depth/20)² × novelty × focus")
    print(f"  Budget: {WALL_CLOCK_SECS}s wall clock (FIXED)")
    print(f"  Search: DeepSeek V3 (LLM IS the search algorithm)")
    print(f"  Mutable: prompt files + config params")
    print(f"  Endpoints: {endpoints}")
    print("=" * 60, flush=True)

    # ── BASELINE ──
    save_prompt_snapshot("baseline")
    print(f"\n[0] BASELINE", flush=True)
    log, elapsed = run_experiment("exp000_baseline", best_config, base_env)
    ers = compute_ers(log)
    summary = extract_summary(log)
    best_ers = ers
    history.append({"ers": ers, "summary": summary, "verdict": "BASELINE", "change": "baseline"})
    print(f"  ERS={ers:.5f} depth={summary['depth']} appends={summary['appends']} "
          f"frontier={summary['max_frontier']} elapsed={elapsed:.0f}s", flush=True)
    if summary['math_samples']:
        print(f"  sample: {summary['math_samples'][-1][:120]}", flush=True)

    with open(RESULTS, "a") as f:
        f.write("\t".join(str(x) for x in [
            0, datetime.now().isoformat(), ers, summary['depth'], summary['appends'],
            summary['dedup'], summary['max_frontier'], summary['bankrupt'],
            f"{elapsed:.0f}", "BASELINE", "baseline", json.dumps(best_config)
        ]) + "\n")

    # ── AGENT LOOP — NEVER STOP ──
    exp_num = 0
    while True:
        exp_num += 1

        # LLM decides what to try (THE CORE INSIGHT)
        action_type, action = llm_research_step(history, best_config, base_env)

        if action is None:
            print(f"\n[{exp_num}] LLM returned no action, retrying...", flush=True)
            time.sleep(5); continue

        # Save current state for rollback
        save_prompt_snapshot(f"pre_exp{exp_num:03d}")
        new_config = best_config.copy()
        change_desc = ""

        if action_type == "edit":
            # LLM edited a prompt file
            fname = action.get("file", "problem.txt")
            content = action.get("content", "")
            reason = action.get("reason", "")
            if content and fname in ["problem.txt", "skill.txt", "context.txt"]:
                (PROMPT_DIR / fname).write_text(content)
                change_desc = f"edit:{fname} ({reason[:60]})"
            else:
                change_desc = f"invalid_edit:{fname}"
        elif action_type == "param":
            param = action.get("param", "")
            value = action.get("value", "")
            reason = action.get("reason", "")
            if param:
                new_config[param] = value
                change_desc = f"{param}={value} ({reason[:60]})"
        else:
            change_desc = f"unknown:{action_type}"

        label = f"exp{exp_num:03d}_{re.sub(r'[^a-zA-Z0-9_=.]', '_', change_desc[:40])}"
        print(f"\n[{exp_num}] {change_desc}", flush=True)

        log, elapsed = run_experiment(label, new_config, base_env)
        ers = compute_ers(log)
        summary = extract_summary(log)

        # GREEDY: improve → keep, else → discard + rollback
        if ers > best_ers:
            verdict = "KEEP"
            prev_best = best_ers
            best_ers = ers
            best_config = new_config
            best_prompt_label = f"exp{exp_num:03d}"
            save_prompt_snapshot(best_prompt_label)
            print(f"  ▲ ERS={ers:.5f} (was {prev_best:.5f}) KEEP", flush=True)
        else:
            verdict = "DISCARD"
            # Rollback prompt files if they were edited
            if action_type == "edit":
                rollback_prompt(f"pre_exp{exp_num:03d}")
            print(f"  ▼ ERS={ers:.5f} (best={best_ers:.5f}) DISCARD", flush=True)

        history.append({"ers": ers, "summary": summary, "verdict": verdict, "change": change_desc})

        print(f"  depth={summary['depth']} appends={summary['appends']} "
              f"frontier={summary['max_frontier']} bankrupt={summary['bankrupt']} "
              f"elapsed={elapsed:.0f}s", flush=True)
        if summary['math_samples']:
            print(f"  sample: {summary['math_samples'][-1][:120]}", flush=True)

        with open(RESULTS, "a") as f:
            f.write("\t".join(str(x) for x in [
                exp_num, datetime.now().isoformat(), ers, summary['depth'],
                summary['appends'], summary['dedup'], summary['max_frontier'],
                summary['bankrupt'], f"{elapsed:.0f}", verdict, change_desc,
                json.dumps(new_config)
            ]) + "\n")


if __name__ == "__main__":
    main()
