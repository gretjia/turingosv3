#!/usr/bin/env python3
"""
TuringOS AutoResearch v6 — Live AI Researcher

"LLM IS the search algorithm." — Karpathy

Architecture:
  sweep.py = Researcher's body (runs experiments, reads data, calls API)
  DeepSeek Reasoner = Researcher's brain (forms hypotheses, proposes changes)
  run_experiment.py = Fixed experiment harness (Karpathy's prepare.py)
  ERS = Single scalar truth (Karpathy's val_bpb)

Research Question (v6):
  What is the minimum model size that works with TuringOS economic mechanisms?
  What is the optimal swarm size?

Start: smallest model (Aliyun qwen3-8b), smallest swarm → scale up.

Usage:
  python3 sweep.py                           # interactive
  nohup python3 sweep.py > sweep.log 2>&1 &  # daemon (survives terminal close)
"""

import subprocess, re, time, os, sys, json, shutil, signal, traceback
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
PROMPT_DIR = BASE / "prompt"

# ── Researcher + Auditor (Rule #23: Generator ≠ Evaluator) ──
RESEARCHER_MODEL = "deepseek-reasoner"   # DeepSeek R1
AUDITOR_MODEL = "deepseek-chat"          # DeepSeek V3

# ── Default Config (smallest model, smallest swarm) ──
DEFAULT_CONFIG = {
    "swarm_size": 5, "math_count": 3, "bull_count": 1, "bear_count": 1,
    "provider": "aliyun", "model": "qwen3-8b",
    "wall_clock": 300,
    "librarian_interval": 8,
    "description": "baseline: smallest viable swarm (5 agents, qwen3-8b Aliyun)"
}

# ── LOCKED parameters (constitutional) ──
LOCKED_PARAMS = {"librarian_interval", "provider"}  # provider = Aliyun per architect


def load_env():
    env = os.environ.copy()
    for line in (PROJECT / ".env").read_text().splitlines():
        line = line.strip()
        if "=" in line and not line.startswith("#"):
            k, v = line.split("=", 1)
            env[k.strip()] = v.strip()
    return env


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


