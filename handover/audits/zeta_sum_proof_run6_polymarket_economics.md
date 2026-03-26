# Run 6 Polymarket 经济学深度分析

**Date**: 2026-03-26
**Engine**: Turing-Polymarket (binary CPMM + Split-Ignition)
**Result**: OMEGA (53 tx, 35 nodes, 1 generation, 6-step GP)

---

## 一、Agent 投资行为全景

### 交易统计

| Agent | 节点数 | 总投资 | 投资模式 | GP 贡献 | REDEEM |
|-------|--------|--------|---------|---------|--------|
| Agent_0 | 5 | 50 | 10×5 | **Step 4+5** | 19.80 |
| Agent_1 | 5 | 132 | 20,10,100,1,1 | **Step 6 (OMEGA)** | **100,000,000,001 (BUG)** |
| Agent_3 | 8 | 72 | 10×6,1×2 | **Step 2** | 9.90 |
| Agent_4 | 4 | 71 | 10,50,10,1 | 0 | 0 |
| Agent_6 | 4 | 121 | 10,1,10,100 | 0 | 0 |
| Agent_7 | 3 | 21 | 10,10,1 | **Step 1** | 9.90 |
| Agent_9 | 5 | 41 | 10×4,1 | **Step 3** | 9.90 |
| Agent_10 | 1 | 10 | 10 | 0 | 0 |
| Agent_12 | 6 | 51 | 10×5,1 | 0 | 0 |
| Agent_13 | 2 | 20 | 10×2 | 0 | 0 |

### 关键行为模式

- **100% SELF-INVEST**: 53 笔交易全部 "Invest in self"。零跨节点投资。
- **保守 Kelly**: 66% 投注 10 Coins (余额 0.1%)，17% 投注 1 Coin。SKILL prompt Kelly 指令完美涌现。
- **零做空**: 没有任何 Agent 买入 NO 份额。

---

## 二、OMEGA 铸币 Bug (P0)

**根因**: `experiments/zeta_sum_proof/src/math_membrane.rs:40`

```rust
return ToolSignal::YieldReward {
    payload: format!("{}\n  -- [OMEGA]", payload),
    reward: 100_000_000_000.0,  // ← Hayekian 遗产，破坏 Polymarket 零和
};
```

**影响链**:
1. Agent_1 投注 1 Coin → MathStepMembrane 将 reward 改为 100B
2. Split-Ignition: LP=1, auto-long=99,999,999,999 → Agent_1 获 ~100B YES
3. OMEGA 解算: 100B YES × 1 Coin = 100B Coins 凭空铸造
4. Step 1 先驱 (Agent_7) 赎回 9.90 vs OMEGA 终局者 100B → 风险倒挂极端恶化

**修复**: COMPLETE 不铸币，只标记 OMEGA tag。

---

## 三、为什么没有做空行为？—— 五层因分析

### 层 1: 协议层缺失 (Protocol Gap)
SKILL prompt 只暴露 invest/search/view 三种动作。没有 `short` 选项。Agent 即使想做空也无法调用。

### 层 2: 信息不对称 (Information Asymmetry)
所有节点 Market Cap 都显示 1.00 (yes_price≈1.0，auto-long 后)。Agent 看不到 YES/NO 储备量、赔率或做空成本。没有差异化套利信号。

### 层 3: 激励结构偏移 (Incentive Misalignment)
每个节点 LP 仅 1 Coin → 做空池深度极浅。空头买 10 Coins NO，对手盘只有 ~1 Coin 流动性。ROI 上限 ≈ 10%。做空和做多的 ROI 都接近零。

### 层 4: LLM 认知偏差 (Cognitive Bias)
LLM 天然是乐观生成者，不是怀疑审计者。做空需要攻击性行为模式，非 LLM 自然倾向。

### 层 5: 100B 铸币扭曲
OMEGA 铸币使唯一有意义的利润来自"最后一步写 COMPLETE"，不是来自做空审计。Agent 自然倾向于竞争终局而非做空他人。

---

## 四、Polymarket 零和悖论

**核心矛盾**: Polymarket 零和 + 保守投注 = 无利可图

在纯零和系统中，赢家的利润 = 输家的本金。但如果:
- 所有 Agent 保守投注 10 Coins
- 每个节点 LP 只有 1 Coin seed
- GP 节点赎回 ~10 Coins (仅保本)
- 非 GP 节点损失 10 Coins

则先驱者 ROI ≈ 0%。无风险溢价自发涌现。

**根因**: 没有跨节点投资 = 没有对手盘 = 没有价格发现 = 没有利润。

---

## 五、DAG 拓扑

