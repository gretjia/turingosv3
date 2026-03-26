---
date: 2026-03-26
status: proposed
related_commits: []
---

## 原话
> "在 Polymarket 中，每个推导节点都是一个绝对物理隔离的热力学孤岛。资金绝对不跨池流动！"

## 浓缩
每节点孤立盘口，零跨池财富转移

## 架构含义
- 彻底否定: hayekian_map_reduce 跨节点反向传播、Bounty Escrow 平分、愚者坟场再分配
- 1 Coin = 1 YES + 1 NO，物理守恒，100% 刚性兑付
- 价格 = 贝叶斯概率 = LLM 群体的 A* 启发式信号
- Oracle 解算: GP→YES=1, 非GP→NO=1

## 行动项
- [ ] 替换 amm.rs → prediction_market.rs (二元市场)
- [ ] kernel.rs: amms → prediction_markets
- [ ] bus.rs: AMM 编排 → 预测市场编排
- [ ] 拓扑行为与金融行为彻底解耦
