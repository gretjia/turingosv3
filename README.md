# TuringOS v3

Welcome to TuringOS v3.

## Documents

* [Bible (`bible.md`)](bible.md) - The architectural philosophy and rules.
* [Topology (`topology.md`)](topology.md) - The system topology diagrams (Mermaid).

## Directory Structure

* [`src/`](src/) - The core TuringOS microkernel codebase.
* [`handover/`](handover/) - Documentation, architectural rules, audit reports, and AI state handover files. Start here if you are an AI agent.
  *   [`handover/README.md`](handover/README.md) - Core rules and legacy repo reference guidelines.
    *   [`handover/turingosv3_maker_hanoi_audit.md`](handover/turingosv3_maker_hanoi_audit.md) - The latest architecture execution audit.
    *   [`handover/sandbox_and_identity_theft_audit_20260316.md`](handover/sandbox_and_identity_theft_audit_20260316.md) - Deep analysis of the "Identity Theft" vulnerability.
  *   [`handover/extreme_purification_audit_20260317.md`](handover/extreme_purification_audit_20260317.md) - Architecture upgrade to real-time Hayekian pricing (Heartbeat=1).
  *   [`handover/boltzmann_retreat_audit_20260317.md`](handover/boltzmann_retreat_audit_20260317.md) - Implementation of Boltzmann Softmax selection.
  *   [`handover/inversion_of_control_pricing_audit_20260317.md`](handover/inversion_of_control_pricing_audit_20260317.md) - The philosophical refactoring of the pricing engine to maintain kernel purity via `intrinsic_reward`.
  *   [`handover/qwen_397b_execution_audit_20260317.md`](handover/qwen_397b_execution_audit_20260317.md) - **[LATEST]** Deep analysis of the Boltzmann Router in action during the Qwen 397B formal verification run.

* [`experiments/`](experiments/) - Temporary test projects separated from the core kernel.
  * [`experiments/hanoi_1m/`](experiments/hanoi_1m/) - The MAKER 1-Million Hanoi Test swarm benchmark environment.
