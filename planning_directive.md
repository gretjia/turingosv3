# Directive: The Laissez-faire Capitalist Upgrade (Austrian Economics Patch)

## Objective
Refactor TuringOS to eliminate fixed-price stakes and introduce a free-floating market economy with visible leaderboards, enabling true VC behavior and preventing deflationary deadlocks.

## Phases
1. **Wallet Tool Update**:
   - Update `src/sdk/tools/wallet.rs` and `src/sdk/tool.rs` to support `InvestOnly { target_node, amount }`.
   - Remove fixed stake assumptions; enforce `amount >= 1.0`.
   - Map LLM's stake directly to `YieldReward` (Intrinsic Reward) for self-proposals.
2. **Bus VC Injection**:
   - Update `src/bus.rs` to handle `InvestOnly`. If intercepted, inject the capital directly into the target node's `intrinsic_reward` and trigger `hayekian_map_reduce()`. Bypass the kernel append.
3. **Market Ticker**:
   - Add `get_market_ticker(top_n)` to `src/kernel.rs`.
   - Update `experiments/minif2f_swarm/src/swarm.rs` to query the ticker and inject it into the LLM prompt.
4. **Prompt Overhaul**:
   - Rewrite `skills/economic_operative.md` to reflect the free-floating, slash-and-burn Austrian economics rules.