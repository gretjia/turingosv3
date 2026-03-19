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

## Output

```
=== VALIDATION REPORT ===
[Layer A] cargo check:     PASS / FAIL
[Layer B] cargo test:      PASS / FAIL
[Layer C] Kernel Audit:    CLEAN / VIOLATIONS FOUND

=== OVERALL: PASS / FAIL ===
```

If any layer fails, provide specific remediation guidance.
