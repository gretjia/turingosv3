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

import subprocess, re, time, os, sys, json, shutil, signal, traceback, hashlib, urllib.request
from pathlib import Path
from datetime import datetime

PROJECT = Path("/home/zephryj/projects/turingosv3")
BINARY = PROJECT / "target/release/evaluator"
PROMPT_DIR = PROJECT / "experiments/zeta_sum_proof/prompt"
RESULTS = PROJECT / "experiments/zeta_sum_proof/audit/autoresearch_v4_phase2.tsv"
LOG_DIR = PROJECT / "experiments/zeta_sum_proof/logs/v4"
GT_LOG_DIR = Path("/tmp/turingos_zeta_logs")
PREV_LIFE_FILE = PROJECT / "experiments/zeta_sum_proof/audit/prev_life_memory.json"

# ═══ ROLE ASSIGNMENT (switch APIs here) ═══
# Role 1: 主研究员 — proposes changes, runs the search algorithm
RESEARCHER_API = "deepseek"                         # "gemini" or "deepseek"
RESEARCHER_MODEL = "deepseek-reasoner"              # DeepSeek R1
RESEARCHER_PROXY = ""                               # DeepSeek doesn't need proxy

# Role 2: 大宪章审核员 — reviews changes for constitutional violations (Rule #23: Generator ≠ Evaluator)
AUDITOR_API = "deepseek"                            # "gemini" or "deepseek"
AUDITOR_MODEL = "deepseek-chat"                     # DeepSeek V3

# ── BUDGET (tunable by Reasoner) ──
WALL_CLOCK_SECS = 300  # 5 min — fast iteration for diagnosing node production rate. Researcher can override via config["WALL_CLOCK"].

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


# Legacy constants — now derived from ROLE ASSIGNMENT above
GEMINI_PROXY = RESEARCHER_PROXY

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


def constitutional_audit_deepseek(base_env):
    """DeepSeek Reasoner audits prompts for constitutional violations.
    Rule #23: Generator (Gemini) ≠ Evaluator (DeepSeek). Roles swapped."""
    ds_key = base_env.get("DEEPSEEK_API_KEY", "")
    if not ds_key:
        print("  ⚠ No DEEPSEEK_API_KEY, skipping audit", flush=True)
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
    try:
        req = urllib.request.Request(
            "https://api.deepseek.com/chat/completions",
            data=json.dumps({
                "model": AUDITOR_MODEL, "max_tokens": 500,
                "messages": [{"role": "user", "content": audit_prompt}]
            }).encode(),
            headers={"Content-Type": "application/json", "Authorization": f"Bearer {ds_key}"})
        with urllib.request.urlopen(req, timeout=120) as resp:
            body = json.loads(resp.read())
            text = (body["choices"][0]["message"].get("content", "") or "").strip()
            return text.upper().startswith("PASS"), text
    except Exception as e:
        print(f"  ⚠ DeepSeek audit error: {e} (defaulting to PASS)", flush=True)
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

    Gemini 3.1 Pro Preview outputs:
      - EITHER a param change: {"action": "param", "param": "X", "value": Y, "reason": "..."}
      - OR a prompt edit: {"action": "edit", "file": "...", "content": "...", "reason": "..."}
    """

    # Read prompt files + researcher notebook (separate from agent-visible context.txt)
    prompts = {}
    for f in ["problem.txt", "skill.txt"]:
        p = PROMPT_DIR / f
        prompts[f] = p.read_text() if p.exists() else "(using default)"
    # research_notes.txt: researcher-only, NOT visible to agents (agents see context.txt via evaluator)
    notes_path = PROMPT_DIR / "research_notes.txt"
    prompts["research_notes.txt"] = notes_path.read_text() if notes_path.exists() else "(empty — write your hypotheses here)"

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

    prompt = f"""You are an autonomous research scientist running experiments on TuringOS — a multi-agent proof system where N LLM agents collaboratively prove mathematical theorems via a prediction market.

METRIC: ERS = (depth/20)² × novelty × focus. Depth is king — depth=10 is 4x better than depth=5.

=== 大宪章 (Magna Carta — align all decisions with these laws) ===

THREE LAWS:
- Law 1 (信息平权): Black-box agents use white-box tools for FREE. Append (building knowledge) costs ZERO. No monopoly of any kind.
- Law 2 (共识的代价): The ONLY action that costs money is investment. Capital = topological confidence. 1 Coin = 1 YES + 1 NO (CTF conservation). Agents profit ONLY by finding mispriced probabilities. System market maker provides initial liquidity for each node.
- Law 3 (数字产权): Each agent has its own independent skill path. Species evolution through experience.

