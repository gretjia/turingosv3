---
date: 2026-04-04
status: implemented
related_commits: []
---

## 原话
> α=0 意味着子节点只要 price > parent_price 就替代父节点，无深度调节。本项目不需要深度调节，因为按照你的公式，一旦父节点价格高，不就永不退位了吗？这是错误的。

## 浓缩
PRICE_GATE_ALPHA>0 = 高价父节点永不退位 bug

## 架构含义
公式 `child_price > parent_price × (1 + α/depth)` 在 α>0 时：
- 高共识父节点 (price=0.9) 门槛 = 0.945，新子节点 (初始 ~0.5) 永远无法超越
- 前沿被"成功"节点堵死，新探索被阻断
- 这不是 feature 而是 bug：违反了"价格是唯一裁判"原则

Fix: PRICE_GATE_ALPHA = 0，永久锁死。纯 `child_price > parent_price` 比较。

## 行动项
- [x] sweep_v4.py: PRICE_GATE_ALPHA=0, 从可调参数移除
- [ ] actor.rs: 考虑将 default 从 0.05 改为 0
