#!/usr/bin/env python3
"""
TuringOS AutoResearch Sweep v2
Inspired by Karpathy's autoresearch: build once, run many, NEVER STOP.

METRIC: Effective Reasoning Score (ERS)
  = depth × novelty × breadth_factor

- depth: max chain depth (deeper = better proof search)
- novelty: fraction of nodes with unique content (penalizes repetition)
- breadth_factor: min(roots, 5)/5 (multiple proof strategies)

The goal: find the config where the Polymarket economy
CONSTRAINS agents to produce deep, novel, multi-strategy reasoning.
"""

import subprocess, re, time, os, math, sys
from pathlib import Path
from collections import defaultdict

PROJECT = Path("/home/zephryj/projects/turingosv3")
BINARY = PROJECT / "target/release/evaluator"
TAPE = Path("/tmp/zeta_sum_tape_full.md")
RESULTS = PROJECT / "experiments/zeta_sum_proof/audit/sweep_v2_results.tsv"
LOG_DIR = PROJECT / "experiments/zeta_sum_proof/logs/sweep_v2"

# All experiments: 15 agents, 300 tx (~2 min each, no rate limiting)
# Focus: ratio sweep across diverse configurations
EXPERIMENTS = [
    # (label,            agents, math, bull, bear, max_tx)
    # --- Baseline ---
    ("equal_5_5_5",      15,     5,    5,    5,    300),
    # --- Math-heavy ---
    ("math_10_3_2",      15,     10,   3,    2,    300),
    ("math_8_4_3",       15,     8,    4,    3,    300),
    # --- Bull-heavy ---
    ("bull_5_8_2",       15,     5,    8,    2,    300),
    ("bull_3_10_2",      15,     3,    10,   2,    300),
    # --- Bear-heavy ---
    ("bear_5_2_8",       15,     5,    2,    8,    300),
    ("bear_3_2_10",      15,     3,    2,    10,   300),
    # --- Math + Bear (builders + critics) ---
    ("mathbear_7_1_7",   15,     7,    1,    7,    300),
    ("mathbear_8_2_5",   15,     8,    2,    5,    300),
    # --- Math + Bull (builders + funders) ---
    ("mathbull_7_7_1",   15,     7,    7,    1,    300),
    ("mathbull_8_5_2",   15,     8,    5,    2,    300),
    # --- Extreme ---
    ("all_math_15_0_0",  15,     15,   0,    0,    300),
    ("no_math_0_8_7",    15,     0,    8,    7,    300),
]


def load_env():
    env = os.environ.copy()
    env_file = PROJECT / ".env"
    for line in env_file.read_text().splitlines():
        line = line.strip()
        if '=' in line and not line.startswith('#'):
            k, v = line.split('=', 1)
            env[k.strip()] = v.strip()
    return env


def run_experiment(label, agents, math, bull, bear, max_tx, base_env):
    """Run evaluator, return (log_content, elapsed)"""
    # Clear stale tape
    if TAPE.exists():
        TAPE.unlink()

    env = base_env.copy()
    env["RUST_LOG"] = "info"
    env["SWARM_SIZE"] = str(agents)
    env["MAX_TX"] = str(max_tx)
    env["MATH_COUNT"] = str(math)
    env["BULL_COUNT"] = str(bull)
    env["BEAR_COUNT"] = str(bear)

    start = time.time()
    try:
        result = subprocess.run(
            [str(BINARY)], cwd=str(PROJECT),
            capture_output=True, text=True, timeout=600, env=env
        )
        output = result.stderr
    except subprocess.TimeoutExpired as e:
        output = (e.stderr or b"").decode(errors="replace")
        output += "\n[TIMEOUT]"

    elapsed = time.time() - start
    log_file = LOG_DIR / f"{label}_{int(time.time())}.log"
    log_file.write_text(output)
    return output, elapsed


def parse_tape():
    """Parse tape dump → nodes dict {id: {author, price, citations, payload}}"""
    if not TAPE.exists():
        return {}

    nodes = {}
    current_id = None
    for line in TAPE.read_text().splitlines():
        m = re.match(r"### `(\S+)` \| Author: (\S+) \| Price: (\S+) \| Citations: \[([^\]]*)\]", line)
        if m:
            current_id = m.group(1)
            cites = [c.strip().strip('"') for c in m.group(4).split(',') if c.strip().strip('"')]
            nodes[current_id] = {
                'author': m.group(2), 'price': int(float(m.group(3))),
                'citations': cites, 'payload': ''
            }
        elif current_id and line.startswith('```') and nodes.get(current_id, {}).get('payload'):
            current_id = None
        elif current_id and not line.startswith('```') and current_id in nodes:
            nodes[current_id]['payload'] += line.strip() + ' '
    return nodes