FOUR ENGINES:
- Engine 1 (Epistemic): Free tools — append, search, view. Building knowledge has zero cost.
- Engine 2 (Capital): Prediction market — invest YES (endorse) or NO (challenge). Price = Bayesian probability. This is the ONLY mechanism for consensus. No artificial depth bias, no frontier limits, no price gate hurdles.
- Engine 3 (Oracle): DeepSeek Reasoner verifies completed proofs. Triggers at P>=90%. Settles all markets on the golden path.
- Engine 4 (Evolution): Each agent has independent skill memory. Librarian (map-reduce clock from topology.md) periodically compresses the tape into agent memory. Bankruptcy triggers autopsy mutation, victory triggers reinforcement. Librarian interval is system infrastructure, not tunable.

LOCKED ECONOMIC PARAMETERS (constitutional defaults):
- FRONTIER_CAP = 0 (unlimited — Law 1, information equality)
- DEPTH_WEIGHT = 0 (no artificial bias — Law 2, price is the only judge)
- PRICE_GATE_ALPHA = 0 (pure price comparison — Law 2, no hurdle)

Your research must ALIGN with these laws. The constitutional guard (Rust + DeepSeek) enforces automatically.

{prev_life_section}
=== YOUR LAB ===

PROMPT FILES:

--- problem.txt (the math problem — architect-set, code-enforced) ---
{prompts['problem.txt']}

--- skill.txt (agent tool instructions — architect-set, code-enforced) ---
{prompts['skill.txt']}

--- research_notes.txt (YOUR private notebook — NOT visible to agents) ---
{prompts['research_notes.txt']}

Use research_notes.txt as your persistent thinking space. Write hypotheses, failed approach notes, plans.
The human operator also writes here. Your notes survive across rounds — this is your memory.
IMPORTANT: This file is PRIVATE to you. Agents cannot see it. Do NOT leak answer paths here — but you CAN reason freely about the problem.

CURRENT CONFIG: {json.dumps(config)}

=== RAW EXPERIMENT DATA ===

LAST {min(len(history), 8)} experiments:
"""
    for h in reversed(history[-8:]):
        s = h['summary']
        prompt += f"  [ERS={h['ers']:.5f}] depth={s['depth']} appends={s['appends']} "
        prompt += f"frontier={s['max_frontier']} bankrupt={s['bankrupt']} "
        prompt += f"dedup={s['dedup']} idle_timeouts={s.get('idle_timeouts',0)} "
        prompt += f"verdict={h['verdict']} change={h.get('change','baseline')}\n"
        if s.get('deepest_chain'):
            prompt += f"    deepest chain: {s['deepest_chain']}\n"
        if s.get('price_signals'):
            prompt += f"    price signals: {s['price_signals']}\n"
        if s.get('math_samples'):
            prompt += f"    math sample: \"{s['math_samples'][-1][:200]}\"\n"
        if s.get('librarian_memory'):
            prompt += f"    librarian says: \"{s['librarian_memory'][:300]}\"\n"

    prompt += """
=== WHAT YOU CAN DO ===

Option A — Edit research_notes.txt (your private notebook — NOT visible to agents):
  {"action":"edit","file":"research_notes.txt","content":"FULL NEW CONTENT","reason":"..."}

Option B — Change any config parameter:
  {"action":"param","param":"PARAM_NAME","value":"VALUE","reason":"..."}
  Available: SWARM_SIZE, MATH_COUNT, BULL_COUNT, BEAR_COUNT, WALL_CLOCK
  All freely explorable. Constitutional guard checks automatically.

Option C — Re-init (fresh start with memory of this life):
  {"action":"re-init","reason":"..."}

=== RESEARCH PRINCIPLES ===

You are a scientist, not a script. Think deeply before each experiment:
- DIAGNOSE before acting: why did the last experiment fail? What is the root cause?
- Form a HYPOTHESIS: "I believe X will improve depth because Y"
- Test ONE variable at a time so you can attribute causation
- Read the raw data carefully — the numbers tell a story
- If stuck after many failures, consider a fundamentally different approach (re-init)
- problem.txt is set by the architect (edits are code-blocked). Everything else is your lab
- You have full freedom. The only law is the 大宪章 (constitutional guard enforces it automatically)

