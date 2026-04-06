#!/usr/bin/env python3
"""
TuringOS Scaling Law — Direct Experiment Executor

Scientific-grade experiment runner. Bypasses sweep.py entirely.
Each experiment: fixed config → fresh isolation → proxy metering → WAL analysis → PPUT → validation.

Usage:
  python3 run_scaling_experiment.py scaling_matrix.json          # run full matrix
  python3 run_scaling_experiment.py scaling_matrix.json --n 5    # run only N=5
  python3 run_scaling_experiment.py scaling_matrix.json --dry-run # show plan without executing
"""

import subprocess, json, os, sys, time, re, shutil, hashlib, tempfile
from pathlib import Path
from datetime import datetime
from collections import defaultdict
import urllib.request

# ── Paths ──
BASE = Path(__file__).resolve().parent
PROJECT = BASE.parent
BINARY = PROJECT / "target/release/evaluator"
RESULTS_FILE = BASE / "scaling_results.tsv"
PROMPT_DIR = PROJECT / "experiments/zeta_sum_proof/prompt"


def load_env():
    env = os.environ.copy()
    for line in (PROJECT / ".env").read_text().splitlines():
        line = line.strip()
        if "=" in line and not line.startswith("#"):
            k, v = line.split("=", 1)
            env[k.strip()] = v.strip()
    return env


def query_proxy_stats(proxy_url):
    try:
        stats_url = proxy_url.rsplit("/v1", 1)[0] + "/stats"
        with urllib.request.urlopen(stats_url, timeout=5) as resp:
            return json.loads(resp.read())
    except Exception as e:
        print("  WARNING: proxy stats query failed: {}".format(e))
        return None


def compute_file_hash(path):
    if not path.exists():
        return "MISSING"
    return hashlib.md5(path.read_bytes()).hexdigest()[:12]


def extract_deepest_chain(nodes):
    """Deterministic golden path: max root_dist leaf, tie-break by min node_id."""
    if not nodes:
        return []
    root_dist = {}
    def rd(nid):
        if nid in root_dist:
            return root_dist[nid]
        cites = [c for c in nodes[nid].get("citations", []) if c in nodes]
        if not cites:
            root_dist[nid] = 0
            return 0
        d = 1 + max(rd(c) for c in cites)
        root_dist[nid] = d
        return d
    for nid in nodes:
        rd(nid)
    if not root_dist:
        return []
    max_rd = max(root_dist.values())
    # Deterministic tie-break: lexicographically smallest node_id
    candidates = sorted([nid for nid, d in root_dist.items() if d == max_rd])
    leaf = candidates[0]
    chain = [leaf]
    while True:
        cites = [c for c in nodes[chain[-1]].get("citations", []) if c in nodes]
        if not cites:
            break
        chain.append(cites[0])
    return list(reversed(chain))


def validate_run(wal_data, config, stats_before, stats_after, elapsed):
    """Returns (valid, reasons). valid=False means this run should be excluded."""
    reasons = []

    # 1. Proxy errors
    if stats_after and stats_after.get("errors", 0) > (stats_before or {}).get("errors", 0):
        reasons.append("proxy_errors_increased")

    # 2. Token count
    total_tokens = 0
    if stats_before and stats_after:
        total_tokens = stats_after.get("total_tokens", 0) - stats_before.get("total_tokens", 0)
    if total_tokens <= 0:
        reasons.append("total_tokens_zero_or_negative")

    # 3. Estimated tokens
    est_before = (stats_before or {}).get("estimated_count", 0)
    est_after = (stats_after or {}).get("estimated_count", 0)
    if est_after > est_before:
        reasons.append("estimated_tokens_used_{}".format(est_after - est_before))

    # 4. Agent count in WAL
    nodes = wal_data.get("tape_files", {})
    actual_agents = set(n.get("author", "") for n in nodes.values())
    expected_n = config["swarm_size"]
    # Note: some agents may never append, so actual <= expected is OK
    # But actual > expected is a bug
    if len(actual_agents) > expected_n:
        reasons.append("more_agents_than_config_{}_vs_{}".format(len(actual_agents), expected_n))

    # 5. Config consistency
    total_roles = config["math_count"] + config["bull_count"] + config["bear_count"]
    if total_roles != config["swarm_size"]:
        reasons.append("role_sum_{}_ne_swarm_size_{}".format(total_roles, config["swarm_size"]))

    # 6. Golden path token verification
    chain = extract_deepest_chain(nodes)
    for nid in chain:
        node = nodes.get(nid, {})
        if node.get("completion_tokens", 0) == 0 and len(chain) > 1:
            reasons.append("golden_path_node_{}_has_zero_tokens".format(nid))
            break  # one is enough to flag

    valid = len(reasons) == 0
    return valid, reasons, total_tokens


