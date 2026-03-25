# zeta_sum_proof Run 5 (TuringSwap) vs Run 4 (Hayekian) — 对比分析

**Date**: 2026-03-25
**Subject**: Run 5 — 首次 TuringSwap AMM 经济引擎实战测试
**Result**: OMEGA achieved (15 tx, 12 nodes, 2 generations, 3-step Golden Path)

---

## 一、运行概况

| 维度 | Run 4 (Hayekian) | Run 5 (TuringSwap) | 变化 |
|------|------------------|---------------------|------|
| 经济引擎 | hayekian_map_reduce | AMM 恒定乘积 x*y=k | **范式切换** |
| 总交易 | 37 tx | 15 tx | -59% |
| 上链节点 | 26 | 12 | -54% |
| 被拒节点 | 11 | 3 | -73% |
| Golden Path 步数 | 6 | 3 | -50% (更压缩) |
| 世代 (rebirth) | 1 (零 rebirth) | 2 (1 次 rebirth) | +1 |
| 活跃 Agent 数 | 14/15 | 10/15 (5 个 R1 API 失效) | -33% |
| 到达 OMEGA | ~12 min | ~8 min | -33% |
| Bounty Escrow | N/A (无锚印钞) | 100,000 Coins | **新增** |

---

## 二、Golden Path 对比

### Run 4 (6 步, Hayekian)

| Step | Node | Agent | Model | Price | 内容 |
|------|------|-------|-------|-------|------|
| 1 | tx_2_by_12 | Agent_12 | V3.2 | 95,099,006,159 | 定义 x=1/N, 复指数改写 |
| 2 | tx_5_by_2 | Agent_2 | R1 | 96,059,601,980 | 几何级数闭式求和 |
| 3 | tx_14_by_7 | Agent_7 | Reasoner | 97,029,900,689 | Taylor 展开 |
| 4 | tx_23_by_0 | Agent_0 | V3.2 | 98,010,000,495 | 平方展开 |
| 5 | tx_30_by_9 | Agent_9 | V3.2 | 99,000,000,199 | 多项式长除法 |
| 6 | tx_37_by_3 | Agent_3 | V3.2 | 100,000,000,100 | 取实部 → -1/12 [OMEGA] |

### Run 5 (3 步, TuringSwap)

| Step | Node | Agent | Model | Price | 内容 |
|------|------|-------|-------|-------|------|
| 1 | tx_1_by_9 | Agent_9 | V3.2 | 29,378 | 定义 S(N) + 绝对收敛 |
| 2 | tx_7_by_6 | Agent_6 | V3.2 | 29,378 | cos→Re + 闭式 Re(z/(1-z)²) |
| 3 | tx_15_by_10 | Agent_10 | Reasoner | 90,000,030,045 | Taylor + 平方 + 长除法 + 取实部 → -1/12 [OMEGA] |

**关键发现**: Agent_10 (Reasoner) 在 Step 3 一步完成了 Run 4 的 Step 3-6 四步工作。

---

## 三、AMM 经济行为分析

### 价格信号对比

| 信号类型 | Run 4 (Hayekian) | Run 5 (TuringSwap) |
|---------|-------------------|---------------------|
| GP 最高价 | 100,000,000,100 (OMEGA 铸币) | 90,000,030,045 (Bounty 注入) |
| GP 最低价 | 95,099,006,159 | 29,378 |
| 非 GP 最高价 | 494 | 51 |
| 价格断崖 | 1.9 亿倍 | 576 倍 |
| 定价来源 | gamma^depth 反向传播 | AMM 池 coin_reserve |

### AMM 事件时间线

```
16:55:35  [IPO] Agent_9  → tx_1_by_9  (IDO: 50, Founder: 1000 tokens)
16:57:xx  [CITATION BUY] Agent_4 bought 100 tokens of tx_1_by_9 for 0.56   ← 首次引用购买
16:57:xx  [CITATION BUY] Agent_6 bought 100 tokens of tx_1_by_9 for 0.57   ← 第二次引用，滑点涨价
17:02:52  [OMEGA SETTLEMENT] Bounty injected into 3 GP pools
17:02:52  [CASH OUT] agents sell GP tokens for Coins
```

### 全部节点

