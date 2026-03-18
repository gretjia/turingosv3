# TuringOS v3 Progress & State

## Current Core Mission
- We are running **TuringOS v3**, focusing on the **MiniF2F Lean 4 Formal Proof Swarm**.
- The ultimate goal is to solve formal mathematics problems by employing a massive, concurrent swarm of LLM agents (up to N=100) exploring proof trees in a custom execution engine.

## The Turing Capitalism Architecture
The system has fully migrated to a **Proof-of-Stake** and **Austrian Economics** model:
- **Free-Floating Stakes:** Agents decide their own cost to execute a node (`1.0` for low confidence, `1000.0` for high confidence). Capital generates topological gravity.
- **The Graveyard Protocol (`src/bus.rs`):** Agents that generate code causing a `Compiler/Sandbox Error` are immediately liquidated (capital burned). Their failure and the explicit Lean 4 compiler error are etched into a public `Graveyard` on the node, forcing future Agents to learn via In-Context Reflection.
- **The Market Ticker (`src/kernel.rs`):** The Kernel generates a dynamic top-3 node leaderboard based on `market_price`. This breaks the information silo and is injected into every prompt.
- **Venture Capital:** Agents facing dead ends can choose to act as VC investors, staking their funds on top leaderboard nodes rather than risking compilation errors, shifting computing power naturally to the most promising branches without centralized orchestration.

## Execution Environment & Hardware
- **Infrastructure:** Tests are primarily executed on `zephrymac-studio` using `tmux` sessions (e.g., `minif2f-sota-run`) to maintain persistence.
- **API Provider:** Due to extreme HTTP 401/429 backpressure and GPU slot exhaustion on SiliconFlow at high concurrency (N=30+), the workload has been completely migrated to **Volcengine Ark (火山引擎)**.
- **Active Model:** We are currently utilizing `doubao-1-5-pro-32k-250115` as the primary reasoning engine for the Swarm, as it natively supports high-throughput concurrent requests without truncating massive `<think>` blocks.

## Latest Incidents & Resolutions (March 17-18, 2026)
1. **The Deflationary Deadlock:** A hardcoded `500` coin tax previously caused the entire Swarm to bankrupt itself rapidly, grinding execution to a halt around step 30. Fixed via the aforementioned Austrian Economics patch (floating stakes + VC).
2. **Volcengine Migration Crisis:** The initial migration failed due to a missing `ep-` ID configuration. After extensive research, we verified that the Volcengine v3 Chat Completions API accepts direct model tags (`doubao-1-5-pro-32k-250115`). The remote `.env` on `zephrymac-studio` has been fully reconstructed with the correct API URL and API Key (`6ef79179-f1f6-484d-8258-585a9ff61b32`).
3. **Node.js Memory Exhaustion:** The local Gemini CLI Agent experienced an OOM (JavaScript heap out of memory) crash. Permanently fixed by appending `export NODE_OPTIONS="--max-old-space-size=8192"` to the system's `~/.bashrc` and `~/.zshrc`.

## Immediate Next Steps (Actionable for New Agents)
1.  **Restart the Swarm:** SSH into `zephrymac-studio`, connect to the `minif2f-sota-run` tmux session, ensure the `.env` is loaded, and start the `full_test_evaluator`. Command:
    ```bash
    cd ~/projects/turingosv3/experiments/minif2f_swarm
    export $(cat ~/projects/turingosv3/.env | xargs)
    cargo run --release --bin full_test_evaluator
    ```
2.  **Monitor the Graveyard:** Observe the terminal logs. You must verify that `[MARKET CASUALTY]` events are firing and successfully preventing early-stage bankruptcy loops via the In-Context Reflection mechanism.
3.  **Monitor VC Activity:** Check if the prompt injections of the `=== 📈 GLOBAL MARKET LEADERBOARD ===` are successfully inducing `ToolSignal::InvestOnly` actions from the LLMs.