def run_one_experiment(config, matrix, run_id, base_env):
    """Run a single experiment with full isolation and metering."""
    n = config["N"]
    proxy_url = matrix["proxy_url"]

    # Fresh temp directory
    run_dir = Path(tempfile.mkdtemp(prefix="scaling_N{}_run{}_".format(n, run_id)))
    wal_path = run_dir / "wal.json"
    tape_path = run_dir / "tape.md"
    log_dir = run_dir / "logs"
    skills_dir = run_dir / "skills"
    log_dir.mkdir()
    skills_dir.mkdir()

    # Frozen env whitelist
    env = base_env.copy()
    env["SWARM_SIZE"] = str(config["swarm_size"])
    env["MATH_COUNT"] = str(config["math_count"])
    env["BULL_COUNT"] = str(config["bull_count"])
    env["BEAR_COUNT"] = str(config["bear_count"])
    env["LLM_PROVIDER"] = matrix.get("provider", "proxy")
    env["LLM_MODEL"] = matrix.get("model", "qwen3-8b")
    env["LLM_URL"] = proxy_url
    env["WAL_PATH"] = str(wal_path)
    env["TAPE_OUTPUT"] = str(tape_path)
    env["LOG_DIR"] = str(log_dir)
    env["AGENT_SKILLS_DIR"] = str(skills_dir)
    env["RUST_LOG"] = "info"
    env["LIBRARIAN_INTERVAL"] = str(matrix.get("librarian_interval", 8))
    env["PROMPT_DIR"] = str(PROMPT_DIR)

    wall_clock = config["wall_clock"]

    print("  [N={}, run={}] Starting (swarm={}, wall_clock={}s)...".format(
        n, run_id, config["swarm_size"], wall_clock), flush=True)

    # Snapshot metadata
    git_sha = subprocess.run(["git", "rev-parse", "HEAD"], capture_output=True, text=True, cwd=str(PROJECT)).stdout.strip()[:12]
    binary_hash = compute_file_hash(BINARY)
    prompt_hash = compute_file_hash(PROMPT_DIR / "problem.txt")
    skill_hash = compute_file_hash(PROMPT_DIR / "skill.txt")

    metadata = {
        "run_id": run_id, "N": n, "config": config,
        "git_sha": git_sha, "binary_hash": binary_hash,
        "prompt_hash": prompt_hash, "skill_hash": skill_hash,
        "timestamp": datetime.now().isoformat(),
        "run_dir": str(run_dir),
    }

    # Query proxy stats BEFORE
    stats_before = query_proxy_stats(proxy_url)

    # Run evaluator
    stderr_log = run_dir / "evaluator_stderr.log"
    start_time = time.time()
    with open(stderr_log, "w") as log_f:
        try:
            proc = subprocess.run(
                [str(BINARY)], cwd=str(PROJECT),
                stdout=subprocess.DEVNULL, stderr=log_f,
                timeout=max(wall_clock + 120, 300),
                env=env,
            )
        except subprocess.TimeoutExpired:
            log_f.write("\n[TIMEOUT]\n")
    elapsed = time.time() - start_time
    elapsed_minutes = elapsed / 60.0

    # Query proxy stats AFTER
    stats_after = query_proxy_stats(proxy_url)

    # Load WAL
    wal_data = {}
    if wal_path.exists():
        try:
            wal_data = json.loads(wal_path.read_text())
        except json.JSONDecodeError:
            pass

    nodes = wal_data.get("tape_files", {})

    # Extract golden path
    chain = extract_deepest_chain(nodes)
    depth = len(chain) - 1 if chain else 0

    # Compute gp_tokens from real completion_tokens in WAL nodes
    gp_tokens = sum(nodes[nid].get("completion_tokens", 0) for nid in chain if nid in nodes)

    # Total tokens from proxy
    total_tokens = 0
    if stats_before and stats_after:
        total_tokens = stats_after.get("total_tokens", 0) - stats_before.get("total_tokens", 0)

    # PPUT
    pput = 0.0
    if gp_tokens > 0 and total_tokens > 0 and elapsed_minutes > 0:
        pput = gp_tokens / (total_tokens * elapsed_minutes)

    # Validate
    valid, reasons, _ = validate_run(wal_data, config, stats_before, stats_after, elapsed)

    # Save metadata
    metadata.update({
        "depth": depth, "nodes": len(nodes), "chain_length": len(chain),
        "gp_tokens": gp_tokens, "total_tokens": total_tokens,
        "elapsed_s": round(elapsed), "elapsed_min": round(elapsed_minutes, 2),
        "pput": round(pput, 10),
        "valid": valid, "invalid_reasons": reasons,
        "stats_before": stats_before, "stats_after": stats_after,
    })
    (run_dir / "metadata.json").write_text(json.dumps(metadata, indent=2))

    status = "VALID" if valid else "INVALID({})".format(",".join(reasons))
    print("  [N={}, run={}] depth={} gp_tok={} total_tok={} PPUT={:.8f} {} ({:.0f}s)".format(
        n, run_id, depth, gp_tokens, total_tokens, pput, status, elapsed), flush=True)

    # Append to results TSV
    if not RESULTS_FILE.exists():
        with open(RESULTS_FILE, "w") as f:
            f.write("N\trun\tdepth\tnodes\tgp_tokens\ttotal_tokens\telapsed_s\tpput\tvalid\tinvalid_reasons\trun_dir\ttimestamp\n")

    with open(RESULTS_FILE, "a") as f:
        row = [
            str(n), str(run_id), str(depth), str(len(nodes)),
            str(gp_tokens), str(total_tokens), str(round(elapsed)),
            str(round(pput, 10)), str(valid), ";".join(reasons) if reasons else "",
            str(run_dir), datetime.now().isoformat(),
        ]
        f.write("\t".join(row) + "\n")

    return pput, depth, valid


