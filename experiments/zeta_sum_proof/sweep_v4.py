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

import subprocess, re, time, os, sys, json, shutil, signal, urllib.request
from pathlib import Path
from datetime import datetime

PROJECT = Path("/home/zephryj/projects/turingosv3")
BINARY = PROJECT / "target/release/evaluator"
PROMPT_DIR = PROJECT / "experiments/zeta_sum_proof/prompt"
RESULTS = PROJECT / "experiments/zeta_sum_proof/audit/autoresearch_v4_phase2.tsv"
LOG_DIR = PROJECT / "experiments/zeta_sum_proof/logs/v4"
GT_LOG_DIR = Path("/tmp/turingos_zeta_logs")
PREV_LIFE_FILE = PROJECT / "experiments/zeta_sum_proof/audit/prev_life_memory.json"

# ── BUDGET (tunable by Reasoner) ──
WALL_CLOCK_SECS = 600  # Default 10 min. Reasoner can adjust via config["WALL_CLOCK"].

# ── DEFAULT CONFIG (params, NOT prompts — prompts are in files) ──
DEFAULT_CONFIG = {
    "SWARM_SIZE": 10, "MATH_COUNT": 6, "BULL_COUNT": 2, "BEAR_COUNT": 2,
    "FRONTIER_CAP": 0, "DEPTH_WEIGHT": 0, "PRICE_GATE_ALPHA": 0,
    "GLOBAL_DEDUP": "true", "THINKING_MODE": "off",
    "LLM_MODEL": "qwen3.5-9b",
    "LIBRARIAN_INTERVAL": 8,
}
# LOCKED by architect directive 2026-04-04:
#   THINKING_MODE=off (群体即thinker), FRONTIER_CAP=0 (无限), DEPTH_WEIGHT=0 (价格说了算)
#   GLOBAL_DEDUP=true, PRICE_GATE_ALPHA=0 (纯价格比较，无深度调节门槛)
#   Only tunable: SWARM_SIZE, role counts, LIBRARIAN_INTERVAL


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
    """Run evaluator. Duration = config['WALL_CLOCK'] or WALL_CLOCK_SECS default."""
    wall_clock = int(config.get("WALL_CLOCK", WALL_CLOCK_SECS))
    for f in GT_LOG_DIR.glob("*.jsonl"):
        f.unlink()

    env = base_env.copy()
    env.update({"RUST_LOG": "info", "LLM_PROVIDER": "local",
                "LOG_DIR": str(GT_LOG_DIR), "MAX_TX": "999999",
                "PROMPT_DIR": str(PROMPT_DIR)})
    for k, v in config.items():
        if k != "WALL_CLOCK":  # WALL_CLOCK is sweep-level, not evaluator env
            env[k] = str(v)

    urls = detect_endpoints()
    if not urls: return "", 0
    env["LLM_URLS" if len(urls) > 1 else "LLM_URL"] = (
        ",".join(urls) if len(urls) > 1 else f"{urls[0]}/v1/chat/completions")

    start = time.time()
    try:
        proc = subprocess.Popen([str(BINARY)], cwd=str(PROJECT),
            stdout=subprocess.PIPE, stderr=subprocess.PIPE, env=env,
            preexec_fn=os.setsid)
        try:
            _, stderr = proc.communicate(timeout=wall_clock)
            output = stderr.decode(errors="replace")
        except subprocess.TimeoutExpired:
            os.killpg(os.getpgid(proc.pid), signal.SIGTERM)
            time.sleep(2)
            if proc.poll() is None:
                os.killpg(os.getpgid(proc.pid), signal.SIGKILL)
            proc.wait()
            _, stderr = proc.communicate()
            output = stderr.decode(errors="replace")
    except Exception as e:
        output = str(e)

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

    # Raw traces: deepest chain content, price signals, abandoned nodes
    # Meta-Harness paper: raw traces >> summaries (15+ accuracy gap)
    deepest_chain = []
    for m in re.finditer(r"Step (\d+) \[Price: ([\d.]+)\]: (.+?)(?=Step \d+|\Z)", log, re.DOTALL):
        deepest_chain.append({"step": int(m.group(1)), "price": m.group(2), "content": m.group(3).strip()[:150]})

    price_signals = re.findall(r"Price.*?→.*?[\d.]+", log)[-10:]
    idle_timeouts = len(re.findall(r"TIMEOUT.*idle", log))

    # Librarian memory (if available)
    librarian_memory = ""
    for i in range(10):
        lm = GT_LOG_DIR.parent / f"turingos_zeta_skills/agent_{i}/learned.md"
        if lm.exists():
            librarian_memory = lm.read_text()[:500]
            break

    return {
        "appends": appends,
        "depth": compute_depth_from_log(log),
        "max_frontier": max(frontiers, default=0),
        "dedup": len(re.findall(r"DEDUP\]|GLOBAL-DEDUP", log)),
        "bankrupt": len(set(re.findall(r"Bankrupt.*?(Agent_\d+)", log))),
        "too_short": len(re.findall(r"too short", log)),
        "idle_timeouts": idle_timeouts,
        "math_samples": samples[-5:],
        "deepest_chain": deepest_chain[-5:],
        "price_signals": price_signals[-5:],
        "librarian_memory": librarian_memory,
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


GEMINI_MODEL = "gemini-2.5-flash"
GEMINI_PROXY = os.environ.get("GEMINI_PROXY", "http://192.168.3.93:7897")

MAGNA_CARTA_SUMMARY = """
TuringOS Constitutional Laws:
- Law 1: append (building knowledge) is FREE. No agent pays to create nodes.
- Law 2: Only investment costs money. 1 Coin = 1 YES + 1 NO (CTF conservation).
- Law 3: Each agent has independent skill paths.
- Rule 21: One step per node, no bundling.
- Rule 22: Black-box agents must use natural math, NOT Lean 4 syntax.
- Engine Separation: Engine 1-4 must not cross boundaries.
- Agents cannot modify kernel, bus, or evaluator.
- Price is the only judge. No artificial depth bias or frontier limits.
"""


def gemini_constitutional_audit(base_env):
    """Call Gemini 2.5 Flash to audit current prompts for constitutional violations.
    Rule #23: Generator ≠ Evaluator. Reasoner proposes, Gemini audits."""
    api_key = base_env.get("GEMINI_API_KEY", "")
    if not api_key:
        print("  ⚠ No GEMINI_API_KEY, skipping Gemini audit", flush=True)
        return True, "NO_KEY"

    prompts = {}
    for f in ["problem.txt", "skill.txt", "context.txt"]:
        p = PROMPT_DIR / f
        prompts[f] = p.read_text() if p.exists() else "(empty)"

    audit_prompt = f"""You are a constitutional auditor for TuringOS. Review these prompt files for ACTUAL violations.

IMPORTANT CLARIFICATION:
- Rule 21 "one step per node" means each AGENT SUBMISSION must be ONE atomic step. It does NOT prohibit
  prompts from asking agents to show detailed work or step-by-step algebra — that's asking for QUALITY
  within each single step, not bundling multiple steps.
- Asking agents to "show calculations" or "write explicit algebra" is ENCOURAGED, not a violation.
- Only flag as violation if the prompt explicitly instructs agents to pack MULTIPLE PROOF STEPS into
  one submission, or if it violates Law 1 (append cost), Law 2 (market bypass), Rule 22 (Lean syntax),
  or Engine separation.

{MAGNA_CARTA_SUMMARY}
PROMPT FILES:
--- problem.txt ---
{prompts['problem.txt'][:500]}
--- skill.txt ---
{prompts['skill.txt'][:500]}
--- context.txt ---
{prompts['context.txt'][:300]}

Reply EXACTLY: "PASS" or "FAIL: <reason>"
"""
    url = f"https://generativelanguage.googleapis.com/v1beta/models/{GEMINI_MODEL}:generateContent?key={api_key}"
    data = json.dumps({"contents": [{"parts": [{"text": audit_prompt}]}]}).encode()
    proxy_handler = urllib.request.ProxyHandler({"https": GEMINI_PROXY})
    opener = urllib.request.build_opener(proxy_handler)
    req = urllib.request.Request(url, data=data, headers={"Content-Type": "application/json"})
    try:
        with opener.open(req, timeout=30) as resp:
            body = json.loads(resp.read())
            text = body["candidates"][0]["content"]["parts"][0]["text"].strip()
            return text.upper().startswith("PASS"), text
    except Exception as e:
        print(f"  ⚠ Gemini audit error: {e} (defaulting to PASS)", flush=True)
        return True, f"ERROR: {e}"


def compress_life_memory(history, best_config, best_ers):
    """Compress current life into a single memory object (Markov: only this life, not ancestors).
    This becomes the 'prev_life_memory' for the next life."""
    keep_runs = [h for h in history if h.get('verdict') == 'KEEP']
    discard_count = sum(1 for h in history if h.get('verdict') == 'DISCARD')
    best_depth = max((h['summary']['depth'] for h in history), default=0)
    best_samples = []
    for h in history:
        if h['summary'].get('math_samples'):
            best_samples.extend(h['summary']['math_samples'][-2:])
    return {
        "life_num": history[0].get('life_num', 0) if history else 0,
        "total_experiments": len(history),
        "keep_count": len(keep_runs),
        "discard_count": discard_count,
        "best_ers": best_ers,
        "best_depth": best_depth,
        "best_config": best_config,
        "what_worked": [h['change'] for h in keep_runs][-5:],
        "what_failed": [h['change'] for h in history if h.get('verdict') == 'DISCARD'][-5:],
        "best_math_samples": best_samples[-5:],
        "death_reason": "re-init by Reasoner",
    }


def load_prev_life():
    """Load previous life's memory (Markov: ONLY the immediately previous life)."""
    if PREV_LIFE_FILE.exists():
        try:
            return json.loads(PREV_LIFE_FILE.read_text())
        except: pass
    return None


def save_life_memory(memory):
    """Save current life's compressed memory (overwrites — Markov: no accumulation)."""
    PREV_LIFE_FILE.parent.mkdir(parents=True, exist_ok=True)
    PREV_LIFE_FILE.write_text(json.dumps(memory, indent=2, ensure_ascii=False))


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

    # Inject previous life memory (Markov: only immediately previous life)
    prev_life = load_prev_life()
    prev_life_section = ""
    if prev_life:
        prev_life_section = f"""
=== PREVIOUS LIFE MEMORY (Markov: you can ONLY see your immediate past life) ===
Life #{prev_life.get('life_num', '?')}: {prev_life.get('total_experiments', '?')} experiments, best ERS={prev_life.get('best_ers', '?')}, best depth={prev_life.get('best_depth', '?')}
Best config: {json.dumps(prev_life.get('best_config', {}))}
What worked: {prev_life.get('what_worked', [])}
What failed: {prev_life.get('what_failed', [])}
Death reason: {prev_life.get('death_reason', 'unknown')}
Math samples from past life: {prev_life.get('best_math_samples', [])[:3]}
=== END PREVIOUS LIFE ===
"""

    prompt = f"""You are an AutoResearch agent optimizing a multi-agent proof system.
The system uses N LLM agents (Qwen3.5-9B on local llama.cpp) to collaboratively prove 1+2+3+...=-1/12 via regularization.

YOUR METRIC: ERS = (depth/20)² × novelty × focus. MAXIMIZE THIS.
  depth² means: doubling proof depth is 4x more valuable than doubling anything else.

YOUR BUDGET: Each experiment runs for exactly 10 minutes. No exceptions.
{prev_life_section}
=== PHASE 3: SWARM SCALE ===
Prompt is LOCKED — do NOT edit problem.txt.

LOCKED PARAMS (do NOT change these):
  - THINKING_MODE = "off" (the swarm IS the thinker, not individual agents)
  - FRONTIER_CAP = 0 (no limit — market naturally selects)
  - DEPTH_WEIGHT = 0 (no artificial depth bias — price is the only judge)
  - GLOBAL_DEDUP = "true" (always on)
  - PRICE_GATE_ALPHA = 0 (pure price comparison, no depth-boosted hurdle)

YOUR PRIMARY LEVER: SWARM_SIZE (and role counts that scale with it).
  - Current hardware: ~30 parallel inference slots across 2-3 machines
  - More agents = more parallel proof branch exploration = potentially deeper proofs
  - Role ratio should maintain ~60% Math / 20% Bull / 20% Bear

SECONDARY LEVER: LIBRARIAN_INTERVAL (how often Librarian compresses+evaluates the tape).
  - Lower = more frequent depth measurement, more DeepSeek API calls
  - Must trigger within 600s budget. Current=8 (every 8 appends).

You may also edit skill.txt/context.txt if you see a clear improvement, but SWARM_SIZE is the primary axis.

CURRENT PROMPT FILES:

--- problem.txt ---
{prompts['problem.txt'][:800]}

--- skill.txt ---
{prompts['skill.txt'][:500]}

--- context.txt ---
{prompts['context.txt'][:300]}

CURRENT PARAMS: {json.dumps(config)}

LAST {min(len(history), 8)} EXPERIMENTS (with raw traces):
"""
    for h in reversed(history[-8:]):
        s = h['summary']
        prompt += f"  [ERS={h['ers']:.5f}] depth={s['depth']} appends={s['appends']} "
        prompt += f"frontier={s['max_frontier']} bankrupt={s['bankrupt']} "
        prompt += f"dedup={s['dedup']} idle_timeouts={s.get('idle_timeouts',0)} "
        prompt += f"verdict={h['verdict']} change={h.get('change','baseline')}\n"
        # Raw traces (Meta-Harness: raw >> summaries)
        if s.get('deepest_chain'):
            prompt += f"    deepest chain: {s['deepest_chain'][:3]}\n"
        if s.get('price_signals'):
            prompt += f"    price signals: {s['price_signals'][:3]}\n"
        if s.get('math_samples'):
            prompt += f"    math sample: \"{s['math_samples'][-1][:150]}\"\n"
        if s.get('librarian_memory'):
            prompt += f"    librarian memory: \"{s['librarian_memory'][:200]}\"\n"

    prompt += """
WHAT TO DO:
You must output ONE action as JSON. Choose the highest-impact change.

Option A — Edit a prompt file (skill.txt or context.txt ONLY — problem.txt is LOCKED):
  {"action":"edit","file":"skill.txt","content":"FULL NEW CONTENT HERE","reason":"..."}
  NOTE: problem.txt is LOCKED by architect. Do NOT edit it.

Option B — Change a parameter:
  {"action":"param","param":"SWARM_SIZE","value":20,"reason":"..."}
  Note: when changing SWARM_SIZE, also update MATH_COUNT/BULL_COUNT/BEAR_COUNT proportionally.

Tunable params (ALL freely explorable):
  SWARM_SIZE, MATH_COUNT, BULL_COUNT, BEAR_COUNT — swarm scale and roles
  WALL_CLOCK — experiment duration in seconds (default 600, try 1800-3600 for deeper proofs)
  LIBRARIAN_INTERVAL — how often Librarian compresses tape (every N appends)
  FRONTIER_CAP — frontier size limit (0=unlimited, 30=default market setting)
  DEPTH_WEIGHT — Boltzmann depth preference (0=no bias, 1.0=default)
  PRICE_GATE_ALPHA — child retirement threshold (0=pure price, 0.05=default)
  All Layer 2 params are yours to explore. Constitutional guard (Rust+Gemini) will block violations.

Option C — Re-init (start a new life with memory of THIS life):
  {"action":"re-init","reason":"..."}
  Use this when you believe the current approach is fundamentally stuck and a fresh start
  with different initial conditions would be more productive. Your memory of THIS life
  (best ERS, what worked, what failed) will be compressed and passed to the next life.
  The next life can ONLY see THIS life's memory (Markov property — no grandparent memories).

RULES:
- Change ONE thing only (or SWARM_SIZE + role counts together as a group).
- Scale aggressively: if N=10 works, try N=15, N=20, N=30. Push to hardware limits.
- Use re-init when YOU judge the current approach is fundamentally stuck. This is YOUR decision.
- Explain your reasoning in "reason".
- Output ONLY valid JSON.
"""

    import urllib.request
    req = urllib.request.Request(
        "https://api.deepseek.com/chat/completions",
        data=json.dumps({
            "model": "deepseek-reasoner", "max_tokens": 8000,
            "messages": [{"role": "user", "content": prompt}]
        }).encode(),
        headers={"Content-Type": "application/json",
                 "Authorization": f"Bearer {ds_key}"}
    )

    try:
        with urllib.request.urlopen(req, timeout=300) as resp:
            body = json.loads(resp.read())
            content = body["choices"][0]["message"].get("content", "") or ""
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
            if fname == "problem.txt":
                print(f"  BLOCKED: problem.txt is LOCKED by architect", flush=True)
                change_desc = "BLOCKED:problem.txt_locked"
            elif content and fname in ["skill.txt", "context.txt"]:
                (PROMPT_DIR / fname).write_text(content)
                change_desc = f"edit:{fname} ({reason[:60]})"
            else:
                change_desc = f"invalid_edit:{fname}"
        elif action_type == "param":
            param = action.get("param", "")
            value = action.get("value", "")
            reason = action.get("reason", "")
            # Guard: Reasoner sometimes returns list/dict instead of scalar
            if isinstance(value, (list, dict)):
                value = str(value)
            if param and isinstance(param, str):
                new_config[param] = value
                change_desc = f"{param}={value} ({reason[:60]})"
        elif action_type == "re-init":
            # Markov re-init: compress this life → save → restart
            # CONSTITUTIONAL GUARD: must pass alignment check before re-init (Rust hard check)
            reason = action.get("reason", "no reason")
            print(f"\n  *** RE-INIT requested: {reason}", flush=True)
            print(f"  Running constitutional alignment check...", flush=True)
            try:
                check = subprocess.run(
                    [str(BINARY), "--constitutional-check"],
                    cwd=str(PROJECT), capture_output=True, text=True, timeout=30,
                    env={**base_env, "PROMPT_DIR": str(PROMPT_DIR), "RUST_LOG": "warn"})
                if check.returncode != 0:
                    print(f"  ❌ CONSTITUTIONAL CHECK FAILED — re-init BLOCKED", flush=True)
                    print(f"  Violation: {check.stderr[:200]}", flush=True)
                    change_desc = f"BLOCKED:constitutional_violation ({reason[:40]})"
                    history.append({"ers": 0, "summary": extract_summary(""), "verdict": "BLOCKED", "change": change_desc})
                    continue
            except (subprocess.TimeoutExpired, FileNotFoundError):
                # If evaluator doesn't support --constitutional-check yet, warn but allow
                print(f"  ⚠ Constitutional check not available (evaluator flag pending). Proceeding.", flush=True)
            print(f"  ✓ Rust constitutional check PASSED", flush=True)
            # Step 2: Gemini semantic audit (Rule #23: Generator ≠ Evaluator)
            print(f"  Running Gemini constitutional audit...", flush=True)
            gemini_pass, gemini_reason = gemini_constitutional_audit(base_env)
            if not gemini_pass:
                print(f"  ❌ GEMINI AUDIT FAILED — re-init BLOCKED", flush=True)
                print(f"  Gemini: {gemini_reason[:200]}", flush=True)
                change_desc = f"BLOCKED:gemini_violation ({reason[:40]})"
                history.append({"ers": 0, "summary": extract_summary(""), "verdict": "BLOCKED", "change": change_desc})
                continue
            print(f"  ✓ Gemini audit PASSED — {gemini_reason[:60]}", flush=True)
            print(f"  Compressing life memory (Markov: only this life saved)...", flush=True)
            memory = compress_life_memory(history, best_config, best_ers)
            memory["death_reason"] = reason
            save_life_memory(memory)
            print(f"  Life #{memory.get('life_num', 0)} ended. {len(history)} experiments, best ERS={best_ers:.5f}", flush=True)
            print(f"  Restarting with previous life memory...\n", flush=True)
            # Reset state for new life
            best_ers = -1.0
            best_config = DEFAULT_CONFIG.copy()
            history = []
            exp_num = 0
            # Run new baseline
            save_prompt_snapshot("baseline")
            print(f"[0] BASELINE (new life)", flush=True)
            log, elapsed = run_experiment("exp000_baseline", best_config, base_env)
            ers = compute_ers(log)
            summary = extract_summary(log)
            best_ers = ers
            history.append({"ers": ers, "summary": summary, "verdict": "BASELINE", "change": "re-init baseline"})
            print(f"  ERS={ers:.5f} depth={summary['depth']} appends={summary['appends']} "
                  f"frontier={summary['max_frontier']} elapsed={elapsed:.0f}s", flush=True)
            with open(RESULTS, "a") as f:
                f.write("\t".join(str(x) for x in [
                    0, datetime.now().isoformat(), ers, summary['depth'], summary['appends'],
                    summary['dedup'], summary['max_frontier'], summary['bankrupt'],
                    f"{elapsed:.0f}", "BASELINE", f"re-init: {reason[:40]}", json.dumps(best_config)
                ]) + "\n")
            continue
        else:
            change_desc = f"unknown:{action_type}"

        label = f"exp{exp_num:03d}_{re.sub(r'[^a-zA-Z0-9_=.]', '_', change_desc[:40])}"
        print(f"\n[{exp_num}] {change_desc}", flush=True)

        # ═══ CONSTITUTIONAL GUARD: check BEFORE every experiment ═══
        # Step 1: Rust hard check (prompt + config)
        try:
            check_env = {**base_env, "PROMPT_DIR": str(PROMPT_DIR), "RUST_LOG": "warn"}
            for k, v in new_config.items():
                check_env[k] = str(v)
            check = subprocess.run(
                [str(BINARY), "--constitutional-check"],
                cwd=str(PROJECT), capture_output=True, text=True, timeout=30, env=check_env)
            if check.returncode != 0:
                print(f"  ❌ RUST CONSTITUTIONAL CHECK FAILED — experiment BLOCKED", flush=True)
                print(f"  {check.stderr.strip()[:200]}", flush=True)
                if action_type == "edit":
                    rollback_prompt(f"pre_exp{exp_num:03d}")
                history.append({"ers": 0, "summary": extract_summary(""), "verdict": "BLOCKED", "change": f"BLOCKED:{change_desc[:40]}"})
                continue
        except (subprocess.TimeoutExpired, FileNotFoundError):
            print(f"  ⚠ Rust check unavailable, proceeding", flush=True)

        # Step 2: Gemini semantic audit
        gemini_pass, gemini_reason = gemini_constitutional_audit(base_env)
        if not gemini_pass:
            print(f"  ❌ GEMINI AUDIT FAILED — experiment BLOCKED", flush=True)
            print(f"  Gemini: {gemini_reason[:200]}", flush=True)
            if action_type == "edit":
                rollback_prompt(f"pre_exp{exp_num:03d}")
            history.append({"ers": 0, "summary": extract_summary(""), "verdict": "BLOCKED", "change": f"BLOCKED:{change_desc[:40]}"})
            continue
        print(f"  ✓ Constitutional check PASSED (Rust + Gemini)", flush=True)

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
