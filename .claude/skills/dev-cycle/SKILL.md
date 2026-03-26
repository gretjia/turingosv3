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

### 4.5. MIGRATION SCAN (mandatory when economic engine or SKILL interfaces change)
**CRITICAL: This step was added after the Run 6 100B-mint incident.**
When changes affect the economic engine (kernel pricing, wallet, bus settlement, reward signals), you MUST:
1. `grep -rn "YieldReward\|InvestOnly\|intrinsic_reward\|global_pool\|on_halt\|100_000_000" experiments/ src/` — find ALL usages of old economic patterns
2. `grep -rn "hayekian\|map_reduce\|bounty_escrow\|amm\|UniswapPool" experiments/ src/` — find ALL stale API references
3. For EACH experiment in `experiments/*/src/`:
   - Read the main SKILL file (membrane, evaluator)
   - Verify it is compatible with the new economic engine
   - Flag any hardcoded rewards, legacy settlement logic, or stale assumptions
4. Fix ALL incompatible experiment code BEFORE proceeding to audit
**Failure to do this step caused the Run 6 Polymarket zero-sum violation (MathStepMembrane 100B legacy mint).**

### 5. AUDIT CODE
- Invoke the `kernel-auditor` agent to audit the actual code changes
- Full audit: purity, minting rights, DAG integrity, wallet rules
- **MUST include experiment SKILL files** — not just src/. Specifically:
  - All `experiments/*/src/*.rs` files that implement TuringTool
  - All `experiments/*/src/bin/evaluator.rs` files
  - Any file found in Stage 4.5 migration scan
- The kernel-auditor prompt MUST list these files explicitly

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
