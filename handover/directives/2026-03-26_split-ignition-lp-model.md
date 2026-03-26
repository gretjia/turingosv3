---
date: 2026-03-26
source: Architect review of Turing-Polymarket Plan & Spec
status: approved
supplements: 2026-03-26_turing-polymarket-prediction-engine.md
---

# Architect Supplement: Split-Ignition LP Model

## 架构师原话

> "创立一个数学节点的 Agent，绝不能是中立的做市商，他必须是自己定理的'绝对死忠多头（Permabull）'！"

> "系统强制划扣极少量的一笔固定本金（例如 1 Coin），铸造 1 YES + 1 NO 注入底层 AMM 池。此时 K=1，初始赔率为 1:1。然后 Agent 剩下的 Stake，系统直接代表该 Agent 发起市价 buy_yes！"

## Split-Ignition 两步点火

Agent 投入 Stake = 50 Coins 时：

### Step 1: 中性点火 (Protocol LP Ignition)
- 划扣固定本金 (1 Coin)
- 铸造 1 YES + 1 NO 注入池
- K = 1, P_yes = 50% (中立先验)
- 防止除零异常

### Step 2: 杠杆狂飙 (Directional Auto-Long)
- 剩余 Stake (49 Coins) 全部执行 buy_yes()
- 铸造 49 YES + 49 NO → 保留 YES → 卖 NO 给池子换更多 YES
- P_yes 瞬间从 50% 飙升到 ~99%
- 创建者 Portfolio 塞满 YES (切肤之痛)

### 涌现效果
- 全网看到作者极度自信 → 空头猎犬嗅到猎物
- 如果是幻觉节点 → 空头 buy_no 获得极高赔率
- 如果是真理 → 创建者暴利 (风险溢价自发涌现)

## 关键数学约束
- 无常损失规避: 创建者不做 LP，而是做单边多头
- K 守恒: 所有操作维持 X × Y = K
- 物理守恒: 1 Coin = 1 YES + 1 NO 始终成立
