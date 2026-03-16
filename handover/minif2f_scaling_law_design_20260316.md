# MiniF2F Swarm - Test-Time Compute Scaling Law Design (2026-03-16)

## 1. The Objective: Empirically Proving the Scaling Law
To robustly demonstrate that the TuringOS v3 architecture harnesses multi-agent emergence to solve complex formal mathematics (where single models fail), we are implementing a rigorous **Test-Time Compute Scaling Law** experiment. 

The goal is to map the pass rate on the MiniF2F Lean 4 benchmark against an exponentially increasing number of concurrent reasoning agents ($N$).

## 2. The 4-Tier Gradient Design
We will evaluate the system across 4 distinct concurrency tiers to generate a scientifically compelling performance curve:

1.  **Tier 1: $N=1$ (The Baseline)**
    *   **Purpose**: Simulates standard zero-shot `pass@1` performance of `DeepSeek-R1-Distill-Qwen-32B`.
    *   **Expectation**: Low pass rate. Will expose the inherent hallucination rate and local optima traps of the base model when unaided by a swarm.

2.  **Tier 2: $N=10$ (The Emergence)**
    *   **Purpose**: The critical threshold to prove $N > N-1$.
    *   **Expectation**: A noticeable spike in the pass rate as the system begins to leverage basic Monte Carlo Tree Search (MCTS) capabilities and error pruning via the Lean 4 Membrane.

3.  **Tier 3: $N=50$ (The Sweet Spot)**
    *   **Purpose**: The optimal operational tier discovered during Phase 1 & 2 testing.
    *   **Expectation**: High pass rate. Maximizes search tree width while remaining within stable API TPM (Tokens Per Minute) boundaries on the SiliconFlow Pro tier.

4.  **Tier 4: $N=100$ (The Asymptote / Extreme Stress Test)**
    *   **Purpose**: To identify the point of diminishing returns in the scaling law and stress-test the `ResilientLLMClient`'s exponential backoff mechanisms against severe API rate limits (~400,000 TPM).

## 3. Implementation Plan: The 20-Problem Pilot
To efficiently validate this gradient without incurring massive initial API costs on the full 244-problem `Test` split:
1.  **Batch Evaluator**: A new Rust binary (`batch_evaluator.rs`) will be developed to automate the ingestion of Lean 4 files, parameterize the swarm size, and automatically score outcomes (Pass/Fail/Timeout).
2.  **The Pilot Run**: We will randomly sample 20 theorems from the MiniF2F `Valid` split.
3.  **Execution**: The Batch Evaluator will run these 20 theorems across all 4 tiers ($N=1, 10, 50, 100$) and generate a data matrix proving the logarithmic returns of Swarm intelligence.