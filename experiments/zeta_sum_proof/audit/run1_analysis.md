# zeta_sum_proof Run 1 — 完整分析

**运行时间**: 2026-03-22 21:34:23 → 22:09:04 (35 分钟活跃，之后全体破产停滞)
**配置**: Actor Model, N=15, 三物种, MathStepMembrane (纯市场验证)
**结果**: 51 append / 21 rejected / 0 OMEGA / **全体破产停滞**

---

## 一、问题清单 (每个附代码 + tape 证据)

### P0. 致命：全体破产 → 系统永久停滞

**证据 (tape)**:
```
Agent_9:  invest=10000 → Balance: 0.00 (Tx 18, 21:39:07) — 首个破产
Agent_12: invest=10000 → Balance: 0.00 (Tx 31, 21:43:17)
Agent_7:  invest=400   → Balance: 0.00 (Tx 42, 21:46:09)
Agent_13: invest=1     → Balance: 0.00 (Tx 32, 21:43:48)
Agent_10: invest=1     → Balance: 0.00 (Tx 50, 21:49:07)
...最终 9/15 agent 破产，6 个存活但无法产出有效交易
```

最后一笔交易 Tx 72 在 22:09。之后 10+ 小时零活动。

**证据 (代码)** — `evaluator.rs:94-100`:
```rust
if balance < 1.0 {
    if rx.changed().await.is_err() { break; }
    continue;  // 永久等待 — 没有 rebirth
}
```

破产 agent 进入 `rx.changed().await` 等待新 snapshot。但所有 agent 都破产后无人提交交易 → reactor 无 input → 无新 snapshot → 死锁。

**根因**: 无 rebirth 机制。旧架构 (full_test_evaluator.rs) 有 `redistribute_pool()` + rebirth，但新 Actor Model evaluator 没有移植。

### P1. 严重：星形拓扑 — 所有步骤都是 "Step 2"，无深度链

**证据 (tape)**: 21 条 rejected payload 中有 **19 条**是 "Step 2: ..."。51 条 append 也主要是 Step 2 变体。

```
Tx 19: "Step 2: complex summation to closed form"
Tx 25: "Step 2: Using cos(m/N) = Re(e^{im/N})..."
Tx 33: "Step 2: Using Euler's formula..."
Tx 35: "Step 4: Expanding for large N..." ← 唯一跳到 Step 4 的尝试，但破产被拒
```

**证据 (代码)** — `actor.rs:61`:
```rust
let parent_id = best_node.id.clone();
```

`build_chain_from_snapshot` 始终返回最贵节点作为 parent_id。所有 agent 都 cite 同一个 parent → 星形拓扑。没有 Boltzmann 路由的随机性，没有 frontier 选择的多样性。

**根因**: Actor Model 简化了 parent 选择 — 不是选 frontier，而是永远选最贵节点。这导致所有新节点都是最贵节点的直接子节点，不形成深度链。

### P2. 严重：Wallet 余额检查逻辑错误 — Balance: 5990 被当作 "Bankrupt"

**证据 (tape)**:
```
Tx 19: Agent_13 REJECTED: Bankrupt: Insufficient funds. Balance: 5990.00 | Amount: 7500
Tx 27: Agent_14 REJECTED: Bankrupt: Insufficient funds. Balance: 4000.00 | Amount: (>4000)
Tx 35: Agent_2  REJECTED: Bankrupt: Insufficient funds. Balance: 1000.00 | Amount: (>1000)
Tx 61: Agent_8  REJECTED: Bankrupt: Insufficient funds. Balance: 1500.00 | Amount: (>1500)
```

这不是 "破产" — 是 **投资金额超过余额**。但日志都显示 "Bankrupt"，造成误导。

**证据 (代码)** — `wallet.rs:84-87`:
```rust
let balance = *self.balances.get(author).unwrap_or(&0.0);
if balance < amount {
    return ToolSignal::Veto(format!("Bankrupt: Insufficient funds. Balance: {:.2}", balance));
}
```

错误信息说 "Bankrupt" 但实际是 "insufficient for this specific bet"。Agent_13 有 5990 coins，不是破产 — 只是这次赌注太大。

**影响**: 这不是 bug（逻辑正确），但日志误导分析。真正的破产是 Balance: 0.00 的那些。

### P3. 中等：无 Hayekian 反向传播 — 价格信号断裂

**证据 (tape)**: 只有 3 次 Phase Transition:
```
21:35:08 PHASE TRANSITION: 500 >= 1.50
21:35:18 PHASE TRANSITION: 2000 >= 750
21:37:17 PHASE TRANSITION: 9000 >= 3000
```

