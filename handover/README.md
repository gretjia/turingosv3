# TuringOS Handover Documentation

This directory is the central repository for AI Agents and Human Architects to understand the theoretical, philosophical, and operational state of the TuringOS v3 project.

**Rule #1:** If you are an AI Agent waking up in this workspace, you MUST read `ai-direct/LATEST.md` before taking any action.

## 1. Core Philosophical Texts
The foundation of the Star-Topology Microkernel and Thermodynamic Swarm computation.
*   [`bible.md`](bible.md) - The fundamental architectural philosophy and rules of TuringOS. (Do not modify without explicit human authorization).
*   [`ALIGNMENT.md`](ALIGNMENT.md) - **Master Alignment Document** — all rules in precedence order, from Layer 1 invariants to zero-hardcode principles.
*   [`topology.md`](topology.md) - Conceptual Mermaid diagrams of the Swarm logic and Kernel map-reduce.
*   [`project_topology.md`](project_topology.md) - Network and hardware infrastructure mappings for the intra-network (`omega-vm`, `mac-studio`, `linux`).
*   [`network_topology_and_ssh.md`](network_topology_and_ssh.md) - Specifics on Reverse Tunnels, WireGuard, and API routing.

## 2. Architect Directives (Chronological)
Immutable archive of architecture-level decisions from the human architect.
*   [`directives/2026-03-19_anti-oreo-free-pricing.md`](directives/2026-03-19_anti-oreo-free-pricing.md) - Anti-Oreo three-boundary topology + free pricing
*   [`directives/2026-03-19_austrian-economics-laissez-faire.md`](directives/2026-03-19_austrian-economics-laissez-faire.md) - Austrian economics laissez-faire
*   [`directives/2026-03-19_big-bang-multiverse-entropy.md`](directives/2026-03-19_big-bang-multiverse-entropy.md) - Big Bang multiverse + entropy
*   [`directives/2026-03-19_time-arrow-grand-unification.md`](directives/2026-03-19_time-arrow-grand-unification.md) - Time Arrow grand unification
*   [`directives/2026-03-20_magna-carta-plan-spec.md`](directives/2026-03-20_magna-carta-plan-spec.md) - Magna Carta plan & spec
*   [`directives/2026-03-20_magna-carta-vfinal.md`](directives/2026-03-20_magna-carta-vfinal.md) - **Magna Carta final** — Three Laws + Four Engines
*   [`directives/2026-03-21_lockfree-actor-plan-spec.md`](directives/2026-03-21_lockfree-actor-plan-spec.md) - Lock-free Actor Model plan & spec
*   [`directives/2026-03-21_lockfree-austrian-naked-swarm.md`](directives/2026-03-21_lockfree-austrian-naked-swarm.md) - Lock-free Austrian naked swarm
*   [`directives/2026-03-23_run1-postmortem-four-remedies.md`](directives/2026-03-23_run1-postmortem-four-remedies.md) - **Run 1 Postmortem** — Boltzmann routing + generation rebirth + superfluid clearing + wallet semantics
*   [`directives/2026-03-23_async-relativistic-universe.md`](directives/2026-03-23_async-relativistic-universe.md) - Relativistic async universe + configurable membrane + WAL preservation

## 3. Architect Insights (Condensed Principles)
Non-obvious design principles extracted from architect conversations. Each ≤50 chars.
*   [`architect-insights/2026-03-20_price-is-experience.md`](architect-insights/2026-03-20_price-is-experience.md) - 质押价格 = Agent 历史经验的压缩编码
*   [`architect-insights/2026-03-20_hayekian-free-market.md`](architect-insights/2026-03-20_hayekian-free-market.md) - 自由浮动质押，价格发现替代中央调度
*   [`architect-insights/2026-03-20_agent-skill-dna.md`](architect-insights/2026-03-20_agent-skill-dna.md) - Skill 即 DNA，幸存 Agent 的演化资产
*   [`architect-insights/2026-03-23_compute-delegation-emergence.md`](architect-insights/2026-03-23_compute-delegation-emergence.md) - 经济高压下 LLM 自发将穷举委派给编译器 ALU
*   [`architect-insights/2026-03-23_fast-slow-heterogeneous-division.md`](architect-insights/2026-03-23_fast-slow-heterogeneous-division.md) - 快模型扫雷填墓，慢模型读墓精准狙击

## 4. Architecture Audits & Upgrades (Chronological)
Detailed post-mortem reports and architectural upgrade logs. These explain *why* the codebase is written the way it is.

### March 14, 2026
*   [`ram_volatility_threat_20260314.md`](ram_volatility_threat_20260314.md) - Introduction of WAL (Write-Ahead Log) for state persistence.
*   [`microkernel_harness_architecture_20260314.md`](microkernel_harness_architecture_20260314.md) - Formalization of the `Kernel` vs `TSP Bus` vs `SKILL` separation.
*   [`engineering_lessons_20260314.md`](engineering_lessons_20260314.md) - Summary of early Rust trait abstractions.
*   [`kv_cache_avalanche_audit_20260314.md`](kv_cache_avalanche_audit_20260314.md) - KV cache avalanche analysis.
*   [`truncation_audit_20260314.md`](truncation_audit_20260314.md) - Truncation issue analysis.

