#!/usr/bin/env python3
"""
TuringOS AutoResearch — Experiment Runner (DO NOT MODIFY)

This is the evaluation harness. It reads config.json, runs the evaluator
binary, parses the output, computes metrics, and prints a standardized
summary. Equivalent to Karpathy's prepare.py — fixed infrastructure.

Usage: python3 run_experiment.py
       python3 run_experiment.py > run.log 2>&1
"""

import subprocess, re, json, time, os, math, sys
from pathlib import Path
from collections import defaultdict

# ── Paths ──
PROJECT = Path(__file__).resolve().parent.parent.parent  # turingosv3/
BINARY = PROJECT / "target/release/evaluator"
CONFIG = Path(__file__).resolve().parent / "config.json"
TAPE = Path("/tmp/zeta_sum_tape_full.md")
LOG_DIR = Path(__file__).resolve().parent / "logs"


def load_env():
    """Load .env from project root"""
    env = os.environ.copy()
    env_file = PROJECT / ".env"
    if not env_file.exists():
        print("ERROR: .env not found at", env_file, file=sys.stderr)
        sys.exit(1)
    for line in env_file.read_text().splitlines():
        line = line.strip()
        if "=" in line and not line.startswith("#"):
            k, v = line.split("=", 1)
            env[k.strip()] = v.strip()
    return env


def load_config():
    """Load config.json"""
    if not CONFIG.exists():
        print("ERROR: config.json not found", file=sys.stderr)
        sys.exit(1)
    with open(CONFIG) as f:
        cfg = json.load(f)
    # Validate
    assert cfg["math_count"] + cfg["bull_count"] + cfg["bear_count"] == cfg["swarm_size"], \
        f"Role counts must sum to swarm_size: {cfg['math_count']}+{cfg['bull_count']}+{cfg['bear_count']} != {cfg['swarm_size']}"
    return cfg


def run_evaluator(cfg, base_env, timeout_secs=600):
    """Run the evaluator binary with config as env vars"""
    # Clear stale tape
    if TAPE.exists():
        TAPE.unlink()

    env = base_env.copy()
    env["RUST_LOG"] = "info"
    env["SWARM_SIZE"] = str(cfg["swarm_size"])
    env["MAX_TX"] = str(cfg["max_tx"])
    env["MATH_COUNT"] = str(cfg["math_count"])
    env["BULL_COUNT"] = str(cfg["bull_count"])
    env["BEAR_COUNT"] = str(cfg["bear_count"])
    # Provider config (defaults to Aliyun qwen3-8b)
    env["LLM_PROVIDER"] = cfg.get("provider", "aliyun")
    env["LLM_MODEL"] = cfg.get("model", "qwen3-8b")

    start = time.time()
    try:
        result = subprocess.run(
            [str(BINARY)], cwd=str(PROJECT),
            capture_output=True, text=True,
            timeout=max(timeout_secs, 1800), env=env  # 30min minimum for Aliyun's slow responses
        )
        output = result.stderr
    except subprocess.TimeoutExpired as e:
        output = (e.stderr or b"").decode(errors="replace")
        output += "\n[TIMEOUT]"

    elapsed = time.time() - start
    return output, elapsed


def parse_tape():
    """Parse /tmp/zeta_sum_tape_full.md → dict of nodes"""
    if not TAPE.exists():
        return {}
    nodes = {}
    current_id = None
    for line in TAPE.read_text().splitlines():
        m = re.match(
            r"### `(\S+)` \| Author: (\S+) \| Price: (\S+) \| Citations: \[([^\]]*)\]",
            line,
        )
        if m:
            current_id = m.group(1)
            cites = [c.strip().strip('"') for c in m.group(4).split(",") if c.strip().strip('"')]
            nodes[current_id] = {
                "author": m.group(2),
                "price": int(float(m.group(3))),
                "citations": cites,
                "payload": "",
            }
        elif current_id and line.startswith("```") and nodes.get(current_id, {}).get("payload", "").strip():
            current_id = None
        elif current_id and not line.startswith("```") and current_id in nodes and line.strip():
            nodes[current_id]["payload"] += line.strip() + " "
    return nodes


