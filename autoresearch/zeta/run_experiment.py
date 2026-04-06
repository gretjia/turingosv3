#!/usr/bin/env python3
"""
TuringOS AutoResearch — Experiment Runner (Karpathy's prepare.py)

Fixed evaluation harness. Reads config.json, runs the evaluator binary,
parses output, computes metrics, persists everything (WAL, tape, config, log).

This file is READ-ONLY infrastructure. The AI Researcher (sweep.py) edits
config.json and prompt files — never this file.

Usage:
  python3 run_experiment.py              # single run, appends to results.tsv
  python3 run_experiment.py > run.log    # capture stdout
"""

import subprocess, re, json, time, os, sys, shutil, signal
from pathlib import Path
from datetime import datetime
from collections import defaultdict

# ── Paths ──
BASE = Path(__file__).resolve().parent
PROJECT = BASE.parent.parent  # turingosv3/
BINARY = PROJECT / "target/release/evaluator"
CONFIG = BASE / "config.json"
RESULTS = BASE / "results.tsv"
LOG_DIR = BASE / "logs"
TAPES_DIR = BASE / "tapes"
CONFIGS_DIR = BASE / "configs"


def load_env():
    """Load .env from project root."""
    env = os.environ.copy()
    env_file = PROJECT / ".env"
    if not env_file.exists():
        print(f"ERROR: .env not found at {env_file}", file=sys.stderr)
        sys.exit(1)
    for line in env_file.read_text().splitlines():
        line = line.strip()
        if "=" in line and not line.startswith("#"):
            k, v = line.split("=", 1)
            env[k.strip()] = v.strip()
    return env


def load_config():
    """Load config.json."""
    if not CONFIG.exists():
        print("ERROR: config.json not found", file=sys.stderr)
        sys.exit(1)
    with open(CONFIG) as f:
        cfg = json.load(f)
    assert cfg["math_count"] + cfg["bull_count"] + cfg["bear_count"] == cfg["swarm_size"], \
        f"Role counts must sum to swarm_size"
    return cfg


def next_run_id():
    """Auto-increment from results.tsv last row."""
    if not RESULTS.exists():
        return 1
    lines = RESULTS.read_text().strip().splitlines()
    if len(lines) <= 1:  # header only
        return 1
    last = lines[-1].split("\t")
    try:
        return int(last[0]) + 1
    except (ValueError, IndexError):
        return len(lines)


def run_evaluator(cfg, base_env, run_id, timeout_secs=600):
    """Run the evaluator binary with per-run isolation."""
    # Per-run directories
    run_log_dir = LOG_DIR / f"run_{run_id:03d}"
    run_log_dir.mkdir(parents=True, exist_ok=True)

    tape_path = TAPES_DIR / f"tape_{run_id:03d}.md"
    wal_path = TAPES_DIR / f"wal_{run_id:03d}.json"

    env = base_env.copy()
    env["RUST_LOG"] = "info"
    env["SWARM_SIZE"] = str(cfg["swarm_size"])
    env["MATH_COUNT"] = str(cfg["math_count"])
    env["BULL_COUNT"] = str(cfg["bull_count"])
    env["BEAR_COUNT"] = str(cfg["bear_count"])
    env["LIBRARIAN_INTERVAL"] = str(cfg.get("librarian_interval", 8))

    # Provider config — default Aliyun (via proxy on Mac, direct on Linux)
    provider = cfg.get("provider", "aliyun")
    model = cfg.get("model", "qwen3-8b")
    env["LLM_PROVIDER"] = provider
    env["LLM_MODEL"] = model

    # Proxy mode: set LLM_URL for local HTTP proxy (V-007: Mac needs proxy for TLS)
    if provider == "proxy":
        proxy_url = cfg.get("proxy_url", "http://127.0.0.1:8088/v1/chat/completions")
        env["LLM_URL"] = proxy_url

    # Per-run isolation: WAL, tape dump, JSONL logs, skills
    env["WAL_PATH"] = str(wal_path)
    env["TAPE_OUTPUT"] = str(tape_path)
    env["LOG_DIR"] = str(run_log_dir)
    env["AGENT_SKILLS_DIR"] = str(run_log_dir / "skills")

    # Wall clock from config (sweep.py can override)
    wall_clock = int(cfg.get("wall_clock", timeout_secs))

    # Redirect evaluator stderr to file instead of buffering in memory (OOM guard)
    stderr_log = run_log_dir / "evaluator_stderr.log"

    start = time.time()
    try:
        stderr_file = open(stderr_log, "w")
        proc = subprocess.Popen(
            [str(BINARY)], cwd=str(PROJECT),
            stdout=subprocess.DEVNULL, stderr=stderr_file, env=env,
            preexec_fn=os.setsid)
        try:
            proc.wait(timeout=max(wall_clock, 120))
        except subprocess.TimeoutExpired:
            os.killpg(os.getpgid(proc.pid), signal.SIGTERM)
            time.sleep(3)  # give evaluator time to flush WAL + tape dump
            if proc.poll() is None:
                os.killpg(os.getpgid(proc.pid), signal.SIGKILL)
            proc.wait()
        stderr_file.close()

        # Read output from file (bounded: last 100KB to prevent OOM)
        if stderr_log.exists():
            size = stderr_log.stat().st_size
            with open(stderr_log, "r", errors="replace") as f:
                if size > 100_000:
                    f.seek(size - 100_000)
                    f.readline()  # skip partial line
                output = f.read()
            if proc.returncode is None or proc.returncode != 0:
                output += "\n[TIMEOUT]" if proc.returncode is None else ""
        else:
            output = ""
    except Exception as e:
        output = str(e)
        try:
            stderr_file.close()
        except Exception:
            pass

    elapsed = time.time() - start
    return output, elapsed, run_log_dir, tape_path, wal_path


