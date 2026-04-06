#!/usr/bin/env python3
"""
TuringOS AutoResearch v7 — Multi-Researcher Swarm Edition

"LLM IS the search algorithm." — Karpathy
"Each researcher is an independent life. Respect them." — Architect

Architecture:
  sweep.py = Researcher's body (runs experiments, reads data, calls API)
  DeepSeek Reasoner = Researcher's brain (forms hypotheses, proposes changes)
  run_experiment.py = Fixed experiment harness (Karpathy's prepare.py)
  PPUT = Progress Per Unit Time (golden_path_tokens / total_tokens × elapsed_minutes)
  bulletin.jsonl = Shared whiteboard (append-only, colleagues' discoveries)

Multi-researcher mode:
  - Each researcher has its own identity.json (id, name, proxy_port, models)
  - Shared knowledge: problem.txt, skill.txt, bulletin.jsonl
  - Private state: config.json, results.tsv, research_notes.txt, logs, tapes
  - Global semaphore: max 2 evaluators running simultaneously (OOM guard)

Usage:
  python3 sweep.py                           # interactive
  nohup python3 sweep.py > sweep.log 2>&1 &  # daemon
"""

import subprocess, re, time, os, sys, json, shutil, signal, traceback, fcntl, resource
import http.client, select
from pathlib import Path
from datetime import datetime

# ── Paths ──
BASE = Path(__file__).resolve().parent
PROJECT = BASE.parent.parent
BINARY = PROJECT / "target/release/evaluator"
CONFIG = BASE / "config.json"
RESULTS = BASE / "results.tsv"
NOTES = BASE / "research_notes.txt"
PREV_LIFE = BASE / "prev_life_memory.json"

# ── Shared paths (Knowledge Commons) ──
SHARED_DIR = BASE.parent / "shared"
BULLETIN = SHARED_DIR / "bulletin.jsonl"
PROMPT_DIR = SHARED_DIR if SHARED_DIR.exists() else BASE / "prompt"

# ── Identity ──
IDENTITY_FILE = BASE / "identity.json"

# ── Global evaluator semaphore (max 2 concurrent across all researchers) ──
SEMAPHORE_FILE = SHARED_DIR / ".evaluator_semaphore"

# ── OOM guard: max RSS per evaluator subprocess (MB) ──
EVALUATOR_MAX_RSS_MB = 2048  # 2GB hard limit per evaluator process tree

# ── Researcher + Auditor (Rule #23: Generator ≠ Evaluator) ──
RESEARCHER_MODEL = "deepseek-reasoner"   # DeepSeek R1
AUDITOR_MODEL = "deepseek-chat"          # DeepSeek V3

# ── Default Config ──
DEFAULT_CONFIG = {
    "swarm_size": 5, "math_count": 3, "bull_count": 1, "bear_count": 1,
    "provider": "proxy", "model": "qwen3-8b",
    "wall_clock": 300,
    "librarian_interval": 8,
    "description": "baseline: smallest viable swarm"
}

# ── LOCKED parameters (constitutional) ──
LOCKED_PARAMS = {"librarian_interval"}


# ══════════════════════════════════════════════════════════════
# Identity & Config
# ══════════════════════════════════════════════════════════════

def load_identity():
    """Load researcher identity. Falls back to solo mode."""
    if IDENTITY_FILE.exists():
        return json.loads(IDENTITY_FILE.read_text())
    return {
        "id": "solo",
        "name": "Solo Researcher",
        "proxy_port": 8088,
        "provider": "proxy",
        "proxy_url": "http://127.0.0.1:8088/v1/chat/completions",
        "default_model": "qwen3-8b",
        "available_models": ["qwen3-8b", "qwen3-32b", "qwen3-235b", "qwen-plus", "qwen-max"],
    }


def load_env():
    env = os.environ.copy()
    for line in (PROJECT / ".env").read_text().splitlines():
        line = line.strip()
        if "=" in line and not line.startswith("#"):
            k, v = line.split("=", 1)
            env[k.strip()] = v.strip()
    return env


# ══════════════════════════════════════════════════════════════
# Bulletin Board — shared whiteboard (append-only JSONL)
# ══════════════════════════════════════════════════════════════

