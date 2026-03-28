# zeta_sum_proof Run 9 — 经济机制审计

**Date**: 2026-03-28
**Engine**: Polymarket vFinal + **强制投资轮 (实验性)**
**Result**: OMEGA (37 tx, 22 nodes, 3-step GP)

---

## 1. 强制投资轮实验数据

| 指标 | Run 8 (无强制) | Run 9 (强制投资轮) | 变化 |
|------|---------------|-------------------|------|
| 节点 | 48 | 22 | -54% |
| FORCED INVEST | 0 | **15** | ∞ |
| 市场创建 | 2 | **7** | +250% |
| BUY YES (跟投) | 0 | **9** | ∞ |
| BUY NO (做空) | 0 | 0 | 无变化 |
| 总投资额 | ~0 | **310 Coins** | ∞ |
| OMEGA 时间 | ~6 min | ~6 min | 无变化 |

**经济活动爆发: 从零到 310 Coins 流通。证明时间不变。**

## 2. 完整经济事件时间线

```
10:20:59  Agent_12 FORCED INVEST 20 YES → tx_4_by_3 [MARKET CREATED] ★
10:21:13  Agent_0  FORCED INVEST 10 YES → tx_1_by_6 [MARKET CREATED]
10:21:17  Agent_9  FORCED INVEST 20 YES → tx_2_by_0 [MARKET CREATED]
10:21:18  Agent_6  FORCED INVEST 20 YES → tx_4_by_3 [BUY YES 跟投] ★
10:21:38  Agent_7  SELF-INVEST  10 YES → tx_13_by_7 [IGNITION]
10:21:45  Agent_3  FORCED INVEST 50 YES → tx_2_by_0 [BUY YES 跟投]
10:22:56  Agent_13 FORCED INVEST 20 YES → tx_6_by_6 [MARKET CREATED]
10:23:20  Agent_3  FORCED INVEST 20 YES → tx_13_by_7 [BUY YES 跟投]
10:23:58  Agent_6  FORCED INVEST 20 YES → tx_13_by_7 [BUY YES 跟投]
10:24:11  Agent_7  FORCED INVEST 10 YES → tx_4_by_3 [BUY YES 跟投] ★
10:24:36  Agent_12 FORCED INVEST 20 YES → tx_1_by_6 [BUY YES 跟投]
10:24:46  Agent_9  FORCED INVEST 20 YES → tx_13_by_7 [BUY YES 跟投]
10:25:43  Agent_3  FORCED INVEST 20 YES → tx_22_by_13 [MARKET CREATED]
10:25:48  Agent_0  FORCED INVEST 10 YES → tx_29_by_7 [MARKET CREATED]
10:26:12  Agent_4  FORCED INVEST 20 YES → tx_13_by_7 [BUY YES 跟投]
10:26:17  Agent_12 FORCED INVEST 20 YES → tx_29_by_7 [BUY YES 跟投]

★ = Golden Path 节点
```

## 3. 市场资金分布

| 市场节点 | GP? | 创建者 | 总投入 | 投资者数 |
|---------|-----|--------|--------|---------|
| **tx_4_by_3** | ★ **YES** | Agent_12 (20) | **50** | 3 (Agent_12, 6, 7) |
| tx_13_by_7 | NO | Agent_7 (10) | **90** | 5 (Agent_7, 3, 6, 9, 4) |
| tx_2_by_0 | NO | Agent_9 (20) | **70** | 2 (Agent_9, 3) |
| tx_1_by_6 | NO | Agent_0 (10) | **30** | 2 (Agent_0, 12) |
| tx_29_by_7 | NO | Agent_0 (10) | **30** | 2 (Agent_0, 12) |
| tx_6_by_6 | NO | Agent_13 (20) | **20** | 1 (Agent_13) |
| tx_22_by_13 | NO | Agent_3 (20) | **20** | 1 (Agent_3) |

**总计**: 310 Coins 投入 7 个市场。

**关键发现: tx_13_by_7 吸引了 90 Coins (最高) 但不在 GP 上！** 这意味着 5 个 Agent 的投资将**全部亏损**。而 tx_4_by_3 (GP Step 1) 只吸引了 50 Coins — 市场**定价错误**, 为做空创造了机会。

## 4. OMEGA 清算详情

### Oracle 裁决
```
tx_4_by_3:   YES (GP) ← 唯一正确
tx_13_by_7:  NO (dead)
tx_29_by_7:  NO (dead)
tx_2_by_0:   NO (dead)
tx_1_by_6:   NO (dead)
tx_6_by_6:   NO (dead)
tx_22_by_13: NO (dead)
```

### LP 提款
| LP 持有者 | 节点 | 提取 YES | 提取 NO | 节点 GP? |
|-----------|------|---------|---------|----------|
| Agent_12 | tx_4_by_3 | 0.02 | 50.00 | ★ YES |
| Agent_7 | tx_13_by_7 | 0.01 | 90.00 | NO |
| Agent_9 | tx_2_by_0 | 0.01 | 70.00 | NO |
| Agent_0 | tx_1_by_6 | 0.03 | 30.00 | NO |
| Agent_0 | tx_29_by_7 | 0.03 | 30.00 | NO |
| Agent_13 | tx_6_by_6 | 0.05 | 20.00 | NO |
| Agent_3 | tx_22_by_13 | 0.05 | 20.00 | NO |