def analyze_tape(nodes):
    """Compute all metrics from parsed tape"""
    if not nodes:
        return {'nodes': 0, 'depth': 0, 'roots': 0, 'novelty': 0,
                'branching': 0, 'unique_prefixes': 0}

    # Build tree
    children = defaultdict(list)
    roots = []
    for nid, n in nodes.items():
        if not n['citations']:
            roots.append(nid)
        else:
            for p in n['citations']:
                if p in nodes:
                    children[p].append(nid)

    # Max depth
    memo = {}
    def depth(nid):
        if nid in memo: return memo[nid]
        kids = children.get(nid, [])
        d = (1 + max(depth(k) for k in kids)) if kids else 0
        memo[nid] = d
        return d
    max_d = max((depth(r) for r in roots), default=0)

    # Novelty: unique 40-char prefixes / total nodes
    prefixes = set()
    for n in nodes.values():
        prefix = n['payload'].strip()[:40].lower()
        prefixes.add(prefix)
    novelty = len(prefixes) / max(len(nodes), 1)

    # Branching factor (avg children per non-leaf)
    non_leaf = [nid for nid in nodes if children.get(nid)]
    avg_branch = sum(len(children[nid]) for nid in non_leaf) / max(len(non_leaf), 1)

    # Capital depth profile: what % of traded nodes are at depth > median?
    node_depths = {}
    def compute_node_depth(nid, d=0):
        node_depths[nid] = d
        for kid in children.get(nid, []):
            compute_node_depth(kid, d + 1)
    for r in roots:
        compute_node_depth(r)

    return {
        'nodes': len(nodes),
        'depth': max_d,
        'roots': len(roots),
        'novelty': round(novelty, 3),
        'branching': round(avg_branch, 2),
        'unique_prefixes': len(prefixes),
    }


def parse_log_metrics(log):
    """Extract trading metrics from log"""
    m = {}
    match = re.search(r"EVALUATION COMPLETE \((\d+) tx, (\d+) gen", log)
    m['tx'] = int(match.group(1)) if match else 0
    m['generations'] = int(match.group(2)) if match else 0
    m['buy_yes'] = len(re.findall(r"BUY YES\]", log))
    m['auto_long'] = len(re.findall(r"AUTO-LONG\]", log))
    m['buy_no'] = len(re.findall(r"BUY NO\]", log))
    m['total_yes'] = m['buy_yes'] + m['auto_long']
    m['proved'] = bool(re.search(r"Proof chain COMPLETE", log))
    m['rejected'] = len(re.findall(r"REJECTED", log))

    traded = set()
    for match in re.finditer(r"BUY (?:YES|NO)\] \S+ bought .+? on (\S+) for", log):
        traded.add(match.group(1))
    for match in re.finditer(r"AUTO-LONG\] \S+ bought .+? on (\S+) for", log):
        traded.add(match.group(1))
    m['traded'] = len(traded)
    return m


def compute_ers(tape_metrics, log_metrics):
    """
    Effective Reasoning Score (ERS)

    ERS = depth × novelty × breadth_factor × (1 + 0.5 × proved)

    - depth: normalized to [0,1] by cap at 15
    - novelty: fraction of unique content [0,1]
    - breadth_factor: min(roots, 5) / 5
    - proved: 50% bonus if OMEGA reached
    """
    depth = min(tape_metrics['depth'], 15) / 15.0
    novelty = tape_metrics['novelty']
    breadth = min(tape_metrics['roots'], 5) / 5.0
    proved_bonus = 1.5 if log_metrics['proved'] else 1.0

    ers = depth * novelty * breadth * proved_bonus
    return round(ers, 4)