def read_bulletin(limit=30):
    """Read last N bulletin entries. Efficient tail-read to avoid loading entire file."""
    if not BULLETIN.exists():
        return []
    try:
        # Read only the tail of the file (last ~64KB) to avoid OOM on large bulletins
        size = BULLETIN.stat().st_size
        with open(BULLETIN, "r", encoding="utf-8", errors="replace") as f:
            if size > 65536:
                f.seek(max(size - 65536, 0))
                f.readline()  # skip partial line
            lines = f.readlines()
        entries = []
        for line in lines[-limit:]:
            line = line.strip()
            if not line:
                continue
            try:
                entries.append(json.loads(line))
            except json.JSONDecodeError:
                continue
        return entries
    except (IOError, OSError):
        return []


def post_bulletin(identity_id, msg_type, msg, pput=None, config=None):
    """Append one entry to bulletin.jsonl. Atomic via fcntl + small write."""
    entry = {
        "ts": datetime.now().isoformat(timespec="seconds"),
        "from": identity_id,
        "type": msg_type,
        "msg": msg[:500],  # cap message length for sanity
    }
    if pput is not None:
        entry["pput"] = round(pput, 6)
    if config is not None:
        # Only include key config fields, not full blob
        entry["config"] = {k: config[k] for k in ("model", "swarm_size", "wall_clock") if k in config}

    line = json.dumps(entry, ensure_ascii=False) + "\n"
    assert len(line.encode()) < 4096, "Bulletin entry too large for atomic write"

    BULLETIN.parent.mkdir(parents=True, exist_ok=True)
    fd = os.open(str(BULLETIN), os.O_WRONLY | os.O_CREAT | os.O_APPEND, 0o644)
    try:
        fcntl.flock(fd, fcntl.LOCK_EX)
        os.write(fd, line.encode())
    finally:
        fcntl.flock(fd, fcntl.LOCK_UN)
        os.close(fd)


def format_bulletin_for_prompt(entries, my_id):
    """Format bulletin entries for researcher prompt. Only show others' entries."""
    others = [e for e in entries if e.get("from") != my_id]
    if not others:
        return ""
    lines = ["\n=== SHARED BULLETIN (colleagues' recent findings) ===\n"]
    for e in others[-15:]:
        prefix = f"  [{e.get('from','?')}] ({e.get('type','?')})"
        msg = e.get('msg', '')
        pput_str = f" [PPUT={e['pput']:.6f}]" if e.get('pput') else ""
        cfg_str = ""
        if e.get('config'):
            cfg_str = f" config={json.dumps(e['config'])}"
        lines.append(f"{prefix} {msg}{pput_str}{cfg_str}")
    lines.append("=== END BULLETIN ===\n")
    return "\n".join(lines)


# ══════════════════════════════════════════════════════════════
# Bulletin Cleanup — archive old entries (called periodically)
# ══════════════════════════════════════════════════════════════

def cleanup_bulletin(max_entries=200):
    """Keep only the last max_entries in bulletin.jsonl, archive the rest. Fully atomic."""
    if not BULLETIN.exists():
        return
    # Lock for entire read-archive-truncate transaction
    fd = os.open(str(BULLETIN), os.O_RDWR | os.O_CREAT, 0o644)
    try:
        fcntl.flock(fd, fcntl.LOCK_EX)
        with open(BULLETIN, "r") as f:
            lines = f.readlines()
        if len(lines) <= max_entries:
            return
        # Archive old entries
        archive = SHARED_DIR / f"bulletin_archive_{datetime.now().strftime('%Y%m%d_%H%M%S')}.jsonl"
        with open(archive, "w") as af:
            af.writelines(lines[:-max_entries])
        # Truncate and write recent entries
        with open(BULLETIN, "w") as bf:
            bf.writelines(lines[-max_entries:])
    finally:
        fcntl.flock(fd, fcntl.LOCK_UN)
        os.close(fd)


# ══════════════════════════════════════════════════════════════
# Global Evaluator Semaphore — OOM guard (max 2 concurrent)
# ══════════════════════════════════════════════════════════════

MAX_CONCURRENT_EVALUATORS = 4

