# 'Extreme Purification' Upgrade Audit (2026-03-17)

**Context:** Analysis of the previous $N=50$ run with Qwen 3.5 397B revealed logic stagnation and API 503 instability. Despite the high-parameter model, the kernel was only triggering MapReduce every 10 steps, allowing low-quality branches to accumulate and pollute the LLM context.

## 1. Technical Audit: The 'Fatigue' Mystery
*   **The 10-Step Blind Spot:** The `ThermodynamicHeartbeatSkill` was set to a threshold of 10. This meant the system was effectively "blind" to proof quality for 10 steps at a time, allowing repetitive tactics (like endless `norm_num`) to take root.
*   **API Backpressure:** $N=50$ with a 397B model hit the physical limits of the SiliconFlow Pro tier, causing massive 503 errors and shrinking the actual search width to near-zero.

## 2. Upgrade: The 'Extreme Purification' Plan
We have implemented a more aggressive, high-frequency kernel strategy.

### A. Real-Time Hayekian Pricing
*   **Heartbeat Threshold: $10 \to 1$**: The kernel now triggers a MapReduce `REDUCE` operation after **every single Tactic append**.
*   **Instant Pruning:** Sub-optimal proof branches are now decapitated instantly, forcing the LLM swarm to focus only on the single highest-priced (most mathematically promising) proof head.

### B. High-Reliability Brain: DeepSeek-V3
*   **Model Switch:** Migrated from `Qwen-3.5-397B` to `deepseek-ai/DeepSeek-V3`.
*   **Rationale:** V3 offers a superior balance of logic depth and API stability. It is less prone to the "HardwareTruncation" errors seen with the MoE 397B flagship under load.

### C. Search Space Optimization
*   **Concurrency: $N=50 \to 20$**: Reduced search width to guarantee that every branch gets a reliable API slot. High-quality $N=20$ is scientifically superior to broken $N=50$.
*   **Endurance: Max Steps $50 \to 100$**: Doubled the survival time for complex proofs, allowing the "Extreme Purification" kernel to support long-form symbolic reasoning.

## 3. Deployment Status
*   **Location:** `zephrymac-studio`
*   **Process:** Running in tmux session `minif2f-sota-run`.
*   **Purge:** All previous WAL files have been deleted to start the 244-theorem SOTA run from a state of total purity.