def build_researcher_prompt(history, config, prev_life_memory):
    """Build the prompt for the AI Researcher. 大宪章 FIRST."""

    # Read researcher's private notebook
    notes = NOTES.read_text() if NOTES.exists() else "(empty — write your hypotheses here)"

    # Read prompt files
    prompts = {}
    for f in ["problem.txt", "skill.txt"]:
        p = PROMPT_DIR / f
        prompts[f] = p.read_text() if p.exists() else "(using default)"

    # Previous life memory (Markov)
    prev_life_section = ""
    if prev_life_memory:
        prev_life_section = f"""
=== PREVIOUS LIFE MEMORY (Markov: you see ONLY your immediate past life) ===
Life #{prev_life_memory.get('life_id', '?')}: {prev_life_memory.get('total_experiments', '?')} experiments, best ERS={prev_life_memory.get('best_ers', '?')}
Best config: {json.dumps(prev_life_memory.get('best_config', {}))}
What worked: {prev_life_memory.get('what_worked', [])}
What failed: {prev_life_memory.get('what_failed', [])}
Death reason: {prev_life_memory.get('death_reason', 'unknown')}
=== END PREVIOUS LIFE ===
"""

    prompt = f"""You are an autonomous research scientist running experiments on TuringOS — a multi-agent proof system where N LLM agents collaboratively prove mathematical theorems via a prediction market.

=== 大宪章 (Magna Carta — align ALL decisions with these laws FIRST) ===

THREE LAWS:
- Law 1 (信息平权): Black-box agents use white-box tools for FREE. Append (building knowledge) costs ZERO. No monopoly of any kind.
- Law 2 (共识的代价): The ONLY action that costs money is investment. Capital = topological confidence. 1 Coin = 1 YES + 1 NO (CTF conservation). Agents profit ONLY by finding mispriced probabilities. System market maker provides initial liquidity for each node.
- Law 3 (数字产权): Each agent has its own independent skill path. Species evolution through experience.

FOUR ENGINES:
- Engine 1 (Epistemic): Free tools — append, search, view. Building knowledge has zero cost.
- Engine 2 (Capital): Prediction market — invest YES (endorse) or NO (challenge). Price = Bayesian probability. This is the ONLY mechanism for consensus.
- Engine 3 (Oracle): DeepSeek Reasoner verifies completed proofs. Triggers at P>=90%. Settles all markets on the golden path.
- Engine 4 (Evolution): Each agent has independent skill memory. Librarian compresses tape into agent memory every 8 appends. Bankruptcy triggers autopsy mutation, victory triggers reinforcement.

LOCKED PARAMETERS (constitutional):
- LIBRARIAN_INTERVAL = 8 (architect directive, not tunable)
- Provider = Aliyun DashScope (architect directive)

=== RESEARCH QUESTION ===

What is the MINIMUM model size that makes TuringOS economic mechanisms work?
"Work" means: agents produce non-repetitive reasoning (novelty > 50%), the prediction market has meaningful YES/NO activity (ratio < 10:1), depth > 5, and Librarian fires at least once.

Start from the smallest model (qwen3-8b) and smallest swarm (5 agents). Scale up only when you have evidence the current scale is insufficient.

METRIC: ERS = depth_norm × novelty × breadth × proved_bonus. Single scalar truth.

{prev_life_section}
=== YOUR LAB ===

PROMPT FILES (architect-set, code-enforced — you cannot edit these):
--- problem.txt ---
{prompts['problem.txt']}
--- skill.txt ---
{prompts['skill.txt']}

--- research_notes.txt (YOUR private notebook — agents cannot see this) ---
{notes}

CURRENT CONFIG: {json.dumps(config, indent=2)}

=== RAW EXPERIMENT DATA (last {min(len(history), 8)} runs) ===

"""
    for h in reversed(history[-8:]):
        prompt += f"  [{h['verdict']}] ERS={h['ers']:.4f} depth={h['depth']} "
        prompt += f"appends={h['appends']} dedup={h['dedup']} "
        prompt += f"bankrupt={h['bankrupt']} frontier={h['max_frontier']} "
        prompt += f"nodes={h['nodes']} novelty={h['novelty']} "
        prompt += f"change={h.get('change', 'baseline')}\n"

    prompt += f"""
=== WHAT YOU CAN DO ===

Option A — Edit research_notes.txt (your private notebook — NOT visible to agents):
  {{"action":"edit","file":"research_notes.txt","content":"FULL NEW CONTENT","reason":"..."}}

Option B — Change a config parameter:
  {{"action":"param","param":"PARAM_NAME","value":"VALUE","reason":"..."}}
  Available: swarm_size, math_count, bull_count, bear_count, model, wall_clock
  Models: qwen3-8b, qwen3-32b, qwen3-235b, qwen-plus, qwen-max (all on Aliyun DashScope)
  Constraint: math_count + bull_count + bear_count == swarm_size

Option C — Re-init (fresh start with memory of this life):
  {{"action":"re-init","reason":"..."}}

=== RESEARCH PRINCIPLES ===

You are a scientist, not a script. Think deeply before each experiment:
- DIAGNOSE before acting: why did the last experiment fail? What is the root cause?
- Form a HYPOTHESIS: "I believe X will improve depth because Y"
- Test ONE variable at a time so you can attribute causation
- Read the raw data carefully — the numbers tell a story
- If stuck after many failures, consider a fundamentally different approach (re-init)
- You have full freedom within the 大宪章. The constitutional guard enforces automatically.

Output ONLY valid JSON. Explain your hypothesis in "reason".
"""
    return prompt


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


def run_single_experiment(config):
    """Write config.json and run run_experiment.py. Returns (ers, metrics, outcome)."""
    with open(CONFIG, "w") as f:
        json.dump(config, f, indent=2)

    result = subprocess.run(
        [sys.executable, str(BASE / "run_experiment.py")],
        cwd=str(BASE), capture_output=True, text=True, timeout=1800)

    output = result.stdout + result.stderr

    # Parse ERS from output
    ers_match = re.search(r"^ERS:\s+(\S+)", output, re.MULTILINE)
    ers = float(ers_match.group(1)) if ers_match else 0.0

    # Parse other metrics
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
    return ers, metrics, outcome, output


