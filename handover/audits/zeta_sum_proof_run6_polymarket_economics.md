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

## 六、修复优先级

| Priority | Issue | Fix |
|----------|-------|-----|
| **P0** | MathStepMembrane 100B OMEGA 铸币 | 删除 YieldReward，改为 ToolSignal::Modify (零铸币) |
| **P1** | 无 short 动作 | SKILL prompt 暴露 `{"tool":"short","node":"X","amount":Y}` |
| **P1** | 价格信号退化 (全部 P_yes≈1) | Snapshot market_ticker 显示 P_yes/P_no 概率 |
| **P2** | 零跨节点投资 | SKILL prompt 引导 "invest in promising nodes you didn't create" |
| **P2** | LP 深度过浅 (1 Coin) | 提高 LP seed 或引入外部 LP 注入 |
| **P3** | 零和低利润悖论 | 架构师需思考激励设计 — 可能需要外部 bounty 或非零和组件 |
