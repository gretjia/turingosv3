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
3.  **Air-Gapped Sandboxing**: Complete isolation of the LLM from the host filesystem during Lean 4 verification, operating exclusively over memory pipes.

## 3. Directory Structure & Documentation

*   [`src/`](src/) - The core TuringOS microkernel codebase.
*   [`experiments/`](experiments/) - Temporary test projects separated from the core kernel.
    *   [`experiments/minif2f_swarm/`](experiments/minif2f_swarm/) - The current SOTA execution harness for 244 Lean 4 theorems.
*   [`handover/`](handover/) - **[START HERE FOR AI AGENTS]** Documentation, architectural rules, audit reports, and AI state handover files. 
    *   [`handover/README.md`](handover/README.md) - The core philosophy, rules, and directory index of the handover assets.
    *   [`handover/ai-direct/LATEST.md`](handover/ai-direct/LATEST.md) - The absolute single source of truth for the current execution state of the workspace.
*   [`archive/`](archive/) - Deprecated patches and logs.
*   [`audit/`](audit/) - Raw tarballs of corrupted WAL files or specific execution artifacts for forensic analysis.
