#!/usr/bin/env python3
"""
TuringOS AutoResearch — Librarian Agent
Compresses accumulated logs into memory. Run offline (e.g., overnight cron).

Memory = compressed logs. Not real-time. Accumulate first, compress later.

Usage:
  python3 librarian.py                    # compress today's logs
  python3 librarian.py --date 2026-04-02  # compress specific date

Outputs:
  memory/YYYY-MM-DD_success.md   — what worked
  memory/YYYY-MM-DD_failure.md   — what didn't (via negativa)
  memory/YYYY-MM-DD_patterns.md  — cross-cutting patterns
"""

import re, os, sys, glob
from pathlib import Path
from datetime import datetime, timedelta
from collections import Counter, defaultdict

BASE = Path(__file__).resolve().parent
LOG_DIR = BASE / "logs"
MEMORY_DIR = BASE / "memory"
RESULTS = BASE / "results.tsv"


def parse_log_summary(log_path):
    """Extract key metrics from a single run log"""
    content = Path(log_path).read_text()
    summary = {}

    summary["appends"] = len(re.findall(r"Append #", content))
    summary["dedup"] = len(re.findall(r"DEDUP", content))
    summary["buy_yes"] = len(re.findall(r"BUY YES", content))
    summary["buy_no"] = len(re.findall(r"BUY NO", content))
    summary["complete"] = len(re.findall(r"COMPLETE CLAIMED", content))
    summary["deepseek"] = len(re.findall(r"DEEPSEEK", content))
    summary["stagnation"] = len(re.findall(r"stagnation", content, re.I))

    # Extract common rejection reasons
    rejections = re.findall(r"REJECTED: ([^\|]+)", content)
    summary["rejection_reasons"] = Counter(
        r.strip()[:60] for r in rejections
    ).most_common(5)

    # Extract model and config
    m = re.search(r"Model: (\S+)", content)
    summary["model"] = m.group(1) if m else "unknown"
    m = re.search(r"(\d+)M/(\d+)B\+/(\d+)B-", content)
    summary["config"] = f"{m.group(1)}/{m.group(2)}/{m.group(3)}" if m else "unknown"

    return summary


def compress_logs(target_date=None):
    """Compress logs from a given date into memory"""
    if target_date is None:
        target_date = datetime.now().strftime("%Y-%m-%d")

    MEMORY_DIR.mkdir(parents=True, exist_ok=True)

    success_logs = sorted(glob.glob(str(LOG_DIR / "success" / "run_*.log")))
    failure_logs = sorted(glob.glob(str(LOG_DIR / "failure" / "run_*.log")))

    if not success_logs and not failure_logs:
        print("No logs to compress.")
        return

    print(f"Compressing {len(success_logs)} success + {len(failure_logs)} failure logs...")

    # ── Success Memory ──
    if success_logs:
        success_memory = f"# Success Memory — {target_date}\n\n"
        success_memory += f"**{len(success_logs)} successful runs**\n\n"

        for log_path in success_logs:
            s = parse_log_summary(log_path)
            ts = Path(log_path).stem.replace("run_", "")
            success_memory += f"## Run {ts}\n"
            success_memory += f"- Config: {s['config']} | Model: {s['model']}\n"
            success_memory += f"- Appends: {s['appends']} | Dedup: {s['dedup']}\n"
            success_memory += f"- YES: {s['buy_yes']} | NO: {s['buy_no']}\n"
            success_memory += f"- Complete claims: {s['complete']} | DeepSeek calls: {s['deepseek']}\n\n"

        out = MEMORY_DIR / f"{target_date}_success.md"
        out.write_text(success_memory)
        print(f"  → {out}")

    # ── Failure Memory (via negativa) ──
    if failure_logs:
        failure_memory = f"# Failure Memory (via negativa) — {target_date}\n\n"
        failure_memory += f"**{len(failure_logs)} failed runs**\n\n"

        # Aggregate rejection reasons across all failures
        all_rejections = Counter()
        all_configs = Counter()

        for log_path in failure_logs:
            s = parse_log_summary(log_path)
            all_configs[s["config"]] += 1
            for reason, count in s["rejection_reasons"]:
                all_rejections[reason] += count

        failure_memory += "## Common Rejection Reasons (across all failures)\n\n"
        for reason, count in all_rejections.most_common(10):
            failure_memory += f"- **{count}x**: {reason}\n"

        failure_memory += f"\n## Configs That Failed\n\n"
        for config, count in all_configs.most_common():
            failure_memory += f"- {config}: {count} failures\n"

        failure_memory += "\n## Per-Run Details\n\n"
        for log_path in failure_logs:
            s = parse_log_summary(log_path)
            ts = Path(log_path).stem.replace("run_", "")
            failure_memory += f"### Run {ts}\n"
            failure_memory += f"- Config: {s['config']} | Model: {s['model']}\n"
            failure_memory += f"- Appends: {s['appends']} | Stagnation: {s['stagnation']}\n"
            if s["rejection_reasons"]:
                failure_memory += f"- Top rejections: {', '.join(f'{r}({c})' for r,c in s['rejection_reasons'][:3])}\n"
            failure_memory += "\n"

        out = MEMORY_DIR / f"{target_date}_failure.md"
        out.write_text(failure_memory)
        print(f"  → {out}")

    # ── Cross-cutting Patterns ──
    patterns_memory = f"# Patterns — {target_date}\n\n"
    patterns_memory += f"Total: {len(success_logs)} success, {len(failure_logs)} failure\n\n"

    # Read results.tsv for ERS trends
    if RESULTS.exists():
        patterns_memory += "## ERS Ranking (from results.tsv)\n\n"
        patterns_memory += "```\n"
        for line in RESULTS.read_text().splitlines():
            patterns_memory += line + "\n"
        patterns_memory += "```\n\n"

    # Key insight extraction
    total_dedup = sum(parse_log_summary(l)["dedup"] for l in success_logs + failure_logs)
    total_appends = sum(parse_log_summary(l)["appends"] for l in success_logs + failure_logs)

    patterns_memory += "## Key Metrics (aggregated)\n\n"
    patterns_memory += f"- Total appends: {total_appends}\n"
    patterns_memory += f"- Total dedup rejections: {total_dedup}\n"
    patterns_memory += f"- Dedup rate: {total_dedup/(total_appends+total_dedup)*100:.0f}%\n" if (total_appends+total_dedup) > 0 else ""

    out = MEMORY_DIR / f"{target_date}_patterns.md"
    out.write_text(patterns_memory)
    print(f"  → {out}")

    print(f"\nLibrarian complete. Memory written to {MEMORY_DIR}/")


if __name__ == "__main__":
    date = None
    if len(sys.argv) > 2 and sys.argv[1] == "--date":
        date = sys.argv[2]
    compress_logs(date)