```
ROOT
├── tx_1_by_12 ── tx_2_by_3 ── tx_5_by_3 ── tx_16_by_3 ── tx_32_by_3
│             └── tx_3_by_1 ── tx_8_by_9 / tx_9_by_6
│             └── tx_6_by_1
│
├── tx_4_by_7 ★ ── tx_7_by_3 ★ ── tx_17_by_9 ★ ── tx_35_by_0 ★ ── tx_50_by_0 ★ ── tx_53_by_1 ★ [OMEGA]
│              └── tx_11_by_1 ── tx_13_by_9 / tx_23_by_6 ── tx_30_by_9
│                            └── tx_25_by_12 ── tx_31 ── tx_34 ── tx_39/43/44
│
├── tx_10_by_13 ── tx_15_by_4 ── tx_28_by_0 / tx_20_by_4
├── tx_24_by_7 / tx_27_by_3 / tx_45_by_9 / tx_46_by_4    (孤岛)

★ = Golden Path (6/35 = 17%)
```

---

## 六、对齐审查 (Alignment Check)

### Layer 1 四大不变量

| 不变量 | 状态 | 证据 |
|--------|------|------|
| kernel.rs 零领域知识 | ✅ **PASS** | prediction_market.rs 纯二元 CPMM 数学，kernel 方法全部 domain-free |
| SKILL-only 铸造 intrinsic_reward | ⚠️ **PASS (但 SKILL 层 bug)** | intrinsic_reward 仍由 WalletTool (SKILL) 通过 YieldReward 设置。kernel 不自行铸造。但 MathStepMembrane (另一个 SKILL) 注入 100B 破坏了系统语义 — 这是 SKILL 层 bug，非 Layer 1 违规 |
| Tape Append-Only DAG | ✅ **PASS** | 35 节点只增不删。无 .remove/.delete/.clear 调用 |
| Stake >= 1.0 | ✅ **PASS** | WalletTool:100 检查 amount < 1.0 → Veto。最低投注 1 Coin 出现 9 次 |

### Polymarket 四条铁律

| 铁律 | 状态 | 证据 |
|------|------|------|
| **铁律 1**: 1 Coin = 1 YES + 1 NO (物理守恒) | ❌ **VIOLATED** | MathStepMembrane 的 100B YieldReward 凭空创造了 ~100B YES 份额。OMEGA 节点 Split-Ignition 时 LP=1 + auto-long=99,999,999,999 → 铸造了远超物理守恒的代币量。Agent_1 赎回 100,000,000,001 Coins，系统不可能有这么多对手盘资金 |
| **铁律 2**: 拓扑免费，金融自理 | ✅ **PASS** | 零引用费。53 笔交易中引用 (citation) 完全免费。金融行为 (invest) 独立于 DAG 连线。大宪章 Law 1 完美复辟 |
| **铁律 3**: 价格即贝叶斯概率 | ⚠️ **DEGRADED** | P_yes + P_no = 1 数学恒等式成立 ✓。但所有 35 个节点 P_yes ≈ 1.0 (99%)，因为每个创建者都 auto-long 到极限且零跨节点投资。**价格退化为单一常数，失去了贝叶斯概率的信息区分功能** |
| **铁律 4**: Oracle 二元审判 | ✅ **PASS** | 6 GP 节点 → YES wins, 29 dead 节点 → NO wins。resolve() 正确执行，redeem() 正确兑付。唯一异常是 100B 铸币导致兑付金额超出零和预期 |

### Anti-Oreo 三界检查

| 界 | 状态 | 证据 |
|----|------|------|
| **顶层白盒** (规则 + Oracle) | ⚠️ **BUG** | 预测市场规则正确运行。但 MathStepMembrane (顶层 SKILL) 的 100B 铸币等于顶层白盒主动干预了经济，违反了"系统绝不凭空下场提供流动性"的原则 |
| **中间黑盒** (Agent 博弈) | ✅ **PASS** | 10 个 Agent 自主博弈。投注金额、节点选择、推导方向全由黑盒自主决定。Kelly Criterion 自发涌现。唯一缺陷：黑盒未发现做空和跟投两种策略 |
| **底层白盒** (DAG + 数学) | ✅ **PASS** | Append-Only DAG 完整。BinaryMarket CPMM 数学正确 (10/10 单元测试通过)。refresh_prices 从 yes_price() 读取。kernel 零干预 |

### 大宪章 (Magna Carta) 三律