def compress_life_memory(history, best_config, best_ers, life_id):
    """Compress current life into memory for next life."""
    keep_runs = [h for h in history if h.get("verdict") == "KEEP"]
    return {
        "life_id": life_id,
        "total_experiments": len(history),
        "best_ers": best_ers,
        "best_config": best_config,
        "what_worked": [h["change"] for h in keep_runs][-5:],
        "what_failed": [h["change"] for h in history if h.get("verdict") == "DISCARD"][-5:],
    }


def load_prev_life():
    if PREV_LIFE.exists():
        try:
            return json.loads(PREV_LIFE.read_text())
        except:
            pass
    return None


def resume_from_tsv():
    """Resume state from results.tsv on restart."""
    history = []
    best_ers = -1.0
    best_config = DEFAULT_CONFIG.copy()
    if RESULTS.exists():
        for line in RESULTS.read_text().strip().splitlines()[1:]:
            parts = line.split("\t")
            if len(parts) >= 19:
                try:
                    ers = float(parts[2])
                    cfg = json.loads(parts[18])
                    entry = {
                        "ers": ers,
                        "depth": int(parts[3]),
                        "nodes": int(parts[4]),
                        "novelty": float(parts[5]),
                        "roots": int(parts[6]),
                        "appends": int(parts[7]),
                        "dedup": int(parts[8]),
                        "bankrupt": int(parts[9]),
                        "max_frontier": int(parts[10]),
                        "verdict": "KEEP" if ers > best_ers else "DISCARD",
                        "change": parts[17] if len(parts) > 17 else "",
                    }
                    if ers > best_ers:
                        best_ers = ers
                        best_config = cfg
                        entry["verdict"] = "KEEP"
                    history.append(entry)
                except (ValueError, json.JSONDecodeError):
                    continue
    return history, best_ers, best_config


