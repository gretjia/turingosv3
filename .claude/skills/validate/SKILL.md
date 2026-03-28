---
name: validate
description: Multi-layer validation — cargo check, cargo test, and kernel purity audit
user_invocable: true
---

# /validate — Multi-Layer Validation

Run all validation layers for TuringOS.

## Layer A: Compilation Check
```bash
cd /home/zephryj/projects/turingosv3 && cargo check
```
Report: PASS or FAIL with error details.

## Layer B: Test Suite
```bash
cd /home/zephryj/projects/turingosv3 && cargo test
```
Report: PASS or FAIL with failing test details.

## Layer C: Kernel Purity Audit
Invoke the `kernel-auditor` agent to perform a full microkernel integrity check:
- Zero domain knowledge in kernel.rs
- SKILL-only reward minting
- Append-Only DAG integrity
- Stake >= 1.0 enforcement
- Bible.md philosophy alignment
- **Expanded scope**: experiment SKILL files (`experiments/*/src/*.rs`), evaluator binaries, problem formalizations

## Layer D: External Audit (when triggered from dev-cycle Stage 8b)
If `/validate` is invoked as part of a dev-cycle, also run:
- Math/Lean correctness: route to `gemini -p`
- Code architecture: route to `codex exec`
- Present external findings VERBATIM to user

## Output

```
=== VALIDATION REPORT ===
[Layer A] cargo check:     PASS / FAIL
[Layer B] cargo test:      PASS / FAIL
[Layer C] Kernel Audit:    CLEAN / VIOLATIONS FOUND
[Layer D] External Audit:  CLEAN / VIOLATIONS FOUND / SKIPPED

=== OVERALL: PASS / FAIL ===
```

If any layer fails, provide specific remediation guidance. FAIL-CLOSED: if violations exist, block commit.