def analyze(nodes, log):
    """Compute all metrics from tape + log"""
    metrics = {}

    # ── Tape metrics ──
    if not nodes:
        return {
            "nodes": 0, "depth": 0, "roots": 0, "novelty": 0.0,
            "unique_prefixes": 0, "tx": 0, "generations": 0,
            "total_yes": 0, "buy_no": 0, "traded": 0, "proved": False,
        }

    children = defaultdict(list)
    roots = []
    for nid, n in nodes.items():
        if not n["citations"]:
            roots.append(nid)
        else:
            for p in n["citations"]:
                if p in nodes:
                    children[p].append(nid)

    # Max depth
    memo = {}
    def depth(nid):
        if nid in memo:
            return memo[nid]
        kids = children.get(nid, [])
        d = (1 + max(depth(k) for k in kids)) if kids else 0
        memo[nid] = d
        return d
    max_d = max((depth(r) for r in roots), default=0)

    # Novelty: unique 40-char prefixes
    prefixes = set()
    for n in nodes.values():
        prefixes.add(n["payload"].strip()[:30].lower())
    novelty = len(prefixes) / max(len(nodes), 1)

    metrics["nodes"] = len(nodes)
    metrics["depth"] = max_d
    metrics["roots"] = len(roots)
    metrics["novelty"] = round(novelty, 4)
    metrics["unique_prefixes"] = len(prefixes)

    # ── Log metrics ──
    match = re.search(r"EVALUATION COMPLETE \((\d+) tx, (\d+) gen", log)
    metrics["tx"] = int(match.group(1)) if match else 0
    metrics["generations"] = int(match.group(2)) if match else 0

    buy_yes = len(re.findall(r"BUY YES\]", log))
    auto_long = len(re.findall(r"AUTO-LONG\]", log))
    metrics["total_yes"] = buy_yes + auto_long
    metrics["buy_no"] = len(re.findall(r"BUY NO\]", log))
    metrics["proved"] = bool(re.search(r"Proof chain COMPLETE", log))

    traded = set()
    for m in re.finditer(r"BUY (?:YES|NO)\] \S+ bought .+? on (\S+) for", log):
        traded.add(m.group(1))
    for m in re.finditer(r"AUTO-LONG\] \S+ bought .+? on (\S+) for", log):
        traded.add(m.group(1))
    metrics["traded"] = len(traded)

    return metrics


def compute_ers(m):
    """
    Effective Reasoning Score

    ERS = depth_norm × novelty × breadth_factor × proved_bonus

    This metric rewards configs where the economic system CONSTRAINS agents
    to produce deep, novel, multi-strategy reasoning.
    """
    depth_norm = min(m["depth"], 15) / 15.0
    novelty = m["novelty"]
    breadth = min(m["roots"], 5) / 5.0
    proved_bonus = 1.5 if m["proved"] else 1.0
    return round(depth_norm * novelty * breadth * proved_bonus, 4)


def main():
    LOG_DIR.mkdir(parents=True, exist_ok=True)

    # Validate
    if not BINARY.exists():
        print(f"ERROR: Binary not found at {BINARY}", file=sys.stderr)
        print("Run: cd experiments/zeta_sum_proof && cargo build --release --bin evaluator",
              file=sys.stderr)
        sys.exit(1)

    cfg = load_config()
    base_env = load_env()

    ratio = f"{cfg['math_count']}M/{cfg['bull_count']}B+/{cfg['bear_count']}B-"
    print(f"=== TuringOS AutoResearch ===")
    print(f"Config: {cfg['swarm_size']} agents ({ratio}), {cfg['max_tx']}tx")
    print(f"Description: {cfg.get('description', 'n/a')}")
    print(f"Running...", flush=True)

    # Run
    log, elapsed = run_evaluator(cfg, base_env)

    # Parse & analyze FIRST (to determine success/failure)
    ts = int(time.time())
    nodes = parse_tape()
    if not nodes:
        appended = len(re.findall(r"Appended", log))
        if appended > 0:
            print(f"[WARN] Tape not found, using log fallback ({appended} tx appended)")
    m = analyze(nodes, log)
    ers = compute_ers(m)

    # Classify: success or failure
    is_success = m["proved"] or (m["depth"] >= 10 and m["novelty"] >= 0.5)
    outcome = "success" if is_success else "failure"
    outcome_dir = LOG_DIR / outcome
    outcome_dir.mkdir(parents=True, exist_ok=True)

    # Save log to success/ or failure/
    log_file = outcome_dir / f"run_{ts}.log"
    log_file.write_text(log)

    # Persist tape (NEVER rely on /tmp/ — Run 6 tape loss lesson)
    if TAPE.exists():
        import shutil
        tape_persist = outcome_dir / f"tape_{ts}.md"
        shutil.copy2(str(TAPE), str(tape_persist))
        print(f"[{outcome.upper()}] Log + tape saved to {outcome_dir}/", flush=True)
    else:
        print(f"[{outcome.upper()}] Log saved to {log_file} (no tape)", flush=True)

    # Print standardized summary (grep-friendly)
    ty = m["total_yes"]
    tn = max(m["buy_no"], 1)
    ratio_str = f"{ty/tn:.1f}:1"

    print("---")
    print(f"ERS:        {ers}")
    print(f"depth:      {m['depth']}")
    print(f"nodes:      {m['nodes']}")
    print(f"novelty:    {m['novelty']}")
    print(f"roots:      {m['roots']}")
    print(f"unique_pfx: {m['unique_prefixes']}")
    print(f"yes:        {ty}")
    print(f"no:         {m['buy_no']}")
    print(f"ratio:      {ratio_str}")
    print(f"traded:     {m['traded']}")
    print(f"proved:     {'YES' if m['proved'] else 'NO'}")
    print(f"tx_actual:  {m['tx']}")
    print(f"generations:{m['generations']}")
    print(f"elapsed_s:  {elapsed:.0f}")
    print(f"log_file:   {log_file}")


if __name__ == "__main__":
    main()
