# Sandbox Infrastructure Audit & Anti-Identity-Theft Upgrade (2026-03-16)

**Context:** During the MiniF2F full test suite execution ($N=50$), a critical "Identity Theft" bug was discovered where the LLM (DeepSeek-R1) would hallucinate/drift into proving a simpler, known theorem (`induction_11div10tonmn1ton`) instead of the assigned difficult problem. The Lean 4 compiler verified the proof as correct, and the system recorded a false positive success.

## 1. The "Identity Theft" Vulnerability
*   **Discovery:** Analysis of `/tmp/*.wal` files showed multiple different theorem files containing identical proofs for a completely unrelated induction problem.
*   **Root Cause:** The `Lean4MembraneSkill` only checked for mathematical correctness via the compiler. It lacked an "Identity Anchor" to ensure the proof corresponded to the specific theorem name being evaluated.
*   **Impact:** The previously reported "63/63" success rate was identified as logically contaminated and scientifically invalid.

## 2. Architectural Solution: Air-Gapped Sandbox Engine
To solve both the security (malicious code) and cognitive (identity theft) issues, we implemented a new abstraction layer.

### A. The Air-Gapped Oracle Trait (`src/sdk/sandbox.rs`)
*   Implemented `SandboxEngine` trait: A pure, stateless interface that swallows code and spits out truth (or error).
*   **LocalProcessSandbox:** A concrete implementation that uses memory pipes (`stdin`) to feed code to the compiler.
*   **Hardware Vision:** This design is `no_std` compatible at the interface level, preparing the kernel for future ASIC/FPGA embedding.

### B. Cognitive Defense (Anti-Identity-Theft)
*   **Identity Anchoring:** `Lean4MembraneSkill` now requires a `theorem_name` at instantiation.
*   **VETO Logic:** Added `check_identity_theft` which inspects the LLM payload. If it attempts to define a different theorem name or omits the target name, it is immediately **VETO'd** before reaching the compiler.

### C. Resource Melting Protection
*   Implemented a 10-second "Gas Limit" (Physical Timeout) via the sandbox to prevent LLM-generated infinite loops from hanging the OS bus.

## 3. Implementation Files
*   `src/sdk/sandbox.rs`: The new sandbox abstraction and local process implementation.
*   `src/sdk/mod.rs`: Added `sandbox` module.
*   `experiments/minif2f_swarm/src/lean4_membrane.rs`: Upgraded with sandbox support and Identity Theft checks.
*   `experiments/minif2f_swarm/src/bin/full_test_evaluator.rs`: Refactored to inject the sandbox into the skill bus.

## 4. Scientific Conclusion
The "Identity Theft" bug proved that **Formal Verification is not enough for Truth if the Context is hijacked.** By anchoring the theorem name in the Membrane, we have achieved a higher level of "Cognitive Integrity" required for genuine SOTA claims.
