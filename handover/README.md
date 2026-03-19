# TuringOS Handover Documentation

This directory is the central repository for AI Agents and Human Architects to understand the theoretical, philosophical, and operational state of the TuringOS v3 project. 

**Rule #1:** If you are an AI Agent waking up in this workspace, you MUST read `ai-direct/LATEST.md` before taking any action.

## 1. Core Philosophical Texts
The foundation of the Star-Topology Microkernel and Thermodynamic Swarm computation.
*   [`bible.md`](bible.md) - The fundamental architectural philosophy and rules of TuringOS. (Do not modify without explicit human authorization).
*   [`topology.md`](topology.md) - Conceptual Mermaid diagrams of the Swarm logic and Kernel map-reduce.
*   [`project_topology.md`](project_topology.md) - Network and hardware infrastructure mappings for the intra-network (`omega-vm`, `mac-studio`, `linux`).
*   [`network_topology_and_ssh.md`](network_topology_and_ssh.md) - Specifics on Reverse Tunnels, WireGuard, and API routing.

## 2. Architecture Audits & Upgrades (Chronological)
Detailed post-mortem reports and architectural upgrade logs. These explain *why* the codebase is written the way it is.

### March 14, 2026
*   [`ram_volatility_threat_20260314.md`](ram_volatility_threat_20260314.md) - Introduction of WAL (Write-Ahead Log) for state persistence.
*   [`microkernel_harness_architecture_20260314.md`](microkernel_harness_architecture_20260314.md) - Formalization of the `Kernel` vs `TSP Bus` vs `SKILL` separation.
*   [`engineering_lessons_20260314.md`](engineering_lessons_20260314.md) - Summary of early Rust trait abstractions.

### March 15, 2026
*   [`concurrency_cognitive_divergence_audit_20260315.md`](concurrency_cognitive_divergence_audit_20260315.md) - Analysis of $N=100$ parallel branches and the "God Jump" phenomenon.

### March 16, 2026 (MiniF2F Lean 4 Implementation)
*   [`sandbox_and_identity_theft_audit_20260316.md`](sandbox_and_identity_theft_audit_20260316.md) - Deep analysis of LLMs hallucinating proofs for simpler theorems and the creation of `SandboxEngine` and "Identity Anchoring".
*   [`minif2f_data_contamination_audit_20260316.md`](minif2f_data_contamination_audit_20260316.md) - Proof that the Lean 4 Popperian Membrane mathematically prevents dataset contamination cheating.

### March 17, 2026 (The SOTA Run Physics Updates)
*   [`extreme_purification_audit_20260317.md`](extreme_purification_audit_20260317.md) - Upgrade to real-time Hayekian pricing (Heartbeat=1).
*   [`boltzmann_retreat_audit_20260317.md`](boltzmann_retreat_audit_20260317.md) - The transition from Greedy Routing to Softmax Probabilistic Backtracking.
*   [`inversion_of_control_pricing_audit_20260317.md`](inversion_of_control_pricing_audit_20260317.md) - The removal of magic strings from `kernel.rs` and the introduction of `intrinsic_reward` yielded by SKILLs.
*   [`qwen_397b_execution_audit_20260317.md`](qwen_397b_execution_audit_20260317.md) - Empirical analysis of Qwen-397B's "last-mile cognitive blind spot" during the formal verification of Theorem 2.
*   [`turing_capitalism_upgrade_audit_20260317.md`](turing_capitalism_upgrade_audit_20260317.md) - **[LATEST]** The epic architectural shift to a Proof-of-Stake economy, including the lexical purge separating TOOLs from SKILLs.

### March 18, 2026 (The Free Market Upgrade)
*   [`austrian_economics_patch_audit_20260318.md`](austrian_economics_patch_audit_20260318.md) - **[LATEST]** The strict implementation of Laissez-faire economics, floating stakes, Market Tickers, and the migration to Volcengine Ark (Doubao-2.0-Pro) to sustain extreme N=30 concurrency.

### March 19, 2026 (Claude Code Harness Migration)
*   Migrated from Gemini CLI Agent to **Claude Code** with full three-layer governance (Hook → Skill → Agent).
*   Created [`CLAUDE.md`](../CLAUDE.md) — Project constitution with Layer 1/2 invariants and engineering rules.
*   Created [`VIA_NEGATIVA.md`](../VIA_NEGATIVA.md) — Registry of proven-false paths.
*   Harness files under [`.claude/`](../.claude/):
    *   **Hooks**: [`block-destructive.sh`](../.claude/hooks/block-destructive.sh) (PreToolUse guard), [`post-edit-validate.sh`](../.claude/hooks/post-edit-validate.sh) (auto `cargo check`), [`stop-guard.sh`](../.claude/hooks/stop-guard.sh) (uncommitted change reminder)
    *   **Agents**: [`kernel-auditor.md`](../.claude/agents/kernel-auditor.md) (Opus, read-only 8-point audit), [`swarm-monitor.md`](../.claude/agents/swarm-monitor.md) (Sonnet, runtime diagnostics), [`handover-writer.md`](../.claude/agents/handover-writer.md) (Sonnet, session handover)
    *   **Skills**: [`/dev-cycle`](../.claude/skills/dev-cycle/SKILL.md), [`/validate`](../.claude/skills/validate/SKILL.md), [`/swarm-launch`](../.claude/skills/swarm-launch/SKILL.md), [`/handover-update`](../.claude/skills/handover-update/SKILL.md), [`/architect-ingest`](../.claude/skills/architect-ingest/SKILL.md)

## 3. Operations
*   [`ai-direct/LATEST.md`](ai-direct/LATEST.md) - Live execution state (Current: MiniF2F N=30 run on `zephrymac-studio` via Volcengine Ark).
*   [`archive/`](archive/) - Deprecated architecture notes and raw diff patches.
