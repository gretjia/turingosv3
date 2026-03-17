# TuringOS v3 Handover & Memory

## Legacy Repositories (Read-Only Reference)

We have two "treasure" repositories from previous iterations:
- `/projects/turingos`
- `/projects/turingosv2`

Inside their respective `/handover/` directories, there are rich records of experiences, architectural decisions, and lessons learned from past versions.

**CRITICAL RULE FOR AI AGENTS:**
- When encountering problems or needing historical context, you are encouraged to **read and refer** to the `/handover/` folders of these legacy repositories.
- **NEVER** copy any files from `turingos` or `turingosv2` into this brand new `turingosv3` project. We must maintain a strict "zero pollution" policy for the v3 architecture.

## Core Disciplines & Architectural Rules

1. **Kernel Immutability:** The core kernel of this project (`kernel.rs`) must not be changed unless absolutely necessary and ONLY with explicit permission from the human architect.
2. **Unix-Like Elegance & Modularity:** Maintain the project as elegantly as Unix. Everything non-kernel must be hot-pluggable, residing completely outside the kernel, and communicating exclusively via APIs.
3. **LLM as an External Blackbox:** Always use industry-standard external APIs for LLM communication. NEVER internalize or hardcode LLM logic into the project.
4. **Architectural Alignment:** Any critical changes must strictly align with the principles defined in `bible.md` and the topology laid out in `topology.md`.
5. **No Hardcoding for Tests:** Never use hardcoding simply to pass a test. This system is designed to face countless real-world, highly complex problems in the future.
6. **Strict Separation of Experiments:** All test projects and benchmarks (such as the MAKER 1 Million Hanoi Test) are temporary test projects. Do not internalize them into the core program. They must be strictly separated from the main project at the file structure level (e.g., placed in an `experiments/` or `benchmarks/` directory).

## Local Directory Context

The `/handover/` directory in this v3 project is actively maintained to contain:
- [`README.md`](README.md) - This document, containing core rules and philosophy.
- [`turingosv3_maker_hanoi_audit.md`](turingosv3_maker_hanoi_audit.md) - The architectural and execution audit report from the 200-step Hanoi trial. Details critical flaws like O(N²) bottlenecks, lack of logical Guillotine implementation, and blind swarm agents.
- [`ai-direct/`](ai-direct/) - The operational handover states.
  - [`ai-direct/LATEST.md`](ai-direct/LATEST.md) - The real-time, frequently updated scratchpad of current work context, goals, and next steps for AI agents. Always read this when starting a session.