Output ONLY valid JSON. Explain your hypothesis in "reason".
"""

    # Main researcher: uses RESEARCHER_API / RESEARCHER_MODEL
    ds_key = base_env.get("DEEPSEEK_API_KEY", "")
    gemini_key = base_env.get("GEMINI_API_KEY", "")

    content = ""
    try:
        if RESEARCHER_API == "deepseek":
            # DeepSeek streaming — keeps connection alive during long thinking
            import http.client, select
            conn = http.client.HTTPSConnection("api.deepseek.com", timeout=30)
            conn.request("POST", "/chat/completions", body=json.dumps({
                "model": RESEARCHER_MODEL, "max_tokens": 8000, "stream": True,
                "messages": [{"role": "user", "content": prompt}]
            }), headers={"Content-Type": "application/json", "Authorization": f"Bearer {ds_key}"})
            resp = conn.getresponse()
            if resp.status != 200:
                print(f"  LLM error: HTTP {resp.status} {resp.read().decode()[:200]}", flush=True)
                return "error", None
            buf = b""
            while True:
                ready, _, _ = select.select([resp], [], [], 120)
                if not ready:
                    print(f"  LLM heartbeat timeout", flush=True); break
                chunk_bytes = resp.read(4096)
                if not chunk_bytes: break
                buf += chunk_bytes
                while b"\n" in buf:
                    line_bytes, buf = buf.split(b"\n", 1)
                    line = line_bytes.decode("utf-8").strip()
                    if not line.startswith("data: "): continue
                    if line[6:] == "[DONE]": break
                    try:
                        delta = json.loads(line[6:]).get("choices", [{}])[0].get("delta", {})
                        content += delta.get("content", "") or ""
                    except json.JSONDecodeError: continue
                else: continue
                break
            conn.close()
        else:
            # Gemini via proxy
            gemini_url = f"https://generativelanguage.googleapis.com/v1beta/models/{RESEARCHER_MODEL}:generateContent?key={gemini_key}"
            data = json.dumps({"contents": [{"parts": [{"text": prompt}]}]}).encode()
            proxy_handler = urllib.request.ProxyHandler({"https": RESEARCHER_PROXY})
            opener = urllib.request.build_opener(proxy_handler)
            req = urllib.request.Request(gemini_url, data=data, headers={"Content-Type": "application/json"})
            with opener.open(req, timeout=600) as resp:
                body = json.loads(resp.read())
                content = body["candidates"][0]["content"]["parts"][0]["text"].strip()

        if content:
            content = re.sub(r'```json\s*', '', content)
            content = re.sub(r'```\s*', '', content)
            match = re.search(r'\{.*\}', content, re.DOTALL)
            if match:
                action = json.loads(match.group())
                return action.get("action", "param"), action
    except Exception as e:
        print(f"  LLM error: {e}", flush=True)

    return "error", None


# ═══ HELPERS (single source of truth) ═══

EMPTY_SUMMARY = {"depth": 0, "appends": 0, "max_frontier": 0, "dedup": 0,
    "bankrupt": 0, "too_short": 0, "idle_timeouts": 0,
    "math_samples": [], "deepest_chain": [], "price_signals": [], "librarian_memory": ""}


def append_tsv(num, ers, summary, elapsed, verdict, change, config):
    """Single TSV writer — no more 4 duplicate blocks."""
    with open(RESULTS, "a") as f:
        f.write("\t".join(str(x) for x in [
            num, datetime.now().isoformat(), ers, summary.get('depth', 0),
            summary.get('appends', 0), summary.get('dedup', 0), summary.get('max_frontier', 0),
            summary.get('bankrupt', 0), f"{elapsed:.0f}" if isinstance(elapsed, float) else elapsed,
            verdict, change, json.dumps(config)
        ]) + "\n")


def constitutional_guard(new_config, base_env, prompts_changed):
    """Rust + Gemini check. Skip Gemini for param-only changes (prompts unchanged)."""
    # Rust check (always — fast subprocess)
    try:
        check_env = {**base_env, "PROMPT_DIR": str(PROMPT_DIR), "RUST_LOG": "warn"}
        for k, v in new_config.items():
            check_env[k] = str(v)
        check = subprocess.run(
            [str(BINARY), "--constitutional-check"],
            cwd=str(PROJECT), capture_output=True, text=True, timeout=30, env=check_env)
        if check.returncode != 0:
            return False, f"RUST: {check.stderr.strip()[:100]}"
    except (subprocess.TimeoutExpired, FileNotFoundError):
        pass  # evaluator unavailable, proceed

    # Gemini check (only when prompts changed — skip for param-only to save 30s + API cost)
    if prompts_changed:
        gemini_pass, reason = constitutional_audit_deepseek(base_env)
        if not gemini_pass:
            return False, f"GEMINI: {reason[:100]}"

    return True, "PASS"


def run_baseline(label, config, base_env, change_desc="baseline"):
    """Run baseline and record. Used by initial start and re-init."""
    save_prompt_snapshot(label.replace("exp000_", ""))
    log, elapsed = run_experiment(label, config, base_env)
    ers = compute_ers(log)
    summary = extract_summary(log)
    print(f"  ERS={ers:.5f} depth={summary['depth']} appends={summary['appends']} "
          f"frontier={summary['max_frontier']} elapsed={elapsed:.0f}s", flush=True)
    if summary.get('math_samples'):
        print(f"  sample: {summary['math_samples'][-1][:120]}", flush=True)
    append_tsv(0, ers, summary, elapsed, "BASELINE", change_desc, config)
    return ers, summary


def resume_from_tsv():
    """Resume state from TSV on restart. No amnesia — read what we already know."""
    history = []
    best_ers = -1.0
    best_config = DEFAULT_CONFIG.copy()
    exp_num = 0
    if RESULTS.exists():
        for line in RESULTS.read_text().splitlines()[1:]:  # skip header
            parts = line.split('\t')
            if len(parts) >= 12:
                try:
                    num = int(parts[0])
                    ers = float(parts[2])
                    depth = int(parts[3])
                    appends = int(parts[4])
                    verdict = parts[9]
                    change = parts[10]
                    config = json.loads(parts[11])
                    history.append({
                        "ers": ers,
                        "summary": {"depth": depth, "appends": appends, "max_frontier": int(parts[6]),
                                    "dedup": int(parts[5]), "bankrupt": int(parts[7]),
                                    "too_short": 0, "idle_timeouts": 0, "math_samples": [],
                                    "deepest_chain": [], "price_signals": [], "librarian_memory": ""},
                        "verdict": verdict, "change": change,
                    })
                    if ers > best_ers:
                        best_ers = ers
                        best_config = config
                    exp_num = max(exp_num, num)
                except (ValueError, json.JSONDecodeError):
                    continue
    return history, best_ers, best_config, exp_num


def main():
    LOG_DIR.mkdir(parents=True, exist_ok=True)
    GT_LOG_DIR.mkdir(parents=True, exist_ok=True)
    PROMPT_DIR.mkdir(parents=True, exist_ok=True)
    base_env = load_env()

    if not BINARY.exists():
        print(f"ERROR: {BINARY} not found", file=sys.stderr); sys.exit(1)

    if not RESULTS.exists():
        RESULTS.parent.mkdir(parents=True, exist_ok=True)
        with open(RESULTS, "w") as f:
            f.write("\t".join(["num", "time", "ers", "depth", "appends", "dedup",
                "frontier", "bankrupt", "elapsed", "verdict", "change", "config"]) + "\n")

    history, best_ers, best_config, exp_num = resume_from_tsv()

    print("=" * 60)
    print("TuringOS AutoResearch v5 — NEVER STOP")
    print(f"  Endpoints: {detect_endpoints()}")
    if history:
        print(f"  RESUMED: {len(history)} experiments, best ERS={best_ers:.5f}, exp#{exp_num}")
    print("=" * 60, flush=True)

    if not history:
        print(f"\n[0] BASELINE", flush=True)
        ers, summary = run_baseline("exp000_baseline", best_config, base_env)
        best_ers = ers
        history.append({"ers": ers, "summary": summary, "verdict": "BASELINE", "change": "baseline"})

    last_error = ""
    while True:
        try:
            exp_num += 1
            action_type, action = llm_research_step(history, best_config, base_env)

            if action is None:
                print(f"\n[{exp_num}] LLM no action, retrying...", flush=True)
                time.sleep(10); exp_num -= 1; continue

            save_prompt_snapshot(f"pre_exp{exp_num:03d}")
            new_config = best_config.copy()
            change_desc = ""
            prompts_changed = False

            # ── Parse action ──
            if action_type == "edit":
                fname = action.get("file", "problem.txt")
                content = action.get("content", "")
                reason = action.get("reason", "")
                if fname in ("problem.txt", "skill.txt", "context.txt"):
                    change_desc = f"BLOCKED:{fname}_locked"
                elif content and fname == "research_notes.txt":
                    (PROMPT_DIR / fname).write_text(content)
                    change_desc = f"edit:notes ({reason[:60]})"
                else:
                    change_desc = f"invalid_edit:{fname}"

            elif action_type == "param":
                param = action.get("param", "")
                value = action.get("value", "")
                reason = action.get("reason", "")
                if isinstance(value, (list, dict)):
                    value = str(value)
                if param in ("FRONTIER_CAP", "DEPTH_WEIGHT", "PRICE_GATE_ALPHA", "LIBRARIAN_INTERVAL"):
                    change_desc = f"BLOCKED:{param}_locked"
                elif param and isinstance(param, str):
                    new_config[param] = value
                    change_desc = f"{param}={value} ({reason[:60]})"

            elif action_type == "re-init":
                reason = action.get("reason", "no reason")
                print(f"\n  *** RE-INIT: {reason}", flush=True)
                ok, msg = constitutional_guard(best_config, base_env, prompts_changed=True)
                if not ok:
                    print(f"  ❌ BLOCKED: {msg}", flush=True)
                    history.append({"ers": 0, "summary": EMPTY_SUMMARY, "verdict": "BLOCKED", "change": f"BLOCKED:{reason[:40]}"})
                    continue
                memory = compress_life_memory(history, best_config, best_ers)
                memory["death_reason"] = reason
                save_life_memory(memory)
                print(f"  Life ended ({len(history)} exp, best={best_ers:.5f}). New life...", flush=True)
                best_ers, best_config, history, exp_num = -1.0, DEFAULT_CONFIG.copy(), [], 0
                print(f"\n[0] BASELINE (new life)", flush=True)
                ers, summary = run_baseline("exp000_baseline", best_config, base_env, f"re-init: {reason[:40]}")
                best_ers = ers
                history.append({"ers": ers, "summary": summary, "verdict": "BASELINE", "change": "re-init baseline"})
                continue
            else:
                change_desc = f"unknown:{action_type}"

            label = f"exp{exp_num:03d}_{re.sub(r'[^a-zA-Z0-9_=.]', '_', change_desc[:40])}"
            print(f"\n[{exp_num}] {change_desc}", flush=True)

            # ── Constitutional guard (skip Gemini for param-only) ──
            ok, msg = constitutional_guard(new_config, base_env, prompts_changed)
            if not ok:
                print(f"  ❌ BLOCKED: {msg}", flush=True)
                if prompts_changed: rollback_prompt(f"pre_exp{exp_num:03d}")
                history.append({"ers": 0, "summary": EMPTY_SUMMARY, "verdict": "BLOCKED", "change": f"BLOCKED:{change_desc[:40]}"})
                continue

            # ── Run experiment ──
            log, elapsed = run_experiment(label, new_config, base_env)
            ers = compute_ers(log)
            summary = extract_summary(log)

            if ers > best_ers:
                verdict = "KEEP"
                print(f"  ▲ ERS={ers:.5f} (was {best_ers:.5f}) KEEP", flush=True)
                best_ers, best_config = ers, new_config
                save_prompt_snapshot(f"exp{exp_num:03d}")
            else:
                verdict = "DISCARD"
                print(f"  ▼ ERS={ers:.5f} (best={best_ers:.5f}) DISCARD", flush=True)
                if prompts_changed: rollback_prompt(f"pre_exp{exp_num:03d}")

            history.append({"ers": ers, "summary": summary, "verdict": verdict, "change": change_desc})
            print(f"  depth={summary['depth']} appends={summary['appends']} frontier={summary['max_frontier']} elapsed={elapsed:.0f}s", flush=True)
            append_tsv(exp_num, ers, summary, elapsed, verdict, change_desc, new_config)
            last_error = ""

        except Exception as e:
            last_error = f"{type(e).__name__}: {e}"
            print(f"\n  ⚠ CAUGHT: {last_error}\n  {traceback.format_exc()[-200:]}", flush=True)
            append_tsv(exp_num, 0, EMPTY_SUMMARY, 0, "ERROR", f"EXCEPTION:{last_error[:60]}", best_config)
            err_summary = {**EMPTY_SUMMARY, "librarian_memory": f"ERROR: {last_error}"}
            history.append({"ers": 0, "summary": err_summary, "verdict": "ERROR", "change": f"EXCEPTION:{last_error[:60]}"})
            try: rollback_prompt(f"pre_exp{exp_num:03d}")
            except: pass
            time.sleep(5)


if __name__ == "__main__":
    main()
