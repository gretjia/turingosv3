# TuringOS v3 Architecture Debugging Log: Multi-Tape Refactoring & Local Deployment
**Date:** March 13, 2026

## 1. The Context Corruption (Self-Poisoning) Loop
**Symptom:** After implementing the requirement to pass the previous exact state to the next agent (Markov state property), the system successfully solved Step 1 but entered an infinite rejection loop (`[-] REJECTED`) on Step 2.
**Root Cause:** Qwen 27B's "Reasoning Content" (`<think>...</think>`) and raw generation tokens (`[Moves: ...]`) were blindly concatenated into `payload` and re-injected verbatim into the `Current State` prompt for the *next* agent. The LLM received its own internal monologue as environmental context, causing catastrophic format parsing failures.
**Solution (Detail Encapsulation):**
- Strict string extraction logic added in `swarm.rs`. Only the sub-string matching the absolute physical state boundaries (`[State: ... ]`) is carried forward. All preceding `Thinking Process` chains are explicitly truncated before being passed to the next step's context window. This perfectly enforces the "Shielding Details" architecture law.

## 2. The 120-Second "Time-Avalanche" in High-Concurrency Swarms
**Symptom:** During the local physical test on `windows1-w1`, the `network_test` binary consistently failed precisely at the 120-second mark with `HTTP Request Failed: error sending request for url` or `500/502 Bad Gateway`, trapping the DAG in an un-advancing state.
**Root Cause:**
- We increased `max_tokens` from 100 to 4096 to accommodate Qwen 3.5 27B's necessary Chain-of-Thought reasoning.
- 4 concurrent Agents bombarded a single physical GPU (Llama.cpp `0.0.0.0:8080`).
- The 1 GPU processing 4 massive 500+ token context streams took approximately 47-55 seconds *per generation*. Context switching caused latency to spike well past 2 minutes.
- The `reqwest` Rust client possessed a hardcoded `.timeout(Duration::from_secs(120))`, causing the client to brutally terminate the connection seconds before the LLM could return the completed payload.
**Solution:**
- Decoupled timeout via `std::env::var("LLAMA_TIMEOUT")`. Defaults to `1800` (30 minutes) to prevent aggressive client-side network cuts during heavy multi-agent inference.
- Implemented exponential backoff and jitter (`tokio::time::sleep(Duration::from_secs(5 * attempt))`) to gracefully handle Llama.cpp Queue Full (500/502) errors without crashing the Swarm thread.

## 3. Physical Backend Architecture (Windows as a Dumb Calculator)
**Symptom:** SSH tunnels over 100.x.x.x tailscale IPS were proving fragile for days-long uninterrupted multi-million test runs. Running Rust directly via Cargo on Windows faced intense cross-compilation errors (`x86_64-pc-windows-gnu` missing linkers).
**Root Cause/Philosophy:** We must keep the Linux Omega-VM as the control node but execute physically adjacent to the GPU to avoid network partitioning.
**Solution:**
- **Zero-Dependency Native Push:** Installed `mingw-w64` on Linux, cross-compiled the `hanoi_1m` project to a standalone `.exe`, and SCP'd it to Windows.
- Started `llama-server.exe` on Windows with `-np 4` (critical for supporting 4 parallel request slots, otherwise defaults to 1).
- Executed the binary natively on Windows in `nohup`-like background mode, redirecting logs directly to `D:\turingos_nonstop_run.log`, achieving 0ms network overhead between the OS and the Agent's "Brain".

## Rule for Future Agents
- NEVER hardcode LLM connection variables (Timeouts, URLs).
- ALWAYS assume LLM payloads are dirty. Sanitize the output (Red-Flagging) *and* sanitize what is carried forward to the next Context. 
- ALWAYS verify the GPU backend's concurrent slot size (`-np` or equivalent) matches the `swarm_size` to prevent HTTP 502/Queue blocks.