---
name: swarm-launch
description: Pre-flight checks and launch sequence for the MiniF2F swarm evaluator
user_invocable: true
---

# /swarm-launch — Swarm Launch Sequence

TuringOS-specific deployment workflow for the MiniF2F swarm.

## Pre-Flight Checks

### 1. Environment Verification
- Check `.env` file exists and contains `VOLCENGINE_API_KEY`
- Do NOT display the key value, only confirm presence

### 2. Build
```bash
cd /home/zephryj/projects/turingosv3
cargo build --release --bin full_test_evaluator
```
Must succeed before proceeding.

### 3. tmux Session
- Check if a tmux session already exists: `tmux list-sessions`
- If exists, warn user and ask whether to reuse or create new

### 4. User Confirmation
**MANDATORY**: Present launch parameters and wait for explicit user confirmation:
- Model: (from CLAUDE.md Layer 2)
- Concurrency N: (from CLAUDE.md Layer 2)
- Boltzmann Temperature: (from CLAUDE.md Layer 2)
- Binary: `target/release/full_test_evaluator`

### 5. Launch
Only after user confirms:
```bash
tmux new-session -d -s swarm 'cd /home/zephryj/projects/turingosv3 && ./target/release/full_test_evaluator'
```

### 6. Initial Monitoring
- Wait 5 seconds, then capture initial output
- Look for:
  - Market Ticker events (healthy start indicator)
  - Graveyard events (expected under Austrian Economics)
  - API errors (immediate problem indicator)
- Report initial status

## Abort Conditions
- Missing VOLCENGINE_API_KEY → ABORT
- cargo build fails → ABORT
- User declines confirmation → ABORT
