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
| `kernel.rs` | Sacred microkernel — pure topology + Hayekian O(V+E) Time-Arrow map-reduce. **Zero domain knowledge.** |
| `bus.rs` | TSP Event Bus — SKILL lifecycle (on_boot → on_init → on_pre_append → on_post_append → on_halt) + `fund_agent` (generation rebirth) |
| `sdk/actor.rs` | **Actor Runtime** — Lock-free concurrent agent model (watch + mpsc), Boltzmann softmax frontier routing (T=0.5) |
| `sdk/snapshot.rs` | **Immutable Universe Snapshot** — Lock-free read for all agents (Append-Only DAG guarantee) |
| `sdk/protocol.rs` | **Agent Output Protocol** — JSON `<action>{...}</action>` parser with legacy fallback |
| `sdk/prompt.rs` | **Minimal Prompt Template** — state-only, no rules explanation ("Gravity doesn't explain itself to apples") |
| `sdk/tool.rs` | **SKILL Interface** — TuringTool trait, AntiZombiePruning, OverwhelmingGapArbitrator |
| `sdk/tools/wallet.rs` | WalletTool — PoS investment economy, free-float pricing, Bankrupt vs Margin Call semantics |
| `sdk/tools/search.rs` | **Free SearchTool** — zero-cost Mathlib/library search (Magna Carta Law 1) |
| `sdk/sandbox.rs` | Isolated process sandbox — Lean 4 compilation with timeout + SIGKILL |
| `drivers/llm_http.rs` | Resilient HTTP client — multi-provider routing (SiliconFlow, DeepSeek) |

### Experiment Projects (`experiments/`)

Each experiment is an independent Cargo project that imports the Core SDK:

| Experiment | Status | Architecture | Description |
|-----------|--------|-------------|-------------|
| `zeta_sum_proof/` | **OMEGA x2** | Actor Model | ζ-sum regularization → -1/12. Run 3: 8 tx, 4-step proof, 5 min |
| `zeta_regularization/` | **OMEGA** | Batch Swarm | ζ(-1) = -1/12 — proved in Run 15 by heterogeneous swarm |
| `number_theory_min/` | **OMEGA** | Batch Swarm | Smallest n: 7∣n, square, last digit 9, digit sum 25 → n=5929 |
| `minif2f_swarm/` | Active | Batch Swarm | MiniF2F 244 Lean 4 theorem batch evaluator |
| `hanoi_1m/` | Legacy | Direct | 1M-token Tower of Hanoi (star topology validation) |

### Alignment & Philosophy

| Document | Role |
|---------|------|
| [`CLAUDE.md`](CLAUDE.md) | **Constitution** — Layer 1 invariants (immutable) + Layer 2 parameters (evolvable) |
| [`handover/ALIGNMENT.md`](handover/ALIGNMENT.md) | **Master Alignment** — all rules in precedence order |
| [`handover/bible.md`](handover/bible.md) | Philosophical foundation (read-only) |
| [`handover/directives/`](handover/directives/) | Architect directive archive (append-only) |
| [`handover/architect-insights/`](handover/architect-insights/) | Design insight archive (append-only) |
| [`handover/ai-direct/LATEST.md`](handover/ai-direct/LATEST.md) | Session handover state (single source of truth) |

---

## The Magna Carta (Four Engines)

### Law 1: Information is Free
Agents can search Mathlib (`SearchTool`) and view nodes (`ViewNode`) at zero cost. Thinking costs nothing. Only writing to Tape costs money.

### Law 2: Only Investment Costs Money
`[Tool: Wallet | Action: Invest | Node: self | Amount: <FLOAT>]` — the single economic action. Compiler error = investment burned.

### Law 3: Digital Property Rights
Each agent has independent identity, balance, and (future) skill DNA.

### Engine Implementation

| Engine | Purpose | Status |
|--------|---------|--------|
| Epistemic Engine | Free search/view tools (Law 1) | ✅ `sdk/tools/search.rs` |
| Pure Capital Engine | Invest-only economy (Law 2) | ✅ `sdk/tools/wallet.rs` |
| Semantic Guillotine | 3-layer OMEGA detection | ✅ `lean4_membrane_tool.rs` (configurable forbidden_tactics) |
| Speciation Engine | Per-agent DNA evolution (Law 3) | ⏸️ Deferred |

---

## Actor Model (Current Architecture)

The latest experiments use a lock-free Actor Model instead of batch-synchronous swarm:

```
┌─────────────────────────────────────────────────┐
│              Event Reactor (serial)              │
│  rx_mempool.recv() → append → MapReduce → snap  │
└──────────────────────┬──────────────────────────┘
                       │ watch::channel (broadcast)
         ┌─────────────┼─────────────┐
         ▼             ▼             ▼
    ┌─────────┐  ┌─────────┐  ┌─────────┐
    │ Agent 0 │  │ Agent 1 │  │ Agent N │
    │ V3.2    │  │Reasoner │  │  R1     │
    │ (fast)  │  │ (slow)  │  │ (deep)  │
    └─────────┘  └─────────┘  └─────────┘
    Boltzmann softmax routing (T=0.5)
    30s timeout → generation rebirth
    Superfluid clearing (MapReduce every append)
```

**Key mechanisms**:
- **Boltzmann Routing**: Probabilistic frontier selection, breaks star topology
- **Generation Rebirth**: 30s timeout detects market collapse → fresh capital injection
- **Superfluid Clearing**: No arbitrator threshold — MapReduce runs every append
- **Kelly Criterion**: SKILL prompt enforces risk management (no all-in)
- **Configurable Membrane**: `forbidden_tactics` per problem domain

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
- `DEEPSEEK_API_KEY` — DeepSeek official API key (shared by deepseek-chat + deepseek-reasoner)

**Options**:
- `FORCE=1 ./scripts/boot-experiment.sh ...` — Overwrite existing project

**WAL preservation**: The boot script preserves WAL files across runs for cross-epoch knowledge inheritance.

---

## Key Results

### ζ-Sum Regularization → -1/12 (zeta_sum_proof, Run 3)
- **OMEGA achieved** in 8 transactions, 4-step proof chain, ~5 minutes
- Actor Model: 15 agents, 3 species (DeepSeek V3.2 + Reasoner + R1)
- Zero empty steps (20-char membrane filter effective)
- Steps 1-3 verified mathematically correct by independent audit
- Full report: `experiments/zeta_sum_proof/audit/run1_analysis.md`

### ζ(-1) = -1/12 (zeta_regularization, Run 15)
- **OMEGA achieved** in 51 minutes, Step 12
- Agent_2 (DeepSeek-R1) used `apply?` to delegate proof search to Lean 4
- 79 free Mathlib searches → `apply?` → all goals closed
- Full report: `experiments/zeta_regularization/audit/run15_OMEGA_analysis.md`

### n = 5929 (number_theory_min)
- **OMEGA achieved** — Agent used `decide` tactic for exhaustive search
- Neuro-symbolic emergence: LLM derived algebraic bound k < 77, then delegated brute-force to Lean 4 ALU

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
Swarm runs autonomously on Mac Studio (Actor Model or Batch Swarm)
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
