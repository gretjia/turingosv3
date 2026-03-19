# TuringOS v3 (Silicon-Native Microkernel)

Welcome to TuringOS v3, a framework for Large Language Models executing formal verification, symbolic reasoning, and deep swarm speculation without system prompt hacks.

## 1. System Architecture

TuringOS v3 is not a prompt-chaining script. It is an operating system built on a **Star-Topology Microkernel Architecture** specifically designed to emulate thermodynamic concepts like Brownian motion, phase transitions, and Hayekian value decay over a computational tape.

*   **The Kernel (`src/kernel.rs`)**: Absolutely mathematically pure. It manages the `Tape` (the global log of all thoughts and actions) and computes $O(V+E)$ `hayekian_map_reduce`. It has zero knowledge of the business domain.
*   **The TSP Bus (`src/bus.rs`)**: The Turing Skill Protocol bus. It passes data between the AI Blackbox and the Kernel, allowing external hooks (Skills) to modify or veto data in real-time.
*   **The Skills (`src/sdk/skill.rs`)**: The specific "laws of physics" applied to the current simulation (e.g., `Lean4MembraneSkill` for Lean 4 formal verification, `AntiZombiePruningSkill` for deadlock prevention).

## 2. Recent Updates (March 2026)

The framework has fundamentally evolved during the **MiniF2F SOTA Run**:
1.  **Inversion of Control (Pricing)**: The kernel was stripped of all string-matching logic. All value minting (`intrinsic_reward = 100_000_000_000.0`) is now exclusively yielded by the external formal verification Membrane.
2.  **Boltzmann Softmax Router**: The swarm routing engine upgraded from `ArgMax` (Greedy) to `Softmax` (Probabilistic), naturally resolving "Roadroller deadlocks" by allowing the swarm compute cloud to spontaneously backtrack to earlier, purer nodes when blocked.
3.  **Turing Capitalism**: Transitioned to a Proof-of-Stake logic where Agents must stake computational funds to generate code. Added an **Austrian Economics patch** with floating stakes, market tickers, and a live VC mechanism to establish true price discovery without deflationary deadlocks.
4.  **Graveyard Protocol**: Failed compiler attempts are etched into public "tombstones", forcing subsequent agents to learn via In-Context Reflection.
5.  **Volcengine Migration**: The computational payload has migrated from SiliconFlow to **Volcengine Ark (Doubao-2.0-Pro)** to sustain extreme $N=30+$ concurrency without hardware truncation.
6.  **Claude Code 三层治理 Harness** (March 19): Migrated from Gemini CLI Agent to Claude Code with a full three-layer governance system:
    *   **Hooks**: Automated pre/post tool-use guards (`block-destructive.sh`, `post-edit-validate.sh`, `stop-guard.sh`)
    *   **Skills**: Workflow orchestration (`/dev-cycle`, `/validate`, `/swarm-launch`, `/handover-update`, `/architect-ingest`)
    *   **Agents**: Specialized sub-agents (`kernel-auditor`, `swarm-monitor`, `handover-writer`)
    *   See [`CLAUDE.md`](CLAUDE.md) for the project constitution and [`VIA_NEGATIVA.md`](VIA_NEGATIVA.md) for proven-false paths.

## 3. Directory Structure & Documentation

*   [`CLAUDE.md`](CLAUDE.md) - **Project Constitution** — Layer 1/2 invariants, engineering rules, key file map.
*   [`VIA_NEGATIVA.md`](VIA_NEGATIVA.md) - Proven-false paths (do not repeat).
*   [`src/`](src/) - The core TuringOS microkernel codebase.
*   [`experiments/`](experiments/) - Temporary test projects separated from the core kernel.
    *   [`experiments/minif2f_swarm/`](experiments/minif2f_swarm/) - The current SOTA execution harness for 244 Lean 4 theorems.
*   [`handover/`](handover/) - **[START HERE FOR AI AGENTS]** Documentation, architectural rules, audit reports, and AI state handover files.
    *   [`handover/README.md`](handover/README.md) - The core philosophy, rules, and directory index of the handover assets.
    *   [`handover/ai-direct/LATEST.md`](handover/ai-direct/LATEST.md) - The absolute single source of truth for the current execution state of the workspace.
*   [`.claude/`](.claude/) - Claude Code governance harness:
    *   [`.claude/settings.json`](.claude/settings.json) - Hook registrations (PreToolUse, PostToolUse, Stop).
    *   [`.claude/hooks/`](.claude/hooks/) - Automated guard scripts.
    *   [`.claude/agents/`](.claude/agents/) - Specialized sub-agent definitions.
    *   [`.claude/skills/`](.claude/skills/) - Workflow skill definitions.
*   [`.claudeignore`](.claudeignore) - Context isolation rules (exclude `target/`, WAL, logs).
*   [`audit/`](audit/) - Raw tarballs of corrupted WAL files or specific execution artifacts for forensic analysis.