| 律 | 状态 | 证据 |
|----|------|------|
| **Law 1**: 信息自由 | ✅ **PASS** | Search (免费) 使用 2 次，View (免费) 使用 ~80 次。Agent 充分利用免费信息权。拓扑引用零成本 |
| **Law 2**: 唯投资有风险 | ⚠️ **PARTIAL** | 投资确实是唯一扣款行为 ✓。但 100B 铸币使 OMEGA 创建者的"风险"变为负数（投 1 Coin 得 100B），彻底破坏了风险-收益正相关的基本原则 |
| **Law 3**: 数字产权 | ✅ **PASS** | Agent 持有 YES 份额 = 数字产权。OMEGA 解算后正确兑付。持仓在 portfolio 中有记录 |

### Bible.md 哲学对齐

| 原则 | 状态 | 证据 |
|------|------|------|
| **苦涩的教训**: 内核零领域知识 | ✅ **PASS** | kernel.rs + prediction_market.rs 纯数学。零 "lean"/"tactic"/"theorem" 字符串 |
| **机制与策略分离**: Kernel = 拓扑, SKILL = 策略 | ✅ **PASS** | 预测市场是纯机制 (x*y=k)。MathStepMembrane 是策略层 SKILL (可替换)。两者物理隔离 |
| **Popperian 证伪**: Lean 4 是绝对仲裁者 | ❌ **NOT IMPLEMENTED** | Run 6 OMEGA 仍由字符串匹配 `[COMPLETE]` 触发。无 Lean 4 编译器验证。Terminal Oracle 是 P0 优先级但尚未实现 |
| **Austrian Economics**: 价格发现替代中央调度 | ⚠️ **DEGRADED** | 理论上 Polymarket 价格 = 贝叶斯概率。实际上所有节点 P_yes≈1.0，价格发现退化。需要跨节点投资 + 做空来激活真正的价格发现 |

### 架构师博弈论预言 vs 实际涌现

| 架构师预言 | 实际涌现 | 差距原因 |
|-----------|---------|---------|
| "先驱者 ROI 2000%" | Step 1 先驱 ROI ≈ -1% (投 10 赎 9.90) | 零跨节点投资 → 无对手盘 → 无利润。100B bug 使利润全部集中在 OMEGA 终局者 |
| "做空刺客局部狂欢" | 零做空行为 | 协议未暴露 short 动作 + 信息不可见 + LLM 乐观偏差 |
| "价格即 A* 雷达" | 全部 P_yes=1.0，零信息量 | Split-Ignition auto-long 到 99% + 无人做空 = 价格锁死在创建者的初始偏见 |
| "1 Coin = 1 YES + 1 NO 守恒" | OMEGA 节点 100B 铸币破坏守恒 | MathStepMembrane 旧代码遗产 |
| "绝对冷酷的热力学孤岛" | 孤岛结构正确运行 | ✅ 唯一完美对齐的预言。零跨池流动，资金隔离验证通过 |

### 对齐总评

```
=== RUN 6 ALIGNMENT VERDICT ===

Layer 1 Invariants:     4/4 PASS
Polymarket 4 Iron Laws: 2/4 PASS, 1 VIOLATED (conservation), 1 DEGRADED (Bayesian price)
Anti-Oreo 3 Boundaries: 2/3 PASS, 1 BUG (top-layer 100B mint)
Magna Carta 3 Laws:     2/3 PASS, 1 PARTIAL (Law 2 risk-reward inverted)
Bible.md Philosophy:    2/4 PASS, 1 NOT IMPLEMENTED (Lean 4), 1 DEGRADED (price discovery)

Overall: PARTIALLY ALIGNED — kernel clean, economic mechanism correct,
but SKILL-layer 100B bug + missing short action + price signal collapse
prevent Polymarket from delivering its theoretical promise.

Root Cause: MathStepMembrane 100B legacy + protocol-level missing actions
Fix Path: P0 (100B fix) → P1 (short + price visibility) → P2 (cross-invest)
```

---

## 七、修复优先级

| Priority | Issue | Fix |
|----------|-------|-----|
| **P0** | MathStepMembrane 100B OMEGA 铸币 | 删除 YieldReward，改为 ToolSignal::Modify (零铸币) |
| **P1** | 无 short 动作 | SKILL prompt 暴露 `{"tool":"short","node":"X","amount":Y}` |
| **P1** | 价格信号退化 (全部 P_yes≈1) | Snapshot market_ticker 显示 P_yes/P_no 概率 |
| **P2** | 零跨节点投资 | SKILL prompt 引导 "invest in promising nodes you didn't create" |
| **P2** | LP 深度过浅 (1 Coin) | 提高 LP seed 或引入外部 LP 注入 |
| **P3** | 零和低利润悖论 | 架构师需思考激励设计 — 可能需要外部 bounty 或非零和组件 |