之后 48 次 append 再无 Phase Transition → map-reduce 从未再运行 → 价格停留在 intrinsic_reward（投资金额），无反向传播。

**证据 (代码)** — `bus.rs tick_map_reduce()`:
```rust
if current_max_price >= self.last_max_price * self.threshold_ratio {
    // threshold_ratio = 1.5
    // 9000 × 1.5 = 13500, 但没有新节点投资 > 13500 → 永不触发
}
```

**根因**: OverwhelmingGapArbitrator 在价格达到 9000 后，需要 13500 才触发下一次 reduce。但没有 agent 投资 > 13500 → 价格信号冻结。

### P4. 中等：Agent_9 试图直接跳到结论但被破产拒绝

**证据 (tape)**:
```
Tx 28: Agent_9 REJECTED: Balance: 0.00 | "For each fixed N, S(N) converges
absolutely. As N→∞, the limit of S(N) equals ζ(-1) = -1/12, proving..."
```

Agent_9 (R1-Distill-32B) 第一步就 invest 10000（梭哈），第二步试图直接写出结论 — 但已破产。

**洞察**: 如果 Agent_9 没有梭哈，它可能用一步直接跳到答案。这说明**定价行为是关键瓶颈** — 不是推理能力。

### P5. 低：R1-Distill-32B 几乎无贡献

**证据 (tape)**:
```
模型贡献: deepseek-reasoner=37(74%), R1=10(20%), R1-Distill-32B=3(6%)
```

R1-Distill-32B (Agent_0, 3, 6, 9, 12) 占 5/15 的席位但只贡献 3/51 的 append。Agent_0 和 Agent_6 **零 append**（全程 observe 或 view）。

### P6. 低：Private Context 未被充分利用

**证据 (tape)**:
```
SEARCH: 2 次 (Agent_7, Agent_10)
VIEW: 12 次
OBSERVE: 4 次
```

搜索和观察的总量（18 次）vs 投资（72 次）= 20%。大多数 agent 直接投资而不先研究。大宪章 Law 1 的"谋定而后动"模式没有充分涌现。

---

## 二、对齐检查

### Layer 1 四大不变量
| 不变量 | 状态 | 证据 |
|--------|------|------|
| kernel.rs 零领域知识 | ✅ | kernel 未触及 |
| SKILL-only reward | ✅ | YieldReward 仅通过 WalletTool |
| Append-Only DAG | ✅ | 51 节点只增不删 |
| 投资 >= 1.0 | ✅ | 2 次 Market Violation 被正确 VETO |

### 反奥利奥三界
| 界 | 状态 | 证据 |
|----|------|------|
| 顶层白盒 | ✅ | reactor 串行处理，Wallet 正确扣款 |
| 中间黑盒 | ✅ | agent 自由选择 invest/search/view/observe |
| 底层白盒 | ✅ | Tape 正确记录 DAG |

### 大宪章 + 新机制
| 机制 | 状态 | 证据 |
|------|------|------|
| Actor Model (无锁) | ✅ **运行正确** | 35 min 内 72 tx，无阻塞 |
| MathStepMembrane | ✅ | 格式检查通过，无误杀 |
| Order Book | ⚠️ **未验证** | 日志中无 Order Book 输出（可能 agent 忽略了） |
| 链感知 prompt | ⚠️ **有效但浅** | agents 看到链但都写 Step 2（P1 问题） |
| 私有上下文 | ✅ | 12 ViewNode + 2 Search 正确注入 |
| 无认知溢价 | ✅ | 系统不补贴思考时间 |
| Rebirth | ❌ **缺失** | 全体破产后系统死锁 (P0 问题) |

### 无锁 Actor Model 架构师指令
| 指令 | 状态 | 证据 |
|------|------|------|
| watch/mpsc 消息传递 | ✅ | 72 tx 无锁串行处理 |
| 慢快模型共存 | ✅ | reasoner(慢) + R1-Distill(快) 同时运行 |
| 事件驱动快照广播 | ✅ | 每次 append 后广播 |

---

## 三、修复优先级

| 优先级 | 问题 | 修复 | 代码位置 |
|--------|------|------|---------|
| **P0** | 全体破产死锁 | **Rebirth**: reactor 每 N 个 tx 检查全体余额，破产 agent 获得新资金 | `evaluator.rs` reactor loop |
| **P1** | 星形拓扑 | **Frontier 选择**: `build_chain_from_snapshot` 应该概率性选择不同 frontier 节点 | `actor.rs:40-61` |
| **P3** | 价格信号冻结 | **降低 Arbitrator 阈值** 或 **定期强制 reduce** | evaluator 或 bus 配置 |