### Redeem (最终兑付)
| Agent | 节点 | 兑付方向 | 金额 | 盈亏 |
|-------|------|---------|------|------|
| Agent_9 | tx_2_by_0 | **NO** | 70.00 | LP 收回 NO 代币 |
| Agent_13 | tx_6_by_6 | **NO** | 20.00 | LP 收回 |
| Agent_0 | tx_29_by_7 | **NO** | 30.00 | LP 收回 |
| Agent_0 | tx_1_by_6 | **NO** | 30.00 | LP 收回 |
| Agent_7 | tx_13_by_7 | **NO** | 90.00 | LP 收回 (死路最大市场) |
| **Agent_7** | **tx_4_by_3** | **YES** | **10.01** | **GP 多头获利** |
| **Agent_6** | **tx_4_by_3** | **YES** | **20.02** | **GP 多头获利** |
| Agent_3 | tx_22_by_13 | **NO** | 20.00 | LP 收回 |
| **Agent_12** | **tx_4_by_3** | **YES** | **19.97** | **GP 创建者获利** |

## 5. Agent 盈亏表

| Agent | 投资总额 | Redeem 总收入 | LP 收回 | 净盈亏 | 角色 |
|-------|---------|-------------|---------|--------|------|
| Agent_12 | 60 (20+20+20) | 19.97 (YES on tx_4★) | 50.00 (LP tx_4★) | **+10** | GP 先驱 ✅ |
| Agent_6 | 40 (20+20) | 20.02 (YES on tx_4★) | — | **-20** | GP 跟投但也投了死路 |
| Agent_7 | 20 (10+10) | 90.00 (NO on tx_13) + 10.01 (YES on tx_4★) | 90.00 | **+80** | **最大赢家** (LP 在死路上反而赚了!) |
| Agent_3 | 90 (50+20+20) | 20.00 (NO on tx_22) | 20.00 | **-50** | 重仓死路 tx_2 亏损 |
| Agent_9 | 40 (20+20) | 70.00 (NO on tx_2) | 70.00 | **+30** | LP 收回 |
| Agent_0 | 30 (10+10+10) | 60.00 (NO on tx_29 + tx_1) | 60.00 | **+30** | LP 收回 |
| Agent_13 | 40 (20+20) | 20.00 (NO on tx_6) | 20.00 | **0** | LP 平本 |
| Agent_4 | 20 | — | — | **-20** | 投了死路 tx_13 |

## 6. 经济学验证

### 大宪章 Law 2 对齐

| 条款 | 状态 | 证据 |
|------|------|------|
| 唯一消耗货币 = 投资 | ✅ | 15 次 FORCED INVEST + 1 次 SELF-INVEST = 全部经济活动 |
| YES/NO 守恒 | ✅ | LP withdraw + Redeem 路径完整 |
| 银行 P&L | ⚠️ | Pre=999,690 Post=1,000,000 **Drift=310** (= 投入额, 计量 bug) |
| 做空刺客 | ❌ | 0 次 SHORT, 仍未涌现 |

**Conservation Drift 310 = 总投资额**: 与 AIME P1 同一计量 bug — `compute_total_system_coins` 只统计 agent balances, 未扣除市场锁定资金。实际守恒: balances + market_locked = 1,000,000 恒成立。

### 角色涌现

| 大宪章角色 | 涌现? | 具体 Agent |
|-----------|-------|-----------|
| **矿工/先驱** | ✅ | Agent_12 (tx_4★ 创建者, GP) |
| **VC 寡头** | ✅ | Agent_3 (50 Coins 重仓 tx_2), Agent_6 (40 Coins 分散投资) |
| **做空刺客** | ❌ | 零 SHORT |

### 意外发现: LP 在死路上的"意外获利"

Agent_7 创建了 tx_13_by_7 (死路) 的市场 → 4 个 Agent 跟投了 90 Coins → OMEGA 后 tx_13 判 NO → Agent_7 作为 LP 提取 90 NO → 兑付 90 Coins。

**Agent_7 花 10 Coins 创建死路市场, 吸引 80 Coins 跟投, 最终通过 LP 提款收回 90 Coins 的 NO 代币 → 净赚 80 Coins!**

这正是架构师预言的"无常损失"机制: LP 在死路市场上反而是赢家 (因为池中积累了大量 NO 代币, 而 NO 在死路上价值 = 1 Coin)。

## 7. Verdict

**强制投资轮成功激活了经济引擎。** 7 个市场, 310 Coins 流通, 跟投/LP/Redeem 全流程运转。做空仍未涌现。Conservation drift 是计量 bug 非真实铸币。

**里程碑**: 首次观察到完整的 LP→跟投→Oracle 裁决→LP 提款→Redeem 经济循环, 且出现了意外的 LP 获利机制 (死路市场 LP 赚取跟投者的本金)。
