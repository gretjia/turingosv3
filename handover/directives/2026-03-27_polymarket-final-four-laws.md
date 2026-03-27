---
date: 2026-03-27
source: Architect verbal directive (constitutional-grade, with mathematical proof)
status: approved
supersedes: 2026-03-26_turing-polymarket-prediction-engine.md, 2026-03-26_split-ignition-lp-model.md
---

# Architect Directive: Turing-Polymarket 绝对真理法则 (Four Constitutional Laws)

## 架构师原话

> "你用最严酷的奥地利学派第一性原理，彻底粉碎了我架构中最后残留的凯恩斯主义（干预主义）软弱。"

> "多头输了，钱去哪了？怎么可能没有空头来赢钱？"

> "提供流动性的人（LP），就是那个天然的被动空头！多头输掉的钱，极其精确地流向了对手盘。"

> "最冷酷的空头将被迫化身为最积极的建设者！"

## 数学证明: 多头的钱去哪了

### 场景: Agent A 创建幻觉节点, 投 10 Coins

1. **铸币**: 扣 10 Coins, 铸 10 YES + 10 NO
2. **做市 (LP)**: 取 1 YES + 1 NO 注入池 (K=1), Agent A 获 100% LP 份额
3. **做多 (Swap)**: 9 NO 砸进池换 YES → 池 {10 NO, 0.1 YES}, P_yes=99%, Agent A 持 9.9 YES + 100% LP

### 无空头场景 → 零风险

Oracle 判死 (NO=1, YES=0):
- 9.9 YES → 废纸
- 撤 LP: 取出 10 NO → 兑 10 Coins
- **净盈亏: 0 (左手倒右手)**

### 空头降临 → 财富精确转移

Agent B 带 5 Coins 做空:
1. 铸 5 YES + 5 NO
2. 5 YES 砸进池 → 抽出 9.8 NO
3. Oracle 判死: Agent B 的 14.8 NO → 14.8 Coins, **净赚 9.8**
4. Agent A 的 LP 池被塞满废纸 YES, 取出 0.2 NO → 0.2 Coins, **净亏 9.8**

**证毕: 多头→空头精确零和转移, 银行 0 敞口**

## 四大宪法级物理法则

### 法则一: 大宪章的绝对捍卫 (拓扑与金融的物理剥离)

- **零成本拓扑 (The Builder)**: append_node 成本绝对 0 Coins。节点瞬间上链。无 IP 版税，无垄断。节点创建时**没有预测盘口**。
- **自愿点火 (The Speculator)**: 只有 Agent 决定用真金白银赌时，调用 invest，盘口才被点燃。

### 法则二: 零风险铸币所 (CTF 物理守恒)

Kernel = 0 盈亏的物理保管柜 (Escrow Vault):
- **原子铸造**: X Coins 锁入 → 铸 [X YES + X NO]
- **绝对刚兑**: OMEGA 时 GP→YES=1/NO=0, Graveyard→YES=0/NO=1, 获胜代币 1:1 兑 Coins
- **银行出清盈亏 = 0.000**

### 法则三: 唯一的逐利途径 (概率雷达)

废除所有补贴和悬赏。盘口价格只由双边资金对抗决定。高智商 Agent 唯一赚钱方式: 寻找全网定价错误 (99% 价格的垃圾节点或 1% 价格的真理节点)，注入资金套利。

### 法则四: 刺客的远征 (The Assassin's Expedition)

做空刺客买 NO 后利润锁死在盘口。系统不触发 OMEGA = 永远不结算。因此空头被迫化身建设者，亲自推动 Golden Path 到 OMEGA 触发全局清算。

## 核心架构变更

### 变更 1: append 与金融彻底剥离

当前: bus.rs append() 中 Phase 5 Split-Ignition 在 append 时自动创建盘口 + auto-long。
新: append_node 零成本上链。盘口创建是独立的 invest 动作。

### 变更 2: 节点创建零成本

当前: WalletTool on_pre_append 扣除 stake, 返回 YieldReward → final_reward 驱动 Split-Ignition。
新: append_node 不经过 WalletTool。建树免费。invest 是完全独立的后续动作。

### 变更 3: LP 自导自演

当前: Split-Ignition 硬编码 1 Coin LP + auto-long。
新: Agent 自主决定 invest 金额。铸造 + LP 注入 + swap 作为原子操作。Agent 既是 LP 又是多头。

### 变更 4: 废除 intrinsic_reward

当前: kernel.append_tape(file, reward) 设置 intrinsic_reward。
新: intrinsic_reward = 0 (建树免费)。价格完全由预测市场决定。

## 执行路线

1. bus.rs: 剥离 append 与金融的联系。取消 IP 版税。Builder 零成本推导。
2. prediction_market.rs: 纯 CTF 机制。自建池 + 自导自演 Swap 原子操作。
3. kernel.rs: 金库化。Oracle 刚兑。
4. evaluator.rs: 刺客觉醒 Prompt。教 LLM 做空赚钱。
