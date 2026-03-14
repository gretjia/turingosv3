# TuringOS v3 Architecture: Microkernel & Harness (2026-03-14)

## The Microkernel Philosophy
Following the principle of "Separation of Mechanism and Policy", the TuringOS v3 architecture has been strictly partitioned into three concentric layers. The inner core remains mathematically pure, while all engineering hacks and thermodynamic friction are handled by the outer harness.

### Layer 1: The Sacred Kernel (`src/kernel.rs`)
- **Mechanism Only**: Pure Turing State Machine, DAG Causality Graph, and Hayekian MapReduce. 
- **Absolute Rigidity**: No retries, no LLM context, no error handling. It demands pure physical states (`[State: ...]`) or it burns the stake. It is mathematically frozen and must never be polluted with external network logic.

### Layer 2: The Project Harness & Watchdog (`experiments/hanoi_1m/src/harness.rs`)
This is the dynamic engineering shell that protects the kernel from the chaos of the physical LLM endpoints.

1. **Cognitive Divergence (N > N-1 Efficiency)**:
   - To prevent identical agents from crashing into the same token truncation wall, the `AgentSupervisor` injects a staggered temperature gradient (`0.2` to `0.8`) based on the agent's ID. This forces heterogeneous reasoning paths in the LLM's latent space.
2. **Defeating the Truncation Paradox**:
   - Overrides the default LLM max generation limit by forcibly passing `"max_tokens": 8192` in the HTTP driver payload, ensuring the model has enough physical room to finish its `<think>` process and output the mandatory state tag.
3. **The Non-Stop Watchdog (Zombification Defense)**:
   - **Strikes 1-3**: Permits thermodynamic fluctuations (minor hallucinations) with simple backoff retries.
   - **Strikes 4-8 (Self-Heal)**: Dynamically appends a `[SYSTEM SOS]` meta-prompt to force the LLM to summarize its thoughts and converge.
   - **Strikes >8 (SuspendAndSOS)**: Declares the specific agent thread mathematically zombified. It suspends the thread indefinitely and screams for Human-in-the-Loop (HITL) intervention. **Crucially, the other N-1 agents continue to execute flawlessly, guaranteeing the OS never halts.**

### Layer 3: Physical Endpoints
- The raw `llama.cpp` or external API endpoints. Protected by the Layer 2 Harness.

*This documentation serves to cement the project's engineering trajectory. Future contributors may expand Layer 2 (e.g., adding Anthropic drivers, Telegram bots), but Layer 1 is off-limits.*