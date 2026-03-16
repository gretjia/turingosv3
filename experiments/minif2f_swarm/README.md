# MiniF2F Swarm (Lean 4)

This experiment uses 50 concurrent `DeepSeek-R1-Distill-Qwen-32B` agents via Volcengine (or SiliconFlow Pro) to solve formal mathematics problems from the MiniF2F dataset using the Lean 4 proof assistant.

## Architecture

This follows the TuringOS v3 Star-Topology Microkernel:
1.  **Kernel**: The immutable core that records Lean 4 tactic states as physical facts. It blindly executes MapReduce value backpropagation.
2.  **Swarm**: 50 concurrent agents exploring the proof tree.
3.  **Skills**:
    *   `ThermodynamicHeartbeatSkill`: Skips intermediate MapReduce executions to save CPU.
    *   `Lean4MembraneSkill`: The absolute Popperian guillotine. It intercepts every payload, runs it through the local `lean` compiler, and immediately Vetoes the state if the compiler throws an error. Only perfectly compiling tactics are allowed into the Kernel.

## Flow

1. Initial state: Theorem statement.
2. Swarm generates a tactic.
3. `Lean4MembraneSkill` runs the tactic against the theorem.
4. If successful, state is appended. If theorem is `No goals`, MapReduce is triggered with massive bounty.
