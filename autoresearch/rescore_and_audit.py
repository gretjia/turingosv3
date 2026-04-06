#!/usr/bin/env python3
"""
PPUT Re-scorer + Depth Auditor — runs every 4 hours via cron.

1. Re-scores recent WAL files with PPUT
2. Verifies depth is real (traces deepest chain, checks content cycling)
3. Posts leaderboard to bulletin
"""

import json, os, fcntl, re, glob
from datetime import datetime
from pathlib import Path
from collections import defaultdict

BASE = Path(__file__).resolve().parent
BULLETIN = BASE / "shared" / "bulletin.jsonl"
RESEARCHERS = {
    "alpha": BASE / "zeta",
    "beta": BASE / "zeta-b",
    "gamma": BASE / "zeta-c",
    "delta": BASE / "zeta-d",
}


def extract_deepest_chain(nodes):
    if not nodes: return []
    root_dist = {}
    def rd(nid):
        if nid in root_dist: return root_dist[nid]
        cites = [c for c in nodes[nid].get("citations",[]) if c in nodes]
        if not cites: root_dist[nid]=0; return 0
        d = 1+max(rd(c) for c in cites); root_dist[nid]=d; return d
    for nid in nodes: rd(nid)
    if not root_dist: return []
    leaf = max(root_dist, key=root_dist.get)
    chain = [leaf]
    while True:
        cites = [c for c in nodes[chain[-1]].get("citations",[]) if c in nodes]
        if not cites: break
        chain.append(cites[0])
    return list(reversed(chain))


def audit_depth(nodes, chain):
    if not chain or len(chain) < 3:
        return {"valid": True, "reason": "too_short"}
    payloads = [nodes[nid].get("payload","").strip()[:60].lower() for nid in chain]
    dupes = len(payloads) - len(set(payloads))
    stripped = []
    for p in payloads:
        clean = re.sub(r'^step\s*\d+\s*[\[:\(].*?[\]:\)]\s*:?\s*', '', p, flags=re.IGNORECASE)
        clean = re.sub(r'^step\s*\d+\s*:?\s*', '', clean, flags=re.IGNORECASE)
        stripped.append(clean[:40])
    near_dupes = len(stripped) - len(set(stripped))
    valid = (dupes == 0 and near_dupes <= len(chain) * 0.15)
    return {"valid": valid, "chain_len": len(chain), "exact_dupes": dupes, "near_dupes": near_dupes}


def rescore_all():
    ts = datetime.now().strftime("%Y-%m-%d %H:%M")
    print("{} === PPUT Re-scoring ===".format(ts))

    leaderboard = []

    for rid, rdir in RESEARCHERS.items():
        tapes_dir = rdir / "tapes"
        results_path = rdir / "results.tsv"
        if not tapes_dir.exists(): continue

        wals = sorted(tapes_dir.glob("wal_*.json"), key=lambda p: p.stat().st_mtime)
        recent = wals[-5:] if len(wals) > 5 else wals

        best_pput = 0
        best_detail = None

        for wal_path in recent:
            try:
                with open(wal_path) as f:
                    wal = json.load(f)
                nodes = wal.get("tape_files", {})
                if not nodes: continue

                chain = extract_deepest_chain(nodes)
                depth = len(chain) - 1 if chain else 0
                gp_chars = sum(len(nodes[nid].get("payload","")) for nid in chain if nid in nodes)
                total_chars = sum(len(n.get("payload","")) for n in nodes.values())

                # Estimate tokens (1 token ≈ 3 chars)
                gp_tokens = gp_chars // 3
                total_tokens = total_chars // 3

                # Read total_tokens from results.tsv if available (real counts)
                run_num = wal_path.stem.split("_")[1] if "_" in wal_path.stem else None
                if run_num and results_path.exists():
                    for line in results_path.read_text().strip().splitlines()[1:]:
                        parts = line.split("\t")
                        if parts[0].strip() == run_num and len(parts) > 20:
                            try:
                                real_total = int(parts[20])
                                if real_total > 0:
                                    total_tokens = real_total
                                real_gp = int(parts[19])
                                if real_gp > 0:
                                    gp_tokens = real_gp
                            except (ValueError, IndexError):
                                pass

                elapsed_minutes = 10.0  # default estimate
                audit = audit_depth(nodes, chain)

                if total_tokens > 0 and elapsed_minutes > 0:
                    pput = gp_tokens / (total_tokens * elapsed_minutes)
                else:
                    pput = 0.0

                detail = {
                    "depth": depth, "gp_tokens": gp_tokens, "total_tokens": total_tokens,
                    "pput": round(pput, 6), "audit": audit,
                }

                if pput > best_pput:
                    best_pput = pput
                    best_detail = detail
                    best_detail["run"] = run_num

            except Exception as e:
                pass

        if best_detail:
            leaderboard.append({"id": rid, **best_detail})

    leaderboard.sort(key=lambda x: x["pput"], reverse=True)

    lines = ["PPUT LEADERBOARD [{}]:".format(ts)]
    for i, e in enumerate(leaderboard):
        a = e.get("audit", {})
        v = "OK" if a.get("valid", True) else "INVALID"
        lines.append("  #{} {} run{}: PPUT={:.6f} depth={} [{}] gp_tok={} total_tok={}".format(
            i+1, e["id"], e.get("run","?"), e["pput"], e["depth"], v, e["gp_tokens"], e["total_tokens"]))

    msg = " | ".join(lines)[:500]

    entry = {"ts": datetime.now().isoformat(timespec="seconds"), "from": "auditor", "type": "insight", "msg": msg}
    line = json.dumps(entry, ensure_ascii=False) + "\n"
    BULLETIN.parent.mkdir(parents=True, exist_ok=True)
    fd = os.open(str(BULLETIN), os.O_WRONLY | os.O_CREAT | os.O_APPEND, 0o644)
    try:
        fcntl.flock(fd, fcntl.LOCK_EX)
        os.write(fd, line.encode())
    finally:
        fcntl.flock(fd, fcntl.LOCK_UN)
        os.close(fd)

    print("  Leaderboard posted ({} researchers)".format(len(leaderboard)))
    for e in leaderboard:
        print("  {} run{}: PPUT={:.6f} depth={}".format(e["id"], e.get("run","?"), e["pput"], e["depth"]))


if __name__ == "__main__":
    rescore_all()