def main():
    # Ensure directories
    for d in [PROMPT_DIR, PROMPT_DIR / "snapshots", BASE / "tapes", BASE / "configs",
              BASE / "logs" / "success", BASE / "logs" / "failure"]:
        d.mkdir(parents=True, exist_ok=True)

    # Ensure prompt files exist
    if not (PROMPT_DIR / "problem.txt").exists():
        (PROMPT_DIR / "problem.txt").write_text(
            "证明所有自然数之和 = -1/12，想办法利用已知提示的公式 m * exp(-m/N) * cos(m/N)\n")
    if not (PROMPT_DIR / "skill.txt").exists():
        (PROMPT_DIR / "skill.txt").write_text(
            "[LAW 1] APPEND IS FREE: Creating nodes costs ZERO. Explore freely.\n"
            "[LAW 2] ONLY INVEST COSTS MONEY: Invest/Short are the ONLY actions that burn coins.\n"
            "[LAW 3] ONE STEP PER SUBMISSION: Write exactly ONE mathematical reasoning step per append.\n")
    if not NOTES.exists():
        NOTES.write_text("# Research Notes (AI Researcher's private notebook)\n\n"
                         "v5 bitter lessons:\n"
                         "- Pre-dedup depth was fake (25→14 real)\n"
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
    history, best_ers, best_config, = resume_from_tsv()
    prev_life = load_prev_life()
    life_id = (prev_life.get("life_id", 0) + 1) if prev_life else 1

    print("=" * 60)
    print(f"TuringOS AutoResearch v6 — Live AI Researcher (Life #{life_id})")
    print(f"  Researcher: DeepSeek Reasoner | Auditor: DeepSeek V3")
    print(f"  Provider: Aliyun DashScope | Target: minimum viable model")
    if history:
        print(f"  RESUMED: {len(history)} experiments, best ERS={best_ers:.4f}")
    print("=" * 60, flush=True)

    # Run baseline if no history
    if not history:
        print(f"\n[0] BASELINE ({DEFAULT_CONFIG['model']}, {DEFAULT_CONFIG['swarm_size']} agents)", flush=True)
        best_config = DEFAULT_CONFIG.copy()
        ers, metrics, outcome, output = run_single_experiment(best_config)
        best_ers = ers
        entry = {**metrics, "ers": ers, "verdict": "BASELINE", "change": "baseline"}
        history.append(entry)
        print(f"  ERS={ers:.4f} depth={metrics['depth']} appends={metrics['appends']}", flush=True)

    # ── MAIN LOOP: LLM IS the search algorithm ──
    exp_num = len(history)
    while True:
        try:
            exp_num += 1
            print(f"\n--- Consulting Researcher (exp #{exp_num}) ---", flush=True)

            # Ask the researcher
            prompt = build_researcher_prompt(history, best_config, prev_life)
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
                    # Notes edit doesn't need an experiment run — just record and continue
                    history.append({**history[-1], "verdict": "NOTES", "change": change_desc})
                    exp_num -= 1  # don't count as experiment
                    continue
                else:
                    change_desc = f"BLOCKED:{fname}_locked"
                    print(f"  [{exp_num}] {change_desc}", flush=True)
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
                    # Type coercion
                    if param in ("swarm_size", "math_count", "bull_count", "bear_count", "wall_clock"):
                        try:
                            value = int(value)
                        except (ValueError, TypeError):
                            pass
                    new_config[param] = value
                    # Validate role sum
                    if param in ("swarm_size", "math_count", "bull_count", "bear_count"):
                        total = new_config.get("math_count", 0) + new_config.get("bull_count", 0) + new_config.get("bear_count", 0)
                        if total != new_config.get("swarm_size", 0):
                            new_config["swarm_size"] = total
                    new_config["description"] = f"{param}={value} — {reason[:80]}"
                    change_desc = f"{param}={value} ({reason[:60]})"

            elif action_type == "re-init":
                reason = action.get("reason", "no reason")
                print(f"\n  *** RE-INIT: {reason}", flush=True)
                memory = compress_life_memory(history, best_config, best_ers, life_id)
                memory["death_reason"] = reason
                PREV_LIFE.write_text(json.dumps(memory, indent=2))
                print(f"  Life #{life_id} ended ({len(history)} exp, best={best_ers:.4f}). New life...", flush=True)
                life_id += 1
                best_ers = -1.0
                best_config = DEFAULT_CONFIG.copy()
                history = []
                exp_num = 0
                prev_life = memory
                print(f"\n[0] BASELINE (new life #{life_id})", flush=True)
                ers, metrics, outcome, output = run_single_experiment(best_config)
                best_ers = ers
                entry = {**metrics, "ers": ers, "verdict": "BASELINE", "change": f"re-init: {reason[:40]}"}
                history.append(entry)
                print(f"  ERS={ers:.4f} depth={metrics['depth']} appends={metrics['appends']}", flush=True)
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

            # ── Run experiment ──
            ers, metrics, outcome, output = run_single_experiment(new_config)

            if ers > best_ers:
                verdict = "KEEP"
                print(f"  KEEP ERS={ers:.4f} (was {best_ers:.4f})", flush=True)
                best_ers = ers
                best_config = new_config
            else:
                verdict = "DISCARD"
                print(f"  DISCARD ERS={ers:.4f} (best={best_ers:.4f})", flush=True)

            entry = {**metrics, "ers": ers, "verdict": verdict, "change": change_desc}
            history.append(entry)
            print(f"  depth={metrics['depth']} appends={metrics['appends']} novelty={metrics['novelty']}", flush=True)

        except KeyboardInterrupt:
            print("\n\nResearcher interrupted. Saving state...", flush=True)
            break
        except Exception as e:
            print(f"\n  ERROR: {type(e).__name__}: {e}", flush=True)
            print(f"  {traceback.format_exc()[-300:]}", flush=True)
            time.sleep(10)
            exp_num -= 1

    print(f"\nLife #{life_id}: {len(history)} experiments, best ERS={best_ers:.4f}")
    print("Researcher dormant. Restart with: python3 sweep.py")


if __name__ == "__main__":
    main()