def parse_tape(tape_path):
    """Parse tape markdown → dict of nodes."""
    if not tape_path.exists():
        return {}
    nodes = {}
    current_id = None
    for line in tape_path.read_text().splitlines():
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
    """Compute all metrics from tape + log."""
    if not nodes:
        return {
            "nodes": 0, "depth": 0, "roots": 0, "novelty": 0.0,
            "unique_prefixes": 0, "tx": 0, "generations": 0,
            "total_yes": 0, "buy_no": 0, "traded": 0, "proved": False,
            "appends": 0, "dedup": 0, "bankrupt": 0, "max_frontier": 0,
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

    # Novelty: unique 30-char prefixes
    prefixes = set()
    for n in nodes.values():
        prefixes.add(n["payload"].strip()[:30].lower())
    novelty = len(prefixes) / max(len(nodes), 1)

    m = {}
    m["nodes"] = len(nodes)
    m["depth"] = max_d
    m["roots"] = len(roots)
    m["novelty"] = round(novelty, 4)
    m["unique_prefixes"] = len(prefixes)

    # Log metrics
    match = re.search(r"EVALUATION COMPLETE \((\d+) appends?, (\d+) total", log)
    m["appends"] = int(match.group(1)) if match else len(re.findall(r"Appended", log))
    m["tx"] = int(match.group(2)) if match else 0
    m["generations"] = len(re.findall(r"Gen \d+ perished|Gen \d+ —", log))

    buy_yes = len(re.findall(r"BUY YES\]", log))
    auto_long = len(re.findall(r"AUTO-LONG\]", log))
    m["total_yes"] = buy_yes + auto_long
    m["buy_no"] = len(re.findall(r"BUY NO\]", log))
    m["proved"] = bool(re.search(r"Proof chain COMPLETE|OMEGA", log))
    m["dedup"] = len(re.findall(r"DEDUP\]|GLOBAL-DEDUP", log))
    m["bankrupt"] = len(re.findall(r"Global bankruptcy|All bankrupt", log))

    frontiers = [int(f) for f in re.findall(r"from (\d+) frontier", log)]
    m["max_frontier"] = max(frontiers, default=0)

    traded = set()
    for t in re.finditer(r"BUY (?:YES|NO)\] \S+ bought .+? on (\S+) for", log):
        traded.add(t.group(1))
    for t in re.finditer(r"AUTO-LONG\] \S+ bought .+? on (\S+) for", log):
        traded.add(t.group(1))
    m["traded"] = len(traded)

    return m


def extract_deepest_chain(nodes):
    """Trace the deepest chain in the DAG. Returns list of node IDs root→leaf."""
    if not nodes:
        return []
    root_dist = {}
    def compute_rd(nid):
        if nid in root_dist: return root_dist[nid]
        cites = [c for c in nodes[nid].get("citations", []) if c in nodes]
        if not cites: root_dist[nid] = 0; return 0
        d = 1 + max(compute_rd(c) for c in cites)
        root_dist[nid] = d
        return d
    for nid in nodes: compute_rd(nid)
    if not root_dist:
        return []
    deepest = max(root_dist, key=root_dist.get)
    chain = [deepest]
    while True:
        cites = [c for c in nodes[chain[-1]].get("citations", []) if c in nodes]
        if not cites: break
        chain.append(cites[0])
    return list(reversed(chain))


def query_proxy_stats(proxy_url):
    """Query proxy /stats endpoint for token counts."""
    import urllib.request
    try:
        stats_url = proxy_url.rsplit("/v1", 1)[0] + "/stats"
        with urllib.request.urlopen(stats_url, timeout=5) as resp:
            return json.loads(resp.read())
    except Exception:
        return None


def reset_proxy_stats(proxy_url):
    """Reset proxy token counters before experiment."""
    import urllib.request
    try:
        reset_url = proxy_url.rsplit("/v1", 1)[0] + "/stats/reset"
        req = urllib.request.Request(reset_url, data=b"", method="POST")
        with urllib.request.urlopen(req, timeout=5) as resp:
            return json.loads(resp.read())
    except Exception:
        return None


def compute_pput(golden_path_tokens, total_tokens, elapsed_minutes):
    """PPUT — Progress Per Unit Time.

    PPUT = golden_path_tokens / (total_tokens × elapsed_minutes)

    Measures: how efficiently did the swarm convert API tokens into
    useful proof progress? No artificial gates — data speaks for itself.
    golden_path_tokens = 0 means PPUT = 0 (no progress).
    """
    if golden_path_tokens <= 0 or total_tokens <= 0 or elapsed_minutes <= 0:
        return 0.0
    return round(golden_path_tokens / (total_tokens * elapsed_minutes), 6)


def main():
    # Ensure directories
    for d in [LOG_DIR, LOG_DIR / "success", LOG_DIR / "failure", TAPES_DIR, CONFIGS_DIR]:
        d.mkdir(parents=True, exist_ok=True)

    if not BINARY.exists():
        print(f"ERROR: Binary not found at {BINARY}", file=sys.stderr)
        print("Run: cd experiments/zeta_sum_proof && cargo build --release --bin evaluator",
              file=sys.stderr)
        sys.exit(1)

    cfg = load_config()
    base_env = load_env()
    run_id = next_run_id()

    # Save config snapshot
    shutil.copy2(CONFIG, CONFIGS_DIR / f"config_{run_id:03d}.json")

    ratio = f"{cfg['math_count']}M/{cfg['bull_count']}B+/{cfg['bear_count']}B-"
    provider = cfg.get("provider", "aliyun")
    model = cfg.get("model", "qwen3-8b")
    print(f"=== TuringOS AutoResearch — Run {run_id:03d} ===")
    print(f"Config: {cfg['swarm_size']} agents ({ratio}), {provider}/{model}")
    print(f"Description: {cfg.get('description', 'n/a')}")
    print(f"Running...", flush=True)

    # Initialize results.tsv if needed
    if not RESULTS.exists():
        with open(RESULTS, "w") as f:
            f.write("run_id\ttimestamp\tPPUT\tdepth\tnodes\tnovelty\troots\tappends\tdedup\t"
                    "bankrupt\tmax_frontier\tyes\tno\ttraded\tproved\telapsed_s\tstatus\t"
                    "description\tconfig_json\tgp_tokens\ttotal_tokens\n")

    # Reset proxy token counters before experiment
    proxy_url = cfg.get("proxy_url", "http://127.0.0.1:8088/v1/chat/completions")
    reset_proxy_stats(proxy_url)

    # Run
    output, elapsed, run_log_dir, tape_path, wal_path = run_evaluator(cfg, base_env, run_id)

    # Save log
    log_file = run_log_dir / "evaluator.log"
    log_file.write_text(output)

    # Parse & analyze — tape markdown first, WAL fallback
    nodes = parse_tape(tape_path)
    if not nodes and wal_path.exists():
        # WAL exists but tape markdown wasn't written (timeout killed before dump)
        # Extract node count from WAL for basic metrics
        try:
            wal_data = json.loads(wal_path.read_text())
            wal_nodes = wal_data.get("tape_files", {})
            if wal_nodes:
                print(f"[WAL FALLBACK] Tape markdown empty but WAL has {len(wal_nodes)} nodes", flush=True)
                # Build minimal nodes dict from WAL
                for nid, f in wal_nodes.items():
                    nodes[nid] = {
                        "author": f.get("author", ""),
                        "price": int(f.get("price", 0)),
                        "citations": f.get("citations", []),
                        "payload": f.get("payload", "")[:200],
                    }
        except (json.JSONDecodeError, KeyError):
            pass
    if not nodes:
        # Last resort: count from JSONL success log
        success_jsonl = run_log_dir / "success.jsonl"
        if success_jsonl.exists():
            lines = success_jsonl.read_text().strip().splitlines()
            print(f"[JSONL FALLBACK] {len(lines)} successful appends in JSONL", flush=True)

    m = analyze(nodes, output)

    # Query proxy for real token counts
    proxy_stats = query_proxy_stats(proxy_url)
    total_tokens = proxy_stats.get("total_tokens", 0) if proxy_stats else 0

    # Golden path tokens: count tokens in deepest chain payloads
    chain = extract_deepest_chain(nodes)
    gp_tokens = 0
    if chain and proxy_stats:
        # Estimate golden path tokens from payload chars (1 token ≈ 3 chars average)
        gp_chars = sum(len(nodes[nid].get("payload", "")) for nid in chain if nid in nodes)
        gp_tokens = gp_chars // 3
        # If total_tokens is 0 (proxy stats failed), estimate from nodes
        if total_tokens == 0:
            total_chars = sum(len(n.get("payload", "")) for n in nodes.values())
            total_tokens = total_chars // 3

    elapsed_minutes = elapsed / 60.0
    pput = compute_pput(gp_tokens, total_tokens, elapsed_minutes)

    m["gp_tokens"] = gp_tokens
    m["total_tokens"] = total_tokens

    # Classify success/failure
    is_success = m["proved"] or (m["depth"] >= 5 and m["novelty"] >= 0.4)
    outcome = "success" if is_success else "failure"
    outcome_dir = LOG_DIR / outcome
    outcome_dir.mkdir(parents=True, exist_ok=True)

    # Copy log + tape to outcome dir
    shutil.copy2(log_file, outcome_dir / f"run_{run_id:03d}.log")
    if tape_path.exists():
        shutil.copy2(tape_path, outcome_dir / f"tape_{run_id:03d}.md")

    # Write to results.tsv (ALWAYS — even on crash)
    ty = m["total_yes"]
    tn = max(m["buy_no"], 1)
    with open(RESULTS, "a") as f:
        row = [
            f"{run_id:03d}",
            datetime.now().isoformat(),
            f"{pput}",
            str(m["depth"]),
            str(m["nodes"]),
            str(m["novelty"]),
            str(m["roots"]),
            str(m["appends"]),
            str(m["dedup"]),
            str(m["bankrupt"]),
            str(m["max_frontier"]),
            str(ty),
            str(m["buy_no"]),
            str(m["traded"]),
            "YES" if m["proved"] else "NO",
            f"{elapsed:.0f}",
            outcome,
            cfg.get("description", ""),
            json.dumps(cfg),
            str(gp_tokens),
            str(total_tokens),
        ]
        f.write("\t".join(row) + "\n")

    # Print summary
    ratio_str = f"{ty/tn:.1f}:1"
    print("---")
    print(f"run_id:     {run_id:03d}")
    print(f"PPUT:       {pput}")
    print(f"depth:      {m['depth']}")
    print(f"gp_tokens:  {gp_tokens}")
    print(f"total_tokens: {total_tokens}")
    print(f"nodes:      {m['nodes']}")
    print(f"novelty:    {m['novelty']}")
    print(f"appends:    {m['appends']}")
    print(f"traded:     {m['traded']}")
    print(f"proved:     {'YES' if m['proved'] else 'NO'}")
    print(f"elapsed_s:  {elapsed:.0f}")
    print(f"status:     {outcome}")

    # Return metrics for sweep.py to consume
    return pput, m, outcome


if __name__ == "__main__":
    main()
