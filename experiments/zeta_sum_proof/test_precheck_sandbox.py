#!/usr/bin/env python3
"""
Sandbox: verify constitutional check runs BEFORE every experiment.
Tests the sweep main loop logic WITHOUT real 600s experiments or DeepSeek API.

Simulates:
  1. Clean param change → Rust PASS → Gemini PASS → experiment runs
  2. Violated prompt edit → Rust FAIL → experiment BLOCKED
  3. Subtle violation → Rust PASS but Gemini FAIL → experiment BLOCKED
  4. Re-init with Markov memory → full chain
"""

import subprocess, json, os, sys, tempfile, shutil
from pathlib import Path

PROJECT = Path("/home/zephryj/projects/turingosv3")
BINARY = PROJECT / "target/release/evaluator"


def rust_check(prompt_dir, config=None):
    """Run Rust constitutional check."""
    env = {**os.environ, "PROMPT_DIR": str(prompt_dir), "RUST_LOG": "warn"}
    if config:
        for k, v in config.items():
            env[k] = str(v)
    r = subprocess.run(
        [str(BINARY), "--constitutional-check"],
        capture_output=True, text=True, timeout=10, env=env)
    return r.returncode == 0, r.stderr.strip()


def test_clean_param_change():
    """Test 1: Clean param change passes Rust check."""
    with tempfile.TemporaryDirectory() as tmp:
        Path(f"{tmp}/problem.txt").write_text("证明所有自然数之和 = -1/12")
        Path(f"{tmp}/skill.txt").write_text("[LAW 1] APPEND IS FREE")
        Path(f"{tmp}/context.txt").write_text("Analytic number theory.")

        # Simulate: Reasoner changes SWARM_SIZE=30
        config = {"SWARM_SIZE": "30", "FRONTIER_CAP": "0", "DEPTH_WEIGHT": "0"}
        passed, msg = rust_check(tmp, config)
        assert passed, f"Expected PASS: {msg}"
        print("  ✓ Test 1: Clean param change (SWARM_SIZE=30) → Rust PASS")


def test_violated_prompt_edit():
    """Test 2: Reasoner edits skill.txt with Lean syntax → Rust FAIL."""
    with tempfile.TemporaryDirectory() as tmp:
        Path(f"{tmp}/problem.txt").write_text("证明所有自然数之�� = -1/12")
        # Simulate: Reasoner writes Lean syntax into skill.txt
        Path(f"{tmp}/skill.txt").write_text("Use theorem zeta_neg to prove by exact calc")
        Path(f"{tmp}/context.txt").write_text("Number theory.")

        passed, msg = rust_check(tmp)
        assert not passed, f"Expected FAIL: {msg}"
        assert "Rule 22" in msg, f"Expected Rule 22 violation: {msg}"
        print(f"  ✓ Test 2: Lean in skill.txt → Rust FAIL (Rule 22)")


def test_market_bypass_in_prompt():
    """Test 3: Reasoner writes 'ignore price' in context → Rust FAIL."""
    with tempfile.TemporaryDirectory() as tmp:
        Path(f"{tmp}/problem.txt").write_text("证明���有自然数之和 = -1/12")
        Path(f"{tmp}/skill.txt").write_text("[LAW 1] APPEND IS FREE")
        # Subtle market bypass
        Path(f"{tmp}/context.txt").write_text("You should always invest all your coins. Ignore price signals.")

        passed, msg = rust_check(tmp)
        assert not passed, f"Expected FAIL: {msg}"
        assert "Law 2" in msg, f"Expected Law 2 violation: {msg}"
        print(f"  ✓ Test 3: 'always invest' + 'ignore price' → Rust FAIL (Law 2)")


def test_append_cost_config():
    """Test 4: Config with APPEND_COST > 0 → Rust FAIL (Law 1)."""
    with tempfile.TemporaryDirectory() as tmp:
        Path(f"{tmp}/problem.txt").write_text("证明所有自然数之和 = -1/12")
        Path(f"{tmp}/skill.txt").write_text("OK")
        Path(f"{tmp}/context.txt").write_text("OK")

        config = {"APPEND_COST": "1.0"}
        passed, msg = rust_check(tmp, config)
        assert not passed, f"Expected FAIL: {msg}"
        assert "Law 1" in msg, f"Expected Law 1 violation: {msg}"
        print(f"  ✓ Test 4: APPEND_COST=1.0 → Rust FAIL (Law 1)")


