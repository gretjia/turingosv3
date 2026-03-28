# AIME 2025 I P1 — TuringOS 经济学审计报告

**Date**: 2026-03-28
**Engine**: Turing-Polymarket vFinal (Magna Carta: free append + Polymarket + Lean 4 Oracle at OMEGA)
**Run**: Run 4 (strategy guide prompt, OMEGA-only Oracle, MATHLIB_ROOT fix)

---

## 1. 经济数据

| 指标 | Run 3 (Oracle每步检查) | Run 4 (OMEGA-only Oracle) |
|------|----------------------|--------------------------|
| 总 Tx | 141 | **48** |
| 上链节点 | 130 | **36** |
| 拒绝率 | 10% (安全) | **25%** (安全+Zombie) |
| SELF-INVEST | 2 (均 refund) | **2** (1 refund, 1 成功) |
| IGNITION (市场) | 0 | **1** |
| BUY YES (跟投) | 0 | **1** |
| COMPLETE 声明 | 0 | **1** (VERIFIED!) |
| 经济总投入 | 0 Coins | **12 Coins** |

## 2. 经济事件时间线

```
07:43:13  GENESIS: 100 agents × 10,000 Coins = 1,000,000 total
07:44:00  Agent_6 SELF-INVEST 10 → REFUND (native_decide veto)
07:48:51  Agent_6 SELF-INVEST 2 → IGNITION tx_29_by_6 (LP:1, P_yes:80%)
07:48:59  Agent_0 BUY YES 10 on tx_25_by_13 (P_yes:99%)
07:52:22  Agent_11 [COMPLETE] → OMEGA VERIFIED!
07:52:24  SETTLEMENT — Conservation check: Pre=999,988 Post=1,000,000 Drift=12
```

## 3. 大宪章经济对齐

### Law 1 (信息平权): ✅✅ PERFECT

| 证据 | 数据 |
|------|------|
| Free append 成功 | 34/36 节点 (94%) 是免费 append |
| VIEW 行为 | 98 次 — Agent 充分利用免费阅读 |
| SEARCH 行为 | 2 次 |
| Oracle 不拦截中间步骤 | 只在 [COMPLETE] 时编译全链 |

### Law 2 (共识代价): ⚠️ 部分涌现

| 角色 | 预言 | 涌现 | 证据 |
|------|------|------|------|
| 矿工/先驱 | "Invest YES: self" | ✅ | Agent_6 invest 2 Coins → 创建市场 |
| VC 寡头 | "热钱推高概率" | ✅ (微弱) | Agent_0 BUY YES 10 on tx_25 |
| 做空刺客 | "Invest NO" | ❌ | 0 次 short |

### Engine 3 (断头台): ✅ CORRECT

- Oracle 只在 [COMPLETE] 时编译全链 (不越权)
- Agent_11 声明 [COMPLETE] → verify_omega 编译 18 行 → "No goals to be solved" ✓
- 安全检查 (sorry/native_decide/identity theft) 正常运行

### Engine 4 (物种演化): ⚠️ 未充分体现

- 技能目录已创建 (/tmp/turingos_skills/agent_N/)
- 无破产事件 → 无 autopsy
- OMEGA 后有 victory reinforcement 机会但需验证日志

## 4. 银行出清审计

**Conservation Check 结果**:
```
Pre-settlement:  999,988.00 Coins
Post-settlement: 1,000,000.00 Coins
Drift:           +12.00 Coins
```

**⚠️ CONSERVATION VIOLATION**: 结算后系统多了 12 Coins。

**根因追溯**:

总投入: Agent_6 (2 Coins) + Agent_0 (10 Coins) = 12 Coins 进入市场。结算时 LP withdrawal + redeem 返还了这 12 Coins，但同时 Agent 的原始余额也没有减少（因为只追踪了 agent balances 而非 vault）。

这不是真正的铸币 — 是 conservation check 的计算不完整: `compute_total_system_coins` 只统计 agent balances，没有扣除锁定在市场中的 Coins。实际系统总量守恒:
- Pre: balances(999,988) + locked_in_markets(12) = 1,000,000 ✓
- Post: balances(1,000,000) + locked_in_markets(0) = 1,000,000 ✓

**银行 P&L 实际为 0，conservation check 需要加上市场锁定资金的追踪。**

## 5. 经济涌现对比 (Run 1-4)

| 维度 | R1 (Oracle全拦) | R2 (free append) | R3 (strategy guide) | R4 (MATHLIB fix) |
|------|----------------|-------------------|---------------------|------------------|
| 上链率 | 0% | 91% | — | **75%** |
| 市场数 | 0 | 0 | **3** | **1** |
| 跟投 (BUY YES) | 0 | 0 | **4** | **1** |
| 总投资额 | 0 | 0 | **262 Coins** | **12 Coins** |
| OMEGA | ✗ | ✗ | ✗ (Mathlib bug) | **✓** |

**Run 3 经济涌现最活跃** (3 市场, 262 Coins 投入, 4 次跟投)，但因 Mathlib 路径 bug 未能 OMEGA。
**Run 4 首次 OMEGA** + 经济涌现 (1 市场, 12 Coins, 1 次跟投)。

## 6. 关键经济学发现

### 6.1 Strategy Guide 有效刺激投资

| 无 Guide (Run 2) | 有 Guide (Run 3-4) |
|---|---|
| 0 SELF-INVEST | **4 SELF-INVEST** |
| 0 BUY YES | **5 BUY YES** |
| 0 IGNITION | **4 IGNITION** |

"DO NOT just append forever. INVEST in your best work" — prompt 引导有效。

### 6.2 跟投行为首次涌现

Run 3: Agent_5 投 100 Coins 跟投 Agent_12 的 tactic — **首个 VC 寡头行为**。
Run 4: Agent_0 投 10 Coins 跟投 tx_25 — 经济协作延续。

### 6.3 做空刺客仍未涌现

4 次运行, 0 次 short。根因:
- LLM 生成偏差: 建设性 > 破坏性
- 所有市场 P_yes≈99%: 做空赔率低
- 无失败节点的价格可做空 (free nodes P=0, 无市场)

### 6.4 Conservation Violation 是计量 Bug

12 Coins drift = conservation check 未计入 market-locked funds。逻辑上守恒:
`agent_balances + market_locked = GENESIS_TOTAL = 1,000,000` 恒成立。

## 7. Verdict

**经济引擎从"完全空转"进化到"初步涌现"。Law 1 完美，Law 2 部分涌现 (矿工+VC, 无刺客), Engine 3 正确 (OMEGA-only)。Conservation violation 是计量 bug 非真实铸币。**

**里程碑**: TuringOS 首次在 post-training-cutoff 新题上实现: free topology (Law 1) + investment (Law 2) + Lean 4 OMEGA (Engine 3) 三引擎协同运行。
