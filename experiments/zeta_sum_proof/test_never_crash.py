#!/usr/bin/env python3
"""Sandbox: verify sweep NEVER crashes and resumes from TSV."""

import sys, json, tempfile, os
from pathlib import Path

# Import sweep functions
sys.path.insert(0, str(Path(__file__).parent))
os.chdir(str(Path(__file__).parent.parent.parent))

# Monkey-patch paths for test
import sweep_v4 as sweep
sweep.PROJECT = Path(tempfile.mkdtemp())
sweep.BINARY = Path("/home/zephryj/projects/turingosv3/target/release/evaluator")
sweep.PROMPT_DIR = sweep.PROJECT / "prompt"
sweep.RESULTS = sweep.PROJECT / "results.tsv"
sweep.LOG_DIR = sweep.PROJECT / "logs"
sweep.GT_LOG_DIR = sweep.PROJECT / "gt_logs"
sweep.PREV_LIFE_FILE = sweep.PROJECT / "prev_life.json"

# Create dirs
for d in [sweep.PROMPT_DIR, sweep.LOG_DIR, sweep.GT_LOG_DIR, sweep.RESULTS.parent]:
    d.mkdir(parents=True, exist_ok=True)
(sweep.PROMPT_DIR / "problem.txt").write_text("test")


def test_resume_empty():
    """Test 1: Resume from empty TSV → empty history."""
    h, ers, cfg, num = sweep.resume_from_tsv()
    assert h == [], f"Expected empty history, got {len(h)}"
    assert ers == -1.0
    assert num == 0
    print("  ✓ Test 1: Resume from empty → fresh start")


def test_resume_with_data():
    """Test 2: Resume from TSV with data → correct state."""
    header = "num\ttime\ters\tdepth\tappends\tdedup\tfrontier\tbankrupt\telapsed\tverdict\tchange\tconfig\n"
    row1 = '0\t2026-04-04\t0.005\t2\t30\t1\t5\t0\t600\tBASELINE\tbaseline\t{"SWARM_SIZE": 10}\n'
    row2 = '1\t2026-04-04\t0.01\t3\t40\t2\t8\t0\t600\tKEEP\tSWARM=20\t{"SWARM_SIZE": 20}\n'
    row3 = '2\t2026-04-04\t0.005\t1\t20\t0\t3\t0\t600\tDISCARD\tSWARM=30\t{"SWARM_SIZE": 20}\n'
    sweep.RESULTS.write_text(header + row1 + row2 + row3)

    h, ers, cfg, num = sweep.resume_from_tsv()
    assert len(h) == 3, f"Expected 3 entries, got {len(h)}"
    assert ers == 0.01, f"Expected best ERS=0.01, got {ers}"
    assert cfg["SWARM_SIZE"] == 20, f"Expected config from KEEP row"
    assert num == 2, f"Expected exp_num=2, got {num}"
    print("  ✓ Test 2: Resume from 3-row TSV → correct best_ers=0.01, config=SWARM20, exp=2")


def test_resume_with_error_row():
    """Test 3: TSV with ERROR row doesn't crash resume."""
    header = "num\ttime\ters\tdepth\tappends\tdedup\tfrontier\tbankrupt\telapsed\tverdict\tchange\tconfig\n"
    row1 = '0\t2026-04-04\t0.005\t2\t30\t1\t5\t0\t600\tBASELINE\tbaseline\t{"SWARM_SIZE": 10}\n'
    row_err = '1\t2026-04-04\t0\t0\t0\t0\t0\t0\t0\tERROR\tEXCEPTION:TypeError\t{"SWARM_SIZE": 10}\n'
    sweep.RESULTS.write_text(header + row1 + row_err)

    h, ers, cfg, num = sweep.resume_from_tsv()
    assert len(h) == 2
    assert h[1]["verdict"] == "ERROR"
    assert ers == 0.005  # best is still from row 0
    print("  ✓ Test 3: Resume with ERROR row → handled correctly")


def test_markov_memory_survives():
    """Test 4: Life memory persists across simulated restart."""
    memory = {"life_num": 1, "best_ers": 0.05, "what_worked": ["N=20"]}
    sweep.save_life_memory(memory)
    loaded = sweep.load_prev_life()
    assert loaded["life_num"] == 1
    assert loaded["best_ers"] == 0.05
    print("  ✓ Test 4: Markov memory save/load survives restart")


if __name__ == "__main__":
    print("=" * 50)
    print("Never-Crash Sandbox: Resume + Error Handling")
    print("=" * 50)

    passed = failed = 0
    for fn in [test_resume_empty, test_resume_with_data, test_resume_with_error_row, test_markov_memory_survives]:
        try:
            fn()
            passed += 1
        except Exception as e:
            print(f"  ✗ {fn.__name__}: {e}")
            failed += 1

    print(f"\n{'=' * 50}")
    print(f"Results: {passed} passed, {failed} failed")
    print(f"{'=' * 50}")
    sys.exit(0 if failed == 0 else 1)