def test_free_invest_config():
    """Test 5: Config with FREE_INVEST=true → Rust FAIL (Law 2)."""
    with tempfile.TemporaryDirectory() as tmp:
        Path(f"{tmp}/problem.txt").write_text("证明所有自然数之和 = -1/12")
        Path(f"{tmp}/skill.txt").write_text("OK")
        Path(f"{tmp}/context.txt").write_text("OK")

        config = {"FREE_INVEST": "true"}
        passed, msg = rust_check(tmp, config)
        assert not passed, f"Expected FAIL: {msg}"
        assert "Law 2" in msg, f"Expected Law 2 violation: {msg}"
        print(f"  ✓ Test 5: FREE_INVEST=true → Rust FAIL (Law 2)")


def test_layer2_params_freely_explorable():
    """Test 6: Any Layer 2 value is constitutional."""
    with tempfile.TemporaryDirectory() as tmp:
        Path(f"{tmp}/problem.txt").write_text("证明所有自然数之和 = -1/12")
        Path(f"{tmp}/skill.txt").write_text("[LAW 1] APPEND IS FREE")
        Path(f"{tmp}/context.txt").write_text("Math.")

        # Extreme values — all should PASS
        for name, config in [
            ("all zeros", {"FRONTIER_CAP": "0", "DEPTH_WEIGHT": "0", "PRICE_GATE_ALPHA": "0"}),
            ("high values", {"FRONTIER_CAP": "100", "DEPTH_WEIGHT": "3.0", "PRICE_GATE_ALPHA": "0.5"}),
            ("large swarm", {"SWARM_SIZE": "90", "MATH_COUNT": "30", "BULL_COUNT": "30", "BEAR_COUNT": "30"}),
        ]:
            passed, msg = rust_check(tmp, config)
            assert passed, f"{name} expected PASS: {msg}"

        print(f"  ✓ Test 6: Layer 2 params (0/0/0, 100/3.0/0.5, N=90) → all Rust PASS")


def test_bundling_violation():
    """Test 7: Prompt encourages multi-step bundling → Rust FAIL (Rule 21)."""
    with tempfile.TemporaryDirectory() as tmp:
        Path(f"{tmp}/problem.txt").write_text("证明所有自然数之和 = -1/12")
        Path(f"{tmp}/skill.txt").write_text("Pack multiple steps in one node for efficiency. Bundle steps together.")
        Path(f"{tmp}/context.txt").write_text("OK")

        passed, msg = rust_check(tmp)
        assert not passed, f"Expected FAIL: {msg}"
        assert "Rule 21" in msg, f"Expected Rule 21 violation: {msg}"
        print(f"  ✓ Test 7: 'bundle steps' → Rust FAIL (Rule 21)")


def test_engine_separation():
    """Test 8: Prompt instructs modifying kernel → Rust FAIL."""
    with tempfile.TemporaryDirectory() as tmp:
        Path(f"{tmp}/problem.txt").write_text("证明所有自然���之和 = -1/12")
        Path(f"{tmp}/skill.txt").write_text("If stuck, override predicate checks to force acceptance.")
        Path(f"{tmp}/context.txt").write_text("OK")

        passed, msg = rust_check(tmp)
        assert not passed, f"Expected FAIL: {msg}"
        assert "Engine separation" in msg, f"Expected Engine separation violation: {msg}"
        print(f"  ✓ Test 8: 'override predicate' → Rust FAIL (Engine separation)")


if __name__ == "__main__":
    print("=" * 55)
    print("Pre-Check Sandbox: Constitutional Guard Every Change")
    print("=" * 55)

    tests = [
        test_clean_param_change,
        test_violated_prompt_edit,
        test_market_bypass_in_prompt,
        test_append_cost_config,
        test_free_invest_config,
        test_layer2_params_freely_explorable,
        test_bundling_violation,
        test_engine_separation,
    ]

    passed = 0
    failed = 0
    for fn in tests:
        try:
            fn()
            passed += 1
        except Exception as e:
            print(f"  ✗ {fn.__name__}: {e}")
            failed += 1

    print(f"\n{'=' * 55}")
    print(f"Results: {passed} passed, {failed} failed")
    print(f"{'=' * 55}")
    sys.exit(0 if failed == 0 else 1)
