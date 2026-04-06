#!/usr/bin/env python3
"""
Hourly Research Summary — analyzes all researchers' recent work and posts to bulletin.

Runs every hour via cron. Reads latest results.tsv and WALs, summarizes progress
toward the current research goal, and posts a concise update to the bulletin.

Usage:
  python3 hourly_summary.py
  cron: 0 * * * * python3 /path/to/hourly_summary.py >> /tmp/autoresearch_swarm/hourly.log 2>&1
"""

import json, os, fcntl, glob, re
from datetime import datetime, timedelta
from pathlib import Path
from collections import defaultdict

BASE = Path(__file__).resolve().parent
BULLETIN = BASE / "shared" / "bulletin.jsonl"
RESEARCHERS = {
    "alpha": ("zeta", "qwen3-8b/Aliyun"),
    "beta": ("zeta-b", "Qwen3-8B/SiliconFlow"),
    "gamma": ("zeta-c", "Qwen3-8B/SiliconFlow"),
    "delta": ("zeta-d", "Volcengine"),
}


def parse_results(results_path, since_hours=1):
    """Parse results.tsv, return recent entries."""
    if not results_path.exists():
        return []
    cutoff = datetime.now() - timedelta(hours=since_hours)
    entries = []
    for line in results_path.read_text().strip().splitlines()[1:]:
        parts = line.split("\t")
        if len(parts) < 18:
            continue
        try:
            ts = datetime.fromisoformat(parts[1])
            ers = float(parts[2])
            depth = int(parts[3])
            nodes = int(parts[4])
            novelty = float(parts[5])
            desc = parts[17] if len(parts) > 17 else ""
            entries.append({
                "ts": ts, "ers": ers, "depth": depth,
                "nodes": nodes, "novelty": novelty, "desc": desc,
            })
        except (ValueError, IndexError):
            continue
    # Return all entries (for total count) and recent ones
    recent = [e for e in entries if e["ts"] > cutoff]
    return entries, recent


def extract_swarm_size(desc):
    """Try to extract swarm_size from description."""
    m = re.search(r'swarm_size=(\d+)', desc)
    if m:
        return int(m.group(1))
    m = re.search(r'(\d+)M/(\d+)B\+/(\d+)B-', desc)
    if m:
        return int(m.group(1)) + int(m.group(2)) + int(m.group(3))
    return None


def summarize():
    ts = datetime.now().strftime("%Y-%m-%d %H:%M")
    print("{} === Hourly Summary ===".format(ts))

    summaries = []
    scaling_data = defaultdict(list)  # N -> [depths]

    for rid, (dirname, provider) in RESEARCHERS.items():
        rdir = BASE / dirname
        results_path = rdir / "results.tsv"

        if not results_path.exists():
            continue

        all_entries, recent = parse_results(results_path, since_hours=1)

        if not all_entries:
            continue

        total = len(all_entries)
        recent_count = len(recent)
        best = max(all_entries, key=lambda e: e["ers"])
        best_recent = max(recent, key=lambda e: e["ers"]) if recent else None

        # Collect scaling data from ALL entries
        for e in all_entries:
            n = extract_swarm_size(e["desc"])
            if n and e["depth"] >= 3:
                scaling_data[n].append(e["depth"])

        status = ""
        if recent_count == 0:
            status = "idle(0 exp/1h)"
        else:
            depths = [e["depth"] for e in recent if e["depth"] > 0]
            avg_d = sum(depths) / len(depths) if depths else 0
            status = "{}exp/1h, avg_depth={:.0f}".format(recent_count, avg_d)
            if best_recent and best_recent["ers"] > 0:
                status += ", best_pput={:.6f}(d={})".format(best_recent["ers"], best_recent["depth"])

        summaries.append("{}[{}exp]: {}".format(rid, total, status))

    # Build scaling law summary
    scaling_line = ""
    if scaling_data:
        points = []
        for n in sorted(scaling_data.keys()):
            depths = sorted(scaling_data[n])
            median = depths[len(depths) // 2]
            points.append("N={}→d={} ({}runs)".format(n, median, len(depths)))
        scaling_line = " | SCALING: " + ", ".join(points)

    # Compose bulletin message
    msg = "HOURLY SUMMARY [{}]: {} {}".format(
        ts[11:16],
        " | ".join(summaries),
        scaling_line
    )

    # Post to bulletin
    entry = {
        "ts": datetime.now().isoformat(timespec="seconds"),
        "from": "auditor",
        "type": "insight",
        "msg": msg[:500],
    }
    line = json.dumps(entry, ensure_ascii=False) + "\n"

    BULLETIN.parent.mkdir(parents=True, exist_ok=True)
    fd = os.open(str(BULLETIN), os.O_WRONLY | os.O_CREAT | os.O_APPEND, 0o644)
    try:
        fcntl.flock(fd, fcntl.LOCK_EX)
        os.write(fd, line.encode())
    finally:
        fcntl.flock(fd, fcntl.LOCK_UN)
        os.close(fd)

    print("  Posted: {}".format(msg[:200]))


if __name__ == "__main__":
    summarize()
