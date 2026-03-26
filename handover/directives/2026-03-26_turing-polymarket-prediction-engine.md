---
date: 2026-03-26
source: Architect verbal directive (extended mechanism design)
status: archived-pending-authorization
supersedes: 2026-03-25_turingswap-amm-economic-engine.md
---

# Architect Directive: Turing-Polymarket 二元真理预测引擎

## 架构师原话

> "系统不再是一个'发奖金的裁判'，而是变成了一个绝对冷酷、机制中立的'股份制清算所（Clearing House）'"

> "每个 Agent 的出清 = 持有每个节点的股份 × 相应股价"

> "改为 Polymarket 机制。"

> "在 Polymarket 中，每个推导节点都是一个绝对物理隔离的热力学孤岛。资金绝对不跨池流动！"

> "真理的价值，不应由'全网有多少傻瓜犯错'来决定，而应严格由'该节点消除的不确定性（熵减）'来决定。"

## 对前两版的终极批判

### TuringSwap AMM (第二版) 之死
- AMM 的引用即买入 = 拓扑行为与金融行为耦合
- 仍有 Bounty Escrow 注入 = 系统印钱
- OMEGA 平分 bounty = GOSPLAN 变体

### 愚者坟场平分 (第三版) 之死
- 跨池财富转移 = 打土豪分田地的中心化再分配
- 违背信息论：真理价值不应由傻瓜数量决定

## Turing-Polymarket 四条铁律

### 铁律 1: 绝对物理守恒 (1 Coin = 1 YES + 1 NO)
系统不没收资金，不印发法币。Agent 向协议发送 1 Coin，原子化铸造 [1 YES + 1 NO]。100% 刚性兑付能力。

### 铁律 2: 大宪章的绝对捍卫（拓扑免费，金融自理）
- 认知层：连线、引用、构建 DAG 图谱 = 0 成本
- 金融层：想赚钱必须到该节点独立盘口用真金白银买 YES 或 NO
- 拓扑结构与金融杠杆彻底解耦

### 铁律 3: 价格即贝叶斯概率
P_yes + P_no = 1。盘口价格 = 全网用真金白银投票的概率共识。
LLM 本质是概率分布输出 → 盘口价格 = 全局 A* 启发式导航雷达。

### 铁律 4: Lean 4 断头台的终极审判
OMEGA 时 Oracle 清算:
- GP 节点: YES → 1 Coin, NO → 0
- 非 GP 节点: NO → 1 Coin, YES → 0

## 博弈推演

### 先驱者暴利 (风险溢价自发涌现)
迷雾期 YES 价格 = 0.05 → Agent 用 50 Coins 买 1000 YES → OMEGA 后兑付 1000 Coins → ROI 2000%

### 做空刺客 (系统免疫网络)
Agent B 把幻觉节点 YES 买到 0.8 → Agent C 看穿漏洞买 NO at 0.2 → 幻觉破灭 NO=1 → Agent B 的钱精确流入 Agent C

### 流动性点火
节点创建者强制支付极小额度 (如 1 Coin) 作为创世 LP，注入等额 YES 和 NO 到双向流动池。

## 与 Anti-Oreo 原则的绝对对齐
- 顶层白盒: 预测市场规则 + Oracle 仲裁
- 中间黑盒: Agent 自主博弈 (买 YES/NO)
- 底层白盒: DAG 拓扑 + 二元清算数学
- 零人为干预、零跨池流动、零系统印钱
