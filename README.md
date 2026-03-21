# TuringOS v3 — Silicon-Native Microkernel for LLM Formal Verification Swarm

## Quick Start: Boot a New Experiment

```bash
# 1. Write your Lean 4 theorem to a file
cat > /tmp/problem.lean << 'EOF'
import Mathlib

set_option maxHeartbeats 400000

theorem my_theorem : 1 + 1 = 2 := by
EOF

# 2. Run the boot script (creates project, compiles, deploys, launches swarm)
./scripts/boot-experiment.sh my_project my_theorem /tmp/problem.lean

# 3. Monitor
ssh zephrymac-studio "tail -f /tmp/my_project_run1.log"
```

The script automates steps 3-10 of experiment creation. Only the Lean 4 formalization (step 2) requires human/LLM intelligence.

---

## Architecture

### Core OS (`src/`)

| Module | Role |
|--------|------|
| `kernel.rs` | Sacred microkernel — pure topology + Hayekian O(V+E) map-reduce. **Zero domain knowledge.** |
| `bus.rs` | TSP Event Bus — SKILL lifecycle (on_boot → on_init → on_pre_append → on_post_append → on_halt) |
| `sdk/protocol.rs` | **Agent Output Protocol** — JSON `<action>{...}</action>` parser with legacy fallback |
| `sdk/prompt.rs` | **Minimal Prompt Template** — state-only, no rules explanation |
| `sdk/tools/wallet.rs` | WalletTool — PoS investment economy, free-float pricing |
| `sdk/tools/search.rs` | **Free SearchTool** — zero-cost Mathlib/library search (Magna Carta Law 1) |
| `sdk/sandbox.rs` | Isolated process sandbox — Lean 4 compilation with timeout + SIGKILL |
| `sdk/membrane.rs` | Legacy output parser (distill_pure_state) |
| `drivers/llm_http.rs` | Resilient HTTP client — multi-provider routing (SiliconFlow, Volcengine, DeepSeek) |

### Experiment Projects (`experiments/`)

Each experiment is an independent Cargo project that imports the Core SDK:

| Experiment | Status | Description |
|-----------|--------|-------------|
| `minif2f_swarm/` | Active | MiniF2F 244 Lean 4 theorem batch evaluator |
| `zeta_regularization/` | **OMEGA** | ζ(-1) = -1/12 — proved in Run 15 by heterogeneous swarm |
| `number_theory_min/` | Running | Smallest n: 7∣n, square, last digit 9, digit sum 25 |

Project-specific files only: `evaluator.rs` (entry + theorem), `lean4_membrane_tool.rs`, `harness.rs`, `swarm.rs`, `wal.rs`

### Alignment & Philosophy

| Document | Role |
|---------|------|
| `CLAUDE.md` | **Constitution** — Layer 1 invariants (immutable) + Layer 2 parameters (evolvable) |
| `handover/ALIGNMENT.md` | **Master Alignment** — all rules in precedence order |
| `handover/bible.md` | Philosophical foundation (read-only) |
| `handover/directives/` | Architect directive archive (append-only) |
| `skills/economic_operative.md` | Economic SKILL — 4 lines, extreme minimalism |

---

## The Magna Carta (Four Engines)

### Law 1: Information is Free
Agents can search Mathlib (`SearchTool`) at zero cost. Thinking costs nothing. Only writing to Tape costs money.

### Law 2: Only Investment Costs Money
`[Tool: Wallet | Action: Invest | Node: self | Amount: <FLOAT>]` — the single economic action. Compiler error = investment burned.

### Law 3: Digital Property Rights
Each agent has independent identity, balance, and (future) skill DNA.

### Engine Implementation

| Engine | Purpose | Status |
|--------|---------|--------|
| Epistemic Engine | Free search tools (Law 1) | ✅ `sdk/tools/search.rs` |
| Pure Capital Engine | Invest-only economy (Law 2) | ✅ `sdk/tools/wallet.rs` |
| Semantic Guillotine | 3-layer OMEGA detection | ✅ `lean4_membrane_tool.rs` |
| Speciation Engine | Per-agent DNA evolution (Law 3) | ⏸️ Deferred |

---

## Boot Script Reference

```
./scripts/boot-experiment.sh <project_name> <theorem_name> <lean_problem_file>
```

| Parameter | Description | Example |
|-----------|-------------|---------|
| `project_name` | Rust crate name `[a-z][a-z0-9_]*` | `number_theory_min` |
| `theorem_name` | Lean 4 theorem identifier | `find_smallest` |
| `lean_problem_file` | File containing Lean 4 theorem statement | `/tmp/problem.lean` |

**Environment variables** (must be set before running):
- `SILICONFLOW_API_KEY` — Primary SiliconFlow API key
- `SILICONFLOW_API_KEY_SECONDARY` — Secondary SF key (separate rate limits)
- `DEEPSEEK_API_KEY` — DeepSeek official API key

**Options**:
- `FORCE=1 ./scripts/boot-experiment.sh ...` — Overwrite existing project

**What it does** (steps 3-10, fully automated):
1. Creates `experiments/<project>/` directory structure
2. Copies shared modules (swarm, harness, WAL, membrane)
3. Generates `Cargo.toml` and `evaluator.rs` from template
4. Registers in workspace `Cargo.toml`
5. Runs `cargo check` (fails fast on bad Lean 4 syntax)
6. Syncs to Mac via rsync
7. Launches tmux session on Mac

---

## Key Results

### ζ(-1) = -1/12 (Run 15)
- **OMEGA achieved** in 51 minutes, Step 12
- Agent_2 (DeepSeek-R1) used `apply?` to delegate proof search to Lean 4
- 79 free Mathlib searches → `apply?` → all goals closed
- Dual independent audit: Codex (PROVISIONALLY VALID) + Gemini (PARTIALLY VALID)
- Full report: `experiments/zeta_regularization/audit/run15_OMEGA_analysis.md`

---

## Development Workflow

```
Human → problem description
  ↓
LLM → Lean 4 formalization (the ONLY step requiring intelligence)
  ↓
Human → confirms spec
  ↓
boot-experiment.sh → project created, compiled, deployed, launched (FULLY AUTOMATED)
  ↓
Swarm runs autonomously on Mac Studio
  ↓
Monitor: tail -f /tmp/<project>_run1.log
```

---

## Harness Structure (`.claude/`)

| Component | Files | Purpose |
|-----------|-------|---------|
| **Hooks** | `block-destructive.sh`, `post-edit-validate.sh`, `stop-guard.sh` | Automated safety guards |
| **Skills** | `/dev-cycle`, `/validate`, `/swarm-launch`, `/handover-update`, `/architect-ingest` | Workflow orchestration |
| **Agents** | `kernel-auditor` (Opus), `swarm-monitor` (Sonnet), `handover-writer` (Sonnet) | Specialized sub-agents |
