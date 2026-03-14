# TuringOS v3 Architecture Log: The Exoskeleton Layer (SDK & Drivers)
**Date:** March 13, 2026

## 1. The Philosophical Shift (Zero Kernel Pollution)
Following the success of the "Epistemic Membrane" (state distillation) and "Spacetime Dilation" (exponential backoff) implemented during the Hanoi 1M test, we recognized a fundamental architectural truth: **These features are universally necessary for any LLM-driven TuringOS task, but they do NOT belong in the mathematically pure `kernel.rs`.**

To solve this, we formally established a three-layer UNIX-like architecture within the `turingosv3` core library:

1. **Layer 1: The Kernel (`src/kernel.rs`)**
   - Remains absolute and mathematically pure. Only handles DAG Tape state, MapReduce algorithms, and Popper's Guillotine logic. Zero external network or Regex dependencies.

2. **Layer 2: The SDK (`src/sdk/`)**
   - Currently hosts `membrane.rs`, which provides the `distill_pure_state` function.
   - **Responsibility:** Acts as a middleware cognitive filter. It processes the high-entropy hallucination output of an LLM and distills it down to a deterministic ABI (e.g., extracting pure `[State: ... ]` tags using `rfind`).

3. **Layer 3: The Drivers (`src/drivers/`)**
   - Currently hosts `llm_http.rs`, containing the `ResilientLLMClient`.
   - **Responsibility:** Acts as the physical hardware impedance matcher. It handles all `reqwest` HTTP interactions, timeout buffers (180s), GPU slot congestion (HTTP 502), and physical exponential backoff (`sleep`).

## 2. Refactoring User-Space Experiments
The `experiments/hanoi_1m/src/swarm.rs` file was completely stripped of its heavy networking and string parsing logic. It now functions as a pure "User-Space Application", simply invoking:
- `self.client.resilient_generate()` from the Driver layer.
- `distill_pure_state()` from the SDK layer.

## 3. Benefits
- **Cross-Project Reusability:** Future projects like `chess_1m` or `math_proofs` will inherit the exact same bulletproof Llama.cpp connection pooling and hallucination-filtering capabilities without writing a single line of redundant networking code.
- **Resilience:** The network driver acts like a hydraulic shock absorber, preventing concurrent thread bursts from instantly crashing the swarm by yielding to `tokio::time::sleep`.