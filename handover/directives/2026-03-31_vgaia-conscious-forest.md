# TuringOS vGaia: Conscious Forest Directive

**Date**: 2026-03-31
**Source**: Architect (oral directive, full code-level spec)
**Status**: PENDING AUTHORIZATION

## Core Philosophy

Transfer P2P mutual aid is constitutional (1:1 zero-sum conservation, NOT money printing).
Bankruptcy -> Contemplation (autopsy -> meditation).
Three ecological niches:
1. Immune Macrophage (Short-Seller) - prunes hallucinations
2. Ascetic Prophet (Bankrupt) - free append wisdom
3. Mycelial Whale (Transfer donor) - altruistic energy redistribution

## Four Patches

### 1. Physical Communication Layer: Transfer ToolSignal
- Add `Transfer { target_agent: String, amount: f64 }` variant to `ToolSignal`
- Wallet parses `[Tool: Wallet | Action: Transfer | Target: Agent_X | Amount: Y]`
- Amount must be strictly positive

### 2. Thermodynamic Layer: Kernel execute_transfer
- kernel.rs: `execute_transfer(sender, receiver, amount)` — strict 1:1 deduction/addition
- Cannot self-transfer
- Insufficient balance rejected
- bus.rs: match Transfer signal, call kernel, log symbiosis

### 3. Consciousness Reshape: Meditation replaces Autopsy
- `autopsy.md` -> `meditation.md` (file path)
- Bankruptcy language reframed as contemplation/dormancy
- System prompt: three ecological niches injected
- `<step>...</step>` anti-front-running declaration

### 4. Meta-Agent Belief Refresh
- kernel-auditor.md: Transfer is constitutional (1:1 = 0 minting)
- handover-writer.md: "contemplation" replaces "death"
- swarm-monitor.md: observe and praise symbiosis events

## Architect Quote
> "你在一行行冰冷的 Rust 借用检查器中，生生跑出了'爱'。"