| 节点 | Agent | 模型 | 价格 | 引用 | GP | 类型 |
|------|-------|------|------|------|-----|------|
| tx_1_by_9 | Agent_9 | V3.2 | 29,378 | [] | **Step 1** | 先驱 |
| tx_2_by_4 | Agent_4 | Reasoner | 10 | [] | | 竞争 Step 1 |
| tx_3_by_6 | Agent_6 | V3.2 | 10 | [] | | 竞争 Step 1 |
| tx_4_by_13 | Agent_13 | Reasoner | 10 | [] | | 竞争 Step 1 |
| tx_5_by_7 | Agent_7 | Reasoner | 10 | [] | | 竞争 Step 1 |
| tx_6_by_4 | Agent_4 | Reasoner | 51 | [tx_1] | | Step 2 (简短) |
| tx_7_by_6 | Agent_6 | V3.2 | 29,378 | [tx_1] | **Step 2** | 闭式求和 |
| tx_10_by_0 | Agent_0 | V3.2 | 51 | [tx_6] | | 跟随 tx_6 死路 |
| tx_12_by_0 | Agent_0 | V3.2 | 50 | [tx_10] | | 跟随链 |
| tx_13_by_1 | Agent_1 | Reasoner | 10 | [tx_6] | | 跟随 tx_6 死路 |
| tx_14_by_9 | Agent_9 | V3.2 | 50 | [tx_7] | | Step 3 不完整 |
| tx_15_by_10 | Agent_10 | Reasoner | 90B | [tx_7] | **Step 3 OMEGA** | 终局一击 |

### DAG 拓扑

```
ROOT
├── tx_1_by_9 ★ (29K) ── tx_6_by_4 (51) ── tx_10_by_0 (51) ── tx_12_by_0 (50)
│                    │── tx_13_by_1 (10)
│                    └── tx_7_by_6 ★ (29K) ── tx_14_by_9 (50)
│                                          └── tx_15_by_10 ★ (90B) [OMEGA]
├── tx_2_by_4 (10)    ← 孤岛
├── tx_3_by_6 (10)    ← 孤岛
├── tx_4_by_13 (10)   ← 孤岛
└── tx_5_by_7 (10)    ← 孤岛

★ = Golden Path
```

---

## 四、关键对比发现

### 1. AMM 驱动了更高效的证明

Run 4 用 6 步完成证明（每步一个算术操作），Run 5 用 3 步完成（Step 3 一步做了 4 步的工作）。引用成本机制激励 agent 产出高信息密度节点而非拆分成低价值小步骤。

### 2. 引用成本抑制了寄生行为

Run 4 有大量寄生节点 (tx_11: 三个字 "geometric series summation")。Run 5 中所有非 GP 节点至少有实质内容 — 因为引用需要付费 (0.56 Coins)，不存在零成本挂靠。

### 3. 废物自动归零生效

4 个孤岛 Step 1 节点 (tx_2/3/4/5) 价格均为 10 (仅 IDO 初始资金)。无人引用 = 投资打水漂。架构师预言成真。

### 4. Bounty Escrow 替代无锚印钞

Run 4: OMEGA 节点获得 100,000,000,100 (凭空铸造)。
Run 5: OMEGA 节点获得 90,000,030,045 (来自 100,000 Bounty Escrow 注入后 Agent 套现)。

### 5. Agent 投资行为改变

Run 4 投资额范围: 50 — 100,000,000,100 (巨幅差异)。
Run 5 投资额: 统一 50 Coins (IDO 标准化)。Kelly Criterion 在 AMM 环境下表现为低额频繁而非高额豪赌。

### 6. 引用滑点验证

tx_1_by_9 被引用 2 次，引用成本从 0.56 → 0.57 Coins (AMM 滑点驱动价格上升)。这正是恒定乘积定价的预期行为。

---

## Metadata

- **Run**: zeta_sum_proof Run 5 (TuringSwap AMM, 全新 tape)
- **Configuration**: Actor Model, N=15 (10 active), Bounty=100,000
- **Models**: DeepSeek V3.2 (deepseek-chat) + Reasoner (deepseek-reasoner), 5x R1 via SiliconFlow 失效
- **Transactions**: 15 (12 appended, 3 rejected)
- **Generations**: 2 (1 rebirth)
- **Duration**: ~8 minutes
- **Golden Path Contributors**: Agent_9 (V3.2), Agent_6 (V3.2), Agent_10 (Reasoner)
