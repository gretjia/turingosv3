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

### 3.5. SPRINT CONTRACT (Anti-Context-Anxiety)
Before coding, define a sprint scope:
- **Max 5 files** per sprint. If the plan touches more, decompose into sequential sprints.
- **Sprint boundary**: list the exact files and changes for this sprint. No scope creep.
- **Reflection checkpoint**: after each sprint, pause and verify:
  - Did I complete what I planned? (not less, not more)
  - Am I rushing or cutting corners due to context pressure?
  - Are there unresolved issues I'm sweeping under the rug?
- If the session is getting long (many tool calls, multiple compactions), explicitly signal: "Context is getting heavy — recommend committing current work and starting a fresh session for remaining items."

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

### 8. EXTERNAL AUDIT (MANDATORY)
**Generator ≠ Evaluator. This stage cannot be skipped.**
The same AI that wrote the code MUST NOT be the sole evaluator. External audit is mandatory.

#### 8a. Internal Self-Audit
- The generator (you) reviews your own changes — this is Stage 5, already done.

#### 8b. External Audit (MANDATORY)
- Route by domain:
  - **Math/Lean 4 correctness**: `gemini -p` (Gemini for mathematical reasoning audit)
  - **Code architecture/harness**: `codex exec` (Codex for code-level alignment audit)
- External auditor receives: the PLAN/SPEC + actual code changes + constitutional rules
- External findings are presented **VERBATIM** to the user — generator cannot summarize, downgrade, or dismiss them
- If external audit finds violations, return to Stage 6 (FIX CODE)

### 9. SUMMARY (FAIL-CLOSED)
- Output a concise summary of all changes made
- List files modified with brief descriptions
- Note any Layer 2 parameter changes
- **Violation check**: if ANY open violation exists (from Stage 5 or 8b), this stage BLOCKS. Cannot proceed to commit.
- **Wait for user confirmation before committing**
- Do NOT auto-commit or auto-push
- If violations were found and fixed, log the violation → fix chain for the handover