### March 15, 2026
*   [`concurrency_cognitive_divergence_audit_20260315.md`](concurrency_cognitive_divergence_audit_20260315.md) - Analysis of N=100 parallel branches and the "God Jump" phenomenon.

### March 16, 2026 (MiniF2F Lean 4 Implementation)
*   [`sandbox_and_identity_theft_audit_20260316.md`](sandbox_and_identity_theft_audit_20260316.md) - LLM hallucinating proofs for simpler theorems → `SandboxEngine` + "Identity Anchoring".
*   [`minif2f_data_contamination_audit_20260316.md`](minif2f_data_contamination_audit_20260316.md) - Lean 4 Popperian Membrane prevents dataset contamination.
*   [`minif2f_phase1_audit_20260316.md`](minif2f_phase1_audit_20260316.md) - MiniF2F Phase 1 audit.
*   [`minif2f_phase2_emergence_audit_20260316.md`](minif2f_phase2_emergence_audit_20260316.md) - MiniF2F Phase 2 emergence.
*   [`minif2f_phase3_induction_audit_20260316.md`](minif2f_phase3_induction_audit_20260316.md) - MiniF2F Phase 3 induction.
*   [`minif2f_scaling_law_design_20260316.md`](minif2f_scaling_law_design_20260316.md) - Scaling law experimental design.
*   [`minif2f_scaling_law_results_20260316.md`](minif2f_scaling_law_results_20260316.md) - Scaling law results.
*   [`star_topology_and_1m_hanoi_audit_20260316.md`](star_topology_and_1m_hanoi_audit_20260316.md) - Star topology + 1M Hanoi audit.
*   [`minif2f_mac_studio_crash_clues_20260316.md`](minif2f_mac_studio_crash_clues_20260316.md) - Mac Studio crash investigation.

### March 17, 2026 (The SOTA Run Physics Updates)
*   [`extreme_purification_audit_20260317.md`](extreme_purification_audit_20260317.md) - Upgrade to real-time Hayekian pricing (Heartbeat=1).
*   [`boltzmann_retreat_audit_20260317.md`](boltzmann_retreat_audit_20260317.md) - Greedy Routing → Softmax Probabilistic Backtracking.
*   [`inversion_of_control_pricing_audit_20260317.md`](inversion_of_control_pricing_audit_20260317.md) - Removal of magic strings from `kernel.rs`, introduction of `intrinsic_reward`.
*   [`qwen_397b_execution_audit_20260317.md`](qwen_397b_execution_audit_20260317.md) - Qwen-397B "last-mile cognitive blind spot" analysis.
*   [`turing_capitalism_upgrade_audit_20260317.md`](turing_capitalism_upgrade_audit_20260317.md) - Proof-of-Stake economy, TOOL/SKILL lexical purge.

### March 18, 2026 (The Free Market Upgrade)
*   [`austrian_economics_patch_audit_20260318.md`](austrian_economics_patch_audit_20260318.md) - Laissez-faire economics, floating stakes, Market Tickers.

### March 19, 2026 (Claude Code Harness Migration)
*   Migrated from Gemini CLI Agent to **Claude Code** with three-layer governance (Hook → Skill → Agent).
*   Created [`CLAUDE.md`](../CLAUDE.md) — Project constitution.
*   Harness under [`.claude/`](../.claude/):
    *   **Hooks**: `block-destructive.sh`, `post-edit-validate.sh`, `stop-guard.sh`
    *   **Agents**: `kernel-auditor` (Opus), `swarm-monitor` (Sonnet), `handover-writer` (Sonnet)
    *   **Skills**: `/dev-cycle`, `/validate`, `/swarm-launch`, `/handover-update`, `/architect-ingest`

### March 20, 2026 (Magna Carta + Cross Audit)
*   [`cross_audit_report_20260319.md`](cross_audit_report_20260319.md) - Cross-model audit report.
*   [`audit-fix-plan-20260320.md`](audit-fix-plan-20260320.md) - Audit fix plan.

### March 23-24, 2026 (Actor Model + Four Remedies)
*   Actor Model implementation (watch + mpsc lock-free architecture)
*   Four Remedies: Boltzmann routing, generation rebirth, superfluid clearing, wallet semantics
*   zeta_sum_proof Run 1-3: from deadlock → OMEGA in 8 tx
*   Configurable Lean4MembraneTool + MathStepMembrane 20-char filter
*   WAL cross-epoch preservation
*   See: `experiments/zeta_sum_proof/audit/run1_analysis.md`

## 5. Operations
*   [`ai-direct/LATEST.md`](ai-direct/LATEST.md) - **Live session state** — single source of truth for current progress.
*   [`archive/`](archive/) - Deprecated architecture notes and raw diff patches.

## 6. Papers & Analysis
*   [`paper_draft_minif2f_scaling.md`](paper_draft_minif2f_scaling.md) - MiniF2F scaling law paper draft.