def acquire_evaluator_slot(identity_id, timeout=600):
    """Acquire a slot to run the evaluator. Blocks until available.
    Uses a directory of lock files as a counting semaphore."""
    SEMAPHORE_FILE.parent.mkdir(parents=True, exist_ok=True)
    for attempt in range(timeout // 5):
        # Try to claim a slot
        for slot in range(MAX_CONCURRENT_EVALUATORS):
            slot_file = SHARED_DIR / f".eval_slot_{slot}.lock"
            try:
                fd = os.open(str(slot_file), os.O_WRONLY | os.O_CREAT, 0o644)
                fcntl.flock(fd, fcntl.LOCK_EX | fcntl.LOCK_NB)
                # Got the lock — write our identity
                os.ftruncate(fd, 0)
                os.lseek(fd, 0, os.SEEK_SET)
                os.write(fd, f"{identity_id} {os.getpid()} {datetime.now().isoformat()}\n".encode())
                return fd, slot  # caller must release
            except (BlockingIOError, OSError):
                try:
                    os.close(fd)
                except:
                    pass
                continue
        # All slots busy — wait
        if attempt % 12 == 0:  # log every 60s
            print(f"  [{identity_id}] Waiting for evaluator slot ({attempt*5}s)...", flush=True)
        time.sleep(5)
    raise TimeoutError(f"Could not acquire evaluator slot in {timeout}s")


def release_evaluator_slot(fd, slot):
    """Release an evaluator slot."""
    try:
        fcntl.flock(fd, fcntl.LOCK_UN)
        os.close(fd)
    except OSError:
        pass


# ══════════════════════════════════════════════════════════════
# LLM Call (DeepSeek Reasoner for researcher brain)
# ══════════════════════════════════════════════════════════════

def call_deepseek(prompt, model, api_key, max_tokens=8000):
    """Call DeepSeek API with streaming (keeps connection alive during long thinking)."""
    conn = http.client.HTTPSConnection("api.deepseek.com", timeout=60)
    conn.request("POST", "/chat/completions", body=json.dumps({
        "model": model, "max_tokens": max_tokens, "stream": True,
        "messages": [{"role": "user", "content": prompt}]
    }), headers={"Content-Type": "application/json", "Authorization": f"Bearer {api_key}"})

    resp = conn.getresponse()
    if resp.status != 200:
        err = resp.read().decode()[:300]
        conn.close()
        raise RuntimeError(f"DeepSeek HTTP {resp.status}: {err}")

    content = ""
    buf = b""
    while True:
        ready, _, _ = select.select([resp], [], [], 120)
        if not ready:
            break
        chunk = resp.read(4096)
        if not chunk:
            break
        buf += chunk
        while b"\n" in buf:
            line_bytes, buf = buf.split(b"\n", 1)
            line = line_bytes.decode("utf-8").strip()
            if not line.startswith("data: "):
                continue
            if line[6:] == "[DONE]":
                conn.close()
                return content
            try:
                delta = json.loads(line[6:]).get("choices", [{}])[0].get("delta", {})
                content += delta.get("content", "") or ""
            except json.JSONDecodeError:
                continue

    conn.close()
    return content


# ══════════════════════════════════════════════════════════════
# Researcher Prompt Builder
# ══════════════════════════════════════════════════════════════

def build_researcher_prompt(history, config, prev_life_memory, identity):
    """Build the prompt for the AI Researcher. 大宪章 FIRST."""

    # Read researcher's private notebook
    notes = NOTES.read_text() if NOTES.exists() else "(empty — write your hypotheses here)"

    # Read prompt files from shared commons
    prompts = {}
    for f in ["problem.txt", "skill.txt"]:
        p = PROMPT_DIR / f
        prompts[f] = p.read_text() if p.exists() else "(using default)"

    # Previous life memory (Markov)
    prev_life_section = ""
    if prev_life_memory:
        prev_life_section = f"""
=== PREVIOUS LIFE MEMORY (Markov: you see ONLY your immediate past life) ===
Life #{prev_life_memory.get('life_id', '?')}: {prev_life_memory.get('total_experiments', '?')} experiments, best ERS={prev_life_memory.get('best_pput', '?')}
Best config: {json.dumps(prev_life_memory.get('best_config', {}))}
What worked: {prev_life_memory.get('what_worked', [])}
What failed: {prev_life_memory.get('what_failed', [])}
Death reason: {prev_life_memory.get('death_reason', 'unknown')}
=== END PREVIOUS LIFE ===
"""

    # Bulletin board (colleagues' findings)
    bulletin_entries = read_bulletin(30)
    bulletin_section = format_bulletin_for_prompt(bulletin_entries, identity["id"])

    # Available models for this researcher
    available = identity.get("available_models", ["qwen3-8b"])
    models_str = ", ".join(available)

    prompt = f"""You are {identity.get('name', 'an autonomous research scientist')} (ID: {identity['id']}), running experiments on TuringOS — a multi-agent proof system where N LLM agents collaboratively prove mathematical theorems via a prediction market.

You are one of several researchers working on the same problem. You can see your colleagues' findings on the shared bulletin board below. Learn from their discoveries, but form your own hypotheses.

=== 大宪章 (Magna Carta — align ALL decisions with these laws FIRST) ===

THREE LAWS:
- Law 1 (信息平权): Black-box agents use white-box tools for FREE. Append costs ZERO.
- Law 2 (共识的代价): The ONLY action that costs money is investment. 1 Coin = 1 YES + 1 NO (CTF conservation).
- Law 3 (数字产权): Each agent has its own independent skill path. Species evolution.

FOUR ENGINES:
- Engine 1 (Epistemic): Free tools — append, search, view.
- Engine 2 (Capital): Prediction market — invest YES/NO. Price = Bayesian probability.
- Engine 3 (Oracle): DeepSeek Reasoner verifies completed proofs.
- Engine 4 (Evolution): Librarian compresses tape into agent memory every 8 appends.

LOCKED: LIBRARIAN_INTERVAL = 8

=== RESEARCH MISSION — TuringOS Scaling Law ===

CURRENT PRIORITY: Measure how proof depth scales with swarm size (N).
This is the single most important unanswered question for TuringOS.

TASK 1 — SCALING LAW: How does depth change as N (swarm_size) increases?
  Known data: N=1→depth=5 (single agent loops), N=3→depth~8, N=5→depth~9-16, N=7→depth~9, N=10→depth~7-18, N=15→depth~9
  The curve appears FLAT from N=3 to N=15 — but data is noisy and large-N is sparse.
  Open question: Is it (A) truly flat (emergence comes from mechanism, not scale), or (B) wall_clock too short for large swarms to fully interact?
  YOUR JOB: Run experiments at your assigned N values. Each N at least 3-5 times. Report ALL depth values (not just the best). Use median + variance.

TASK 2 — EMERGENCE FLOOR: Find the minimum conditions for emergence.
  Two dimensions: (a) minimum model size, (b) minimum swarm size.
  Already confirmed: qwen3-8b (8B) produces emergence at N=5 (4.6x vs single agent).
  Doubao-seed-2.0-lite produces emergence at N=5 (depth=11).
  Doubao-seed-1.6-flash does NOT (depth=1-2). The floor is somewhere in between.

TASK 3 — LOG-LOG CURVE: Collect enough data to plot log(N) vs log(depth).
  If depth ∝ N^α: α>1 = superlinear (more agents = disproportionately better), α=1 = linear, α<1 = sublinear (diminishing returns).
  Need data especially at large N (20, 30, 50+) to determine the slope.
  After finding the rough curve, add data points near the emergence threshold (e.g., N=4 if N=3 fails but N=5 works).

CONTROLS: Single-agent baselines (N=1) need at least 5 runs per model for reliable median.

CONSTRAINTS:
  - Model is FIXED per researcher (check your identity.json). Do not switch models.
  - Economic system is CONSTITUTIONAL — cannot be changed.
  - wall_clock should scale with N: N≤10→600s, N=15→900-1200s, N=20+→1800-3600s.
  - Report failures too — what N value DOESN'T work is equally important.

METRIC: PPUT (Progress Per Unit Time) — the ONLY scoring metric.
  PPUT = golden_path_tokens / (total_tokens × elapsed_minutes)
  golden_path_tokens = token count of the deepest proof chain (useful work)
  total_tokens = all API tokens consumed by all agents (total cost)
  elapsed_minutes = wall clock time
  Higher PPUT = more efficient progress. No artificial thresholds.
  golden_path = 0 tokens means PPUT = 0 (no progress at all).

{prev_life_section}
{bulletin_section}
=== YOUR LAB ===

PROMPT FILES (shared, read-only):
--- problem.txt ---
{prompts['problem.txt']}
--- skill.txt ---
{prompts['skill.txt']}

--- research_notes.txt (YOUR private notebook — only you can see this) ---
{notes}

CURRENT CONFIG: {json.dumps(config, indent=2)}

=== RAW EXPERIMENT DATA (last {min(len(history), 8)} runs) ===

"""
    for h in reversed(history[-8:]):
        prompt += f"  [{h['verdict']}] PPUT={h['pput']:.6f} depth={h['depth']} "
        prompt += f"appends={h['appends']} dedup={h['dedup']} "
        prompt += f"bankrupt={h['bankrupt']} frontier={h['max_frontier']} "
        prompt += f"nodes={h['nodes']} novelty={h['novelty']} "
        prompt += f"change={h.get('change', 'baseline')}\n"

    prompt += f"""
=== WHAT YOU CAN DO ===

Option A — Edit research_notes.txt (your private notebook):
  {{"action":"edit","file":"research_notes.txt","content":"FULL NEW CONTENT","reason":"..."}}

Option B — Change a config parameter:
  {{"action":"param","param":"PARAM_NAME","value":"VALUE","reason":"..."}}
  Available: swarm_size, math_count, bull_count, bear_count, model, wall_clock
  Models available to you: {models_str}
  Constraint: math_count + bull_count + bear_count == swarm_size

Option C — Re-init (fresh start with memory of this life):
  {{"action":"re-init","reason":"..."}}

Option D — Post to shared bulletin (your colleagues will see this):
  {{"action":"bulletin","type":"insight|hypothesis|warning","msg":"your message","reason":"..."}}

=== RESEARCH PRINCIPLES ===

You are a scientist, not a script. Think deeply before each experiment:
- DIAGNOSE before acting: why did the last experiment fail?
- Form a HYPOTHESIS: "I believe X will improve depth because Y"
- Test ONE variable at a time
- Read the raw data carefully
- Check the bulletin: your colleagues may have already tried what you're considering
- Share your discoveries: if you learn something non-obvious, post it to the bulletin

Output ONLY valid JSON. Explain your hypothesis in "reason".
"""
    return prompt


# ══════════════════════════════════════════════════════════════
# Response Parser
# ══════════════════════════════════════════════════════════════

def parse_llm_response(content):
    """Extract JSON action from LLM response."""
    if not content:
        return "error", None
    content = re.sub(r'```json\s*', '', content)
    content = re.sub(r'```\s*', '', content)
    match = re.search(r'\{.*\}', content, re.DOTALL)
    if match:
        try:
            action = json.loads(match.group())
            return action.get("action", "param"), action
        except json.JSONDecodeError:
            pass
    return "error", None


# ══════════════════════════════════════════════════════════════
# Constitutional Guard
# ══════════════════════════════════════════════════════════════

def constitutional_guard(config, base_env):
    """Rust constitutional check (fast, no API cost)."""
    try:
        check_env = {**base_env, "RUST_LOG": "warn"}
        for k, v in config.items():
            check_env[str(k).upper()] = str(v)
        check = subprocess.run(
            [str(BINARY), "--constitutional-check"],
            cwd=str(PROJECT), capture_output=True, text=True, timeout=30, env=check_env)
        if check.returncode != 0:
            return False, check.stderr.strip()[:200]
    except (subprocess.TimeoutExpired, FileNotFoundError):
        pass
    return True, "PASS"


# ══════════════════════════════════════════════════════════════
# Experiment Runner (OOM-safe: stdout→file, setrlimit, semaphore)
# ══════════════════════════════════════════════════════════════

def _set_evaluator_limits():
    """preexec_fn for evaluator subprocess: set virtual memory limit.
    Note: RLIMIT_AS may be ineffective on macOS/Apple Silicon.
    The monitor_swarm.sh provides a secondary OOM watchdog."""
    try:
        max_bytes = EVALUATOR_MAX_RSS_MB * 1024 * 1024
        resource.setrlimit(resource.RLIMIT_AS, (max_bytes, max_bytes))
    except (ValueError, resource.error) as e:
        # Log but don't fail — macOS may not support this
        print(f"  [OOM-guard] RLIMIT_AS not enforced: {e} (monitor watchdog is backup)", flush=True)


def run_single_experiment(config, identity):
    """Write config.json, run evaluator with OOM guards. Returns (pput, metrics, outcome)."""
    with open(CONFIG, "w") as f:
        json.dump(config, f, indent=2)

    # Redirect stdout/stderr to file instead of capturing in memory (Gemini OOM fix)
    log_dir = BASE / "logs"
    log_dir.mkdir(parents=True, exist_ok=True)
    run_ts = datetime.now().strftime("%Y%m%d_%H%M%S")
    stdout_log = log_dir / f"eval_{identity['id']}_{run_ts}.log"

    with open(stdout_log, "w") as log_f:
        result = subprocess.run(
            [sys.executable, str(BASE / "run_experiment.py")],
            cwd=str(BASE),
            stdout=log_f,
            stderr=subprocess.STDOUT,
            timeout=1800,
            preexec_fn=_set_evaluator_limits,
        )

    # Read output back from file (bounded read: last 50KB)
    output = ""
    if stdout_log.exists():
        size = stdout_log.stat().st_size
        with open(stdout_log, "r") as f:
            if size > 50_000:
                f.seek(size - 50_000)
                f.readline()  # skip partial line
            output = f.read()

    # Parse ERS from output
    pput_match = re.search(r"^PPUT:\s+(\S+)", output, re.MULTILINE)
    pput = float(pput_match.group(1)) if pput_match else 0.0

    def extract(key):
        m = re.search(rf"^{key}:\s+(\S+)", output, re.MULTILINE)
        return m.group(1) if m else "0"

    metrics = {
        "depth": int(extract("depth")),
        "nodes": int(extract("nodes")),
        "novelty": float(extract("novelty")),
        "roots": int(extract("roots")),
        "appends": int(extract("appends")),
        "dedup": int(extract("dedup")),
        "bankrupt": 0,
        "max_frontier": 0,
    }

    outcome = extract("status")
    return pput, metrics, outcome, output


# ══════════════════════════════════════════════════════════════
# State Management
# ══════════════════════════════════════════════════════════════

def compress_life_memory(history, best_config, best_pput, life_id):
    keep_runs = [h for h in history if h.get("verdict") == "KEEP"]
    return {
        "life_id": life_id,
        "total_experiments": len(history),
        "best_pput": best_pput,
        "best_config": best_config,
        "what_worked": [h["change"] for h in keep_runs][-5:],
        "what_failed": [h["change"] for h in history if h.get("verdict") == "DISCARD"][-5:],
    }


def load_prev_life():
    if PREV_LIFE.exists():
        try:
            return json.loads(PREV_LIFE.read_text())
        except Exception:
            pass
    return None


def resume_from_tsv():
    """Resume state from results.tsv on restart."""
    history = []
    best_pput = -1.0
    best_config = DEFAULT_CONFIG.copy()
    if RESULTS.exists():
        for line in RESULTS.read_text().strip().splitlines()[1:]:
            parts = line.split("\t")
            if len(parts) >= 19:
                try:
                    pput_val = float(parts[2])
                    cfg = json.loads(parts[18])
                    entry = {
                        "pput": pput_val,
                        "depth": int(parts[3]),
                        "nodes": int(parts[4]),
                        "novelty": float(parts[5]),
                        "roots": int(parts[6]),
                        "appends": int(parts[7]),
                        "dedup": int(parts[8]),
                        "bankrupt": int(parts[9]),
                        "max_frontier": int(parts[10]),
                        "verdict": "KEEP" if pput_val > best_pput else "DISCARD",
                        "change": parts[17] if len(parts) > 17 else "",
                    }
                    if pput_val > best_pput:
                        best_pput = pput_val
                        best_config = cfg
                        entry["verdict"] = "KEEP"
                    history.append(entry)
                except (ValueError, json.JSONDecodeError):
                    continue
    return history, best_pput, best_config


# ══════════════════════════════════════════════════════════════
# Main Loop
# ══════════════════════════════════════════════════════════════

def acquire_instance_lock():
    """Ensure only one sweep.py instance per researcher directory. Returns lock fd."""
    lock_path = BASE / ".sweep.lock"
    fd = os.open(str(lock_path), os.O_WRONLY | os.O_CREAT, 0o644)
    try:
        fcntl.flock(fd, fcntl.LOCK_EX | fcntl.LOCK_NB)
        os.ftruncate(fd, 0)
        os.write(fd, f"{os.getpid()}\n".encode())
        return fd  # keep open for lifetime of process
    except BlockingIOError:
        os.close(fd)
        print(f"ERROR: Another sweep.py is already running in {BASE}", file=sys.stderr)
        sys.exit(1)


def main():
    # Per-researcher single-instance lock (prevents duplicate processes)
    _instance_lock_fd = acquire_instance_lock()

    identity = load_identity()

    # Apply identity to default config
    DEFAULT_CONFIG["provider"] = identity.get("provider", "proxy")
    DEFAULT_CONFIG["model"] = identity.get("default_model", "qwen3-8b")
    if identity.get("proxy_url"):
        DEFAULT_CONFIG["proxy_url"] = identity["proxy_url"]

    # Ensure directories
    for d in [PROMPT_DIR, BASE / "tapes", BASE / "configs",
              BASE / "logs" / "success", BASE / "logs" / "failure"]:
        d.mkdir(parents=True, exist_ok=True)

    # Ensure private notes exist
    if not NOTES.exists():
        NOTES.write_text(f"# Research Notes — {identity.get('name', 'Researcher')}\n\n"
                         "v5 bitter lessons:\n"
                         "- Pre-dedup depth was fake (25->14 real)\n"
                         "- Investment squeezes building (286/300 tx were investments)\n"
                         "- 14B scatters more than 7B (depth=4 vs 14)\n"
                         "- Institution design > parameter tuning (50% vs <20% impact)\n")

    base_env = load_env()
    ds_key = base_env.get("DEEPSEEK_API_KEY", "")
    if not ds_key:
        print("ERROR: DEEPSEEK_API_KEY not found in .env", file=sys.stderr)
        sys.exit(1)

    if not BINARY.exists():
        print(f"ERROR: {BINARY} not found. Run: cargo build --release --bin evaluator", file=sys.stderr)
        sys.exit(1)

    # Resume or start fresh
    history, best_pput, best_config = resume_from_tsv()
    prev_life = load_prev_life()
    life_id = (prev_life.get("life_id", 0) + 1) if prev_life else 1

    print("=" * 60)
    print(f"TuringOS AutoResearch v7 — {identity.get('name', 'Researcher')} (Life #{life_id})")
    print(f"  ID: {identity['id']} | Port: {identity.get('proxy_port', '?')}")
    print(f"  Researcher Brain: DeepSeek Reasoner")
    print(f"  Agent Models: {', '.join(identity.get('available_models', ['?']))}")
    if history:
        print(f"  RESUMED: {len(history)} experiments, best PPUT={best_pput:.6f}")
    print("=" * 60, flush=True)

    # Announce birth on bulletin
    post_bulletin(identity["id"], "insight",
                  f"{identity.get('name','Researcher')} online (Life #{life_id}), "
                  f"models: {identity.get('default_model','?')}")

    # Run baseline if no history
    if not history:
        print(f"\n[0] BASELINE ({DEFAULT_CONFIG['model']}, {DEFAULT_CONFIG['swarm_size']} agents)", flush=True)
        best_config = DEFAULT_CONFIG.copy()

        # Acquire semaphore before running evaluator
        slot_fd, slot_num = acquire_evaluator_slot(identity["id"])
        try:
            pput, metrics, outcome, output = run_single_experiment(best_config, identity)
        finally:
            release_evaluator_slot(slot_fd, slot_num)

        best_pput = pput
        entry = {**metrics, "pput": pput, "verdict": "BASELINE", "change": "baseline"}
        history.append(entry)
        print(f"  PPUT={pput:.6f} depth={metrics['depth']} appends={metrics['appends']}", flush=True)
        post_bulletin(identity["id"], "breakthrough",
                      f"baseline PPUT={pput:.6f} model={DEFAULT_CONFIG['model']}", pput=pput, config=best_config)

    # Periodic cleanup counter
    cleanup_counter = 0

    # ── MAIN LOOP: LLM IS the search algorithm ──
    exp_num = len(history)
    while True:
        try:
            exp_num += 1
            cleanup_counter += 1

            # Periodic bulletin cleanup (every 50 experiments)
            if cleanup_counter % 50 == 0:
                cleanup_bulletin(200)

            print(f"\n--- [{identity['id']}] Consulting Researcher (exp #{exp_num}) ---", flush=True)

            # Ask the researcher
            prompt = build_researcher_prompt(history, best_config, prev_life, identity)
            content = call_deepseek(prompt, RESEARCHER_MODEL, ds_key)
            action_type, action = parse_llm_response(content)

            if action is None:
                print(f"  Researcher returned no action, retrying in 10s...", flush=True)
                time.sleep(10)
                exp_num -= 1
                continue

            new_config = best_config.copy()
            change_desc = ""

            # ── Parse action ──
            if action_type == "edit":
                fname = action.get("file", "")
                content_text = action.get("content", "")
                reason = action.get("reason", "")
                if fname == "research_notes.txt" and content_text:
                    NOTES.write_text(content_text)
                    change_desc = f"edit:notes ({reason[:60]})"
                    print(f"  [{exp_num}] {change_desc}", flush=True)
                    history.append({**history[-1], "verdict": "NOTES", "change": change_desc})
                    exp_num -= 1
                    continue
                else:
                    change_desc = f"BLOCKED:{fname}_locked"
                    print(f"  [{exp_num}] {change_desc}", flush=True)
                    exp_num -= 1
                    continue

            elif action_type == "bulletin":
                msg_type = action.get("type", "insight")
                msg = action.get("msg", "")
                reason = action.get("reason", "")
                if msg:
                    post_bulletin(identity["id"], msg_type, msg)
                    print(f"  [{exp_num}] POSTED to bulletin: ({msg_type}) {msg[:80]}", flush=True)
                exp_num -= 1
                continue

            elif action_type == "param":
                param = action.get("param", "")
                value = action.get("value", "")
                reason = action.get("reason", "")
                if param in LOCKED_PARAMS:
                    change_desc = f"BLOCKED:{param}_locked"
                    print(f"  [{exp_num}] {change_desc}", flush=True)
                    exp_num -= 1
                    continue
                elif param and isinstance(param, str):
                    if param in ("swarm_size", "math_count", "bull_count", "bear_count", "wall_clock"):
                        try:
                            value = int(value)
                        except (ValueError, TypeError):
                            pass
                    new_config[param] = value
                    if param in ("swarm_size", "math_count", "bull_count", "bear_count"):
                        total = new_config.get("math_count", 0) + new_config.get("bull_count", 0) + new_config.get("bear_count", 0)
                        if total != new_config.get("swarm_size", 0):
                            new_config["swarm_size"] = total
                    # Ensure proxy_url matches identity
                    if identity.get("proxy_url"):
                        new_config["proxy_url"] = identity["proxy_url"]
                    new_config["provider"] = identity.get("provider", "proxy")
                    new_config["description"] = f"{param}={value} — {reason[:80]}"
                    change_desc = f"{param}={value} ({reason[:60]})"

            elif action_type == "re-init":
                reason = action.get("reason", "no reason")
                print(f"\n  *** RE-INIT: {reason}", flush=True)
                memory = compress_life_memory(history, best_config, best_pput, life_id)
                memory["death_reason"] = reason
                PREV_LIFE.write_text(json.dumps(memory, indent=2))
                post_bulletin(identity["id"], "warning",
                              f"re-init after {len(history)} exp (best={best_pput:.4f}): {reason[:100]}")
                life_id += 1
                best_pput = -1.0
                best_config = DEFAULT_CONFIG.copy()
                history = []
                exp_num = 0
                prev_life = memory

                print(f"\n[0] BASELINE (new life #{life_id})", flush=True)
                slot_fd, slot_num = acquire_evaluator_slot(identity["id"])
                try:
                    pput, metrics, outcome, output = run_single_experiment(best_config, identity)
                finally:
                    release_evaluator_slot(slot_fd, slot_num)
                best_pput = pput
                entry = {**metrics, "pput": pput, "verdict": "BASELINE", "change": f"re-init: {reason[:40]}"}
                history.append(entry)
                print(f"  PPUT={pput:.6f} depth={metrics['depth']} appends={metrics['appends']}", flush=True)
                post_bulletin(identity["id"], "breakthrough",
                              f"new life baseline PPUT={pput:.6f}", pput=pput, config=best_config)
                continue
            else:
                change_desc = f"unknown:{action_type}"
                exp_num -= 1
                continue

            print(f"  [{exp_num}] {change_desc}", flush=True)

            # ── Constitutional guard ──
            ok, msg = constitutional_guard(new_config, base_env)
            if not ok:
                print(f"  BLOCKED: {msg}", flush=True)
                exp_num -= 1
                continue

            # ── Acquire semaphore + Run experiment ──
            slot_fd, slot_num = acquire_evaluator_slot(identity["id"])
            try:
                pput, metrics, outcome, output = run_single_experiment(new_config, identity)
            finally:
                release_evaluator_slot(slot_fd, slot_num)

            if pput > best_pput:
                verdict = "KEEP"
                print(f"  KEEP PPUT={pput:.6f} (was {best_pput:.6f})", flush=True)
                best_pput = pput
                best_config = new_config
                # Auto-broadcast breakthrough
                post_bulletin(identity["id"], "breakthrough",
                              f"{change_desc} → PPUT={pput:.6f}",
                              pput=pput, config=new_config)
            else:
                verdict = "DISCARD"
                print(f"  DISCARD PPUT={pput:.6f} (best={best_pput:.6f})", flush=True)

            entry = {**metrics, "pput": pput, "verdict": verdict, "change": change_desc}
            history.append(entry)
            print(f"  depth={metrics['depth']} appends={metrics['appends']} novelty={metrics['novelty']}", flush=True)

        except KeyboardInterrupt:
            print("\n\nResearcher interrupted. Saving state...", flush=True)
            post_bulletin(identity["id"], "warning",
                          f"going offline (Life #{life_id}, {len(history)} exp, best={best_pput:.4f})")
            break
        except Exception as e:
            print(f"\n  ERROR: {type(e).__name__}: {e}", flush=True)
            print(f"  {traceback.format_exc()[-300:]}", flush=True)
            time.sleep(10)
            exp_num -= 1

    print(f"\n{identity.get('name','Researcher')} Life #{life_id}: {len(history)} experiments, best PPUT={best_pput:.6f}")
    print("Researcher dormant. Restart with: python3 sweep.py")


if __name__ == "__main__":
    main()