def main():
    LOG_DIR.mkdir(parents=True, exist_ok=True)
    base_env = load_env()

    if not BINARY.exists():
        print(f"ERROR: {BINARY} not found", file=sys.stderr)
        sys.exit(1)

    # TSV header
    header = [
        "label", "agents", "math", "bull", "bear", "tx_budget",
        "nodes", "depth", "roots", "novelty", "unique_pfx",
        "yes", "no", "ratio", "traded", "proved",
        "ERS", "elapsed_s"
    ]
    with open(RESULTS, "w") as f:
        f.write("\t".join(header) + "\n")

    N = len(EXPERIMENTS)
    print(f"{'='*65}")
    print(f"TuringOS AutoResearch Sweep v2 — {N} experiments")
    print(f"Metric: ERS = depth × novelty × breadth × proved_bonus")
    print(f"All: 15 agents, 300 tx, Pro/Qwen2.5-7B-Instruct")
    print(f"{'='*65}", flush=True)

    results = []
    best_ers = 0
    best_label = ""

    for idx, (label, agents, math, bull, bear, max_tx) in enumerate(EXPERIMENTS):
        ratio_str = f"{math}M/{bull}B+/{bear}B-"
        print(f"\n[{idx+1}/{N}] {label} ({ratio_str})", flush=True)

        log, elapsed = run_experiment(label, agents, math, bull, bear, max_tx, base_env)
        print(f"  Time: {elapsed:.0f}s", flush=True)

        nodes = parse_tape()
        tape_m = analyze_tape(nodes)
        log_m = parse_log_metrics(log)
        ers = compute_ers(tape_m, log_m)

        ty = log_m['total_yes']
        tn = max(log_m['buy_no'], 1)
        yn_ratio = f"{ty/tn:.1f}:1"
        proved = "YES" if log_m['proved'] else "NO"

        print(f"  Nodes={tape_m['nodes']} Depth={tape_m['depth']} Roots={tape_m['roots']} "
              f"Novelty={tape_m['novelty']:.2f} ({tape_m['unique_prefixes']} unique)", flush=True)
        print(f"  YES:NO={yn_ratio} Traded={log_m['traded']}/{tape_m['nodes']} Proved={proved}", flush=True)
        print(f"  ERS={ers}", flush=True)

        row = [
            label, str(agents), str(math), str(bull), str(bear), str(max_tx),
            str(tape_m['nodes']), str(tape_m['depth']), str(tape_m['roots']),
            str(tape_m['novelty']), str(tape_m['unique_prefixes']),
            str(ty), str(log_m['buy_no']), yn_ratio, str(log_m['traded']),
            proved, str(ers), f"{elapsed:.0f}"
        ]
        with open(RESULTS, "a") as f:
            f.write("\t".join(row) + "\n")

        results.append((label, ers, tape_m, log_m, ratio_str))
        if ers > best_ers:
            best_ers = ers
            best_label = label
            print(f"  ★ NEW BEST!", flush=True)

    # ══════════════════════════════════════════════════════
    # PHASE 2: Best config at 2× and 4× budget
    # ══════════════════════════════════════════════════════
    print(f"\n{'='*65}")
    print(f"Phase 1 Winner: {best_label} (ERS={best_ers})")
    print(f"Phase 2: Confirmation at 2× and 4× budget")
    print(f"{'='*65}", flush=True)

    best_cfg = None
    for (label, agents, math, bull, bear, max_tx) in EXPERIMENTS:
        if label == best_label:
            best_cfg = (agents, math, bull, bear, max_tx)
            break

    if best_cfg:
        for mult, mult_label in [(2, "2x"), (4, "4x")]:
            agents, math, bull, bear, base_tx = best_cfg
            confirm_tx = base_tx * mult
            clabel = f"CONFIRM_{best_label}_{mult_label}"
            ratio_str = f"{math}M/{bull}B+/{bear}B-"
            print(f"\n[CONFIRM] {clabel} ({ratio_str}, {confirm_tx}tx)", flush=True)

            log, elapsed = run_experiment(clabel, agents, math, bull, bear, confirm_tx, base_env)
            print(f"  Time: {elapsed:.0f}s", flush=True)

            nodes = parse_tape()
            tape_m = analyze_tape(nodes)
            log_m = parse_log_metrics(log)
            ers = compute_ers(tape_m, log_m)

            ty = log_m['total_yes']
            tn = max(log_m['buy_no'], 1)
            yn_ratio = f"{ty/tn:.1f}:1"
            proved = "YES" if log_m['proved'] else "NO"

            print(f"  Nodes={tape_m['nodes']} Depth={tape_m['depth']} Roots={tape_m['roots']} "
                  f"Novelty={tape_m['novelty']:.2f}", flush=True)
            print(f"  YES:NO={yn_ratio} Traded={log_m['traded']}/{tape_m['nodes']} Proved={proved}", flush=True)
            print(f"  ERS={ers}", flush=True)

            row = [
                clabel, str(agents), str(math), str(bull), str(bear), str(confirm_tx),
                str(tape_m['nodes']), str(tape_m['depth']), str(tape_m['roots']),
                str(tape_m['novelty']), str(tape_m['unique_prefixes']),
                str(ty), str(log_m['buy_no']), yn_ratio, str(log_m['traded']),
                proved, str(ers), f"{elapsed:.0f}"
            ]
            with open(RESULTS, "a") as f:
                f.write("\t".join(row) + "\n")

    # ══════════════════════════════════════════════════════
    # FINAL RANKING
    # ══════════════════════════════════════════════════════
    print(f"\n{'='*65}")
    print(f"SWEEP COMPLETE — {RESULTS}")
    print(f"{'='*65}")

    results.sort(key=lambda x: x[1], reverse=True)
    print(f"\nRanking by ERS (depth × novelty × breadth):")
    print(f"{'Label':<22s} {'ERS':>6s} {'Depth':>5s} {'Nodes':>5s} {'Novel':>6s} "
          f"{'Roots':>5s} {'YES:NO':>7s} {'Proved':>6s}")
    print("-" * 70)
    for i, (label, ers, tm, lm, ratio) in enumerate(results):
        ty = lm['total_yes']
        tn = max(lm['buy_no'], 1)
        p = "✓" if lm['proved'] else "✗"
        print(f"#{i+1:2d} {label:<20s} {ers:>6.4f} {tm['depth']:>5d} {tm['nodes']:>5d} "
              f"{tm['novelty']:>6.2f} {tm['roots']:>5d} {ty/tn:>5.1f}:1 {p:>6s}")

    print(f"\n★ OPTIMAL: {best_label} (ERS={best_ers})")
    print(f"  See confirmation runs at 2× and 4× budget in {RESULTS}")


if __name__ == "__main__":
    main()
