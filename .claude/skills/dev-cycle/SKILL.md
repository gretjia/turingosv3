---
name: dev-cycle
description: Full 9-stage development cycle for TuringOS — plan, audit, code, validate, summarize
user_invocable: true
---

# /dev-cycle — Full Development Cycle

Execute the complete development workflow in 9 stages.

## Stages

### 1. PLAN
- Read `handover/ai-direct/LATEST.md` for current state
- Read `handover/bible.md` for philosophy constraints
- Draft implementation plan with specific files and changes
- Present plan to user for approval

### 2. AUDIT PLAN
- Invoke the `kernel-auditor` agent to review the plan
- Check: Does the plan violate any Layer 1 invariants?
- Check: Does the plan align with bible.md philosophy?

### 3. FIX PLAN
- If audit found issues, revise the plan
- Re-present to user if significant changes were made

### 4. CODE
- Execute the approved code changes
- The PostToolUse hook will automatically run `cargo check` on critical files
- Make incremental changes, verifying each step

### 5. AUDIT CODE
- Invoke the `kernel-auditor` agent to audit the actual code changes
- Full audit: purity, minting rights, DAG integrity, wallet rules

### 6. FIX CODE
- If audit found violations, fix them immediately
- Re-run audit until CLEAN

### 7. VALIDATE
- Run `cargo check` (compilation)
- Run `cargo test` (unit + integration tests)
- Both must pass before proceeding

### 8. EXTERNAL AUDIT
- Skip for TuringOS (no external LLM audit requirement)
- Log: "External audit skipped — not applicable"

### 9. SUMMARY
- Output a concise summary of all changes made
- List files modified with brief descriptions
- Note any Layer 2 parameter changes
- **Wait for user confirmation before committing**
- Do NOT auto-commit or auto-push