def main():
    if len(sys.argv) < 2:
        print("Usage: python3 run_scaling_experiment.py <matrix.json> [--n N] [--dry-run]")
        sys.exit(1)

    matrix_path = Path(sys.argv[1])
    matrix = json.loads(matrix_path.read_text())

    filter_n = None
    dry_run = False
    for i, arg in enumerate(sys.argv[2:], 2):
        if arg == "--n" and i + 1 < len(sys.argv):
            filter_n = int(sys.argv[i + 1])
        if arg == "--dry-run":
            dry_run = True

    configs = matrix["configs"]
    if filter_n is not None:
        configs = [c for c in configs if c["N"] == filter_n]

    runs_per_n = matrix.get("runs_per_n", 5)

    if not BINARY.exists():
        print("ERROR: {} not found. Build first.".format(BINARY))
        sys.exit(1)

    print("=" * 60)
    print("TuringOS Scaling Law — PPUT Experiment")
    print("  Model: {}".format(matrix.get("model", "?")))
    print("  N values: {}".format([c["N"] for c in configs]))
    print("  Runs per N: {}".format(runs_per_n))
    print("  Total experiments: {}".format(len(configs) * runs_per_n))
    print("  Serial execution (one at a time)")
    print("=" * 60)

    if dry_run:
        print("\n[DRY RUN] Would execute:")
        for c in configs:
            for r in range(1, runs_per_n + 1):
                print("  N={} run={} swarm={} wall_clock={}s".format(c["N"], r, c["swarm_size"], c["wall_clock"]))
        return

    base_env = load_env()

    for config in configs:
        n = config["N"]
        print("\n=== N={} ({} runs) ===".format(n, runs_per_n))
        for run_id in range(1, runs_per_n + 1):
            try:
                run_one_experiment(config, matrix, run_id, base_env)
            except Exception as e:
                print("  [N={}, run={}] ERROR: {}".format(n, run_id, e))
            # Brief cooldown between runs
            time.sleep(5)

    # Summary
    print("\n" + "=" * 60)
    print("EXPERIMENT COMPLETE")
    print("Results: {}".format(RESULTS_FILE))
    if RESULTS_FILE.exists():
        lines = RESULTS_FILE.read_text().strip().splitlines()[1:]
        valid_count = sum(1 for l in lines if "\tTrue\t" in l)
        invalid_count = sum(1 for l in lines if "\tFalse\t" in l)
        print("  Valid: {}  Invalid: {}  Total: {}".format(valid_count, invalid_count, len(lines)))
    print("=" * 60)


if __name__ == "__main__":
    main()
