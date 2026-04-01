---
date: 2026-04-01
status: proposed
related_commits: []
---

## 原话
> 经济活动太少，重复节点太多。处方：角色分化 — 15 agents 分 5数学/5做多/5做空，用角色对抗驱动经济活动与探索多样性。

## 浓缩
角色分化5/5/5对抗驱动经济活动与探索多样性

## 架构含义
- Layer 1 无影响 — 角色分化不触碰永恒不变量，仍遵循 Law 1 免费建树 + Law 2 投资消耗
- Layer 2 影响 — 15 agents 的 SKILL prompt 需按角色分化：5 数学探索者 / 5 做多者 / 5 做空者
- 延续 2026-03-31 zero-shorting 洞察的升级版：从"prompt 引导做空"升级为"结构性角色对抗"
- 做多/做空对抗 = Engine 2 市场效率的结构性保障，不再依赖 LLM 自发批判
- 数学角色专注 Engine 1 拓扑探索，交易角色专注 Engine 2 定价，职责分离更彻底
- Anti-Zombie 阈值 (Layer 2 Rule 8) 配合角色分化可更有效：重复节点问题从源头缓解

## 行动项
- [ ] 设计三类角色的 SKILL prompt 模板：mathematician / bull / bear
- [ ] bus.rs 或 SKILL 层实现角色分配机制（on_init 时固定分配或轮换）
- [ ] 做空角色 prompt 需包含具体盈利场景（继承 zero-shorting 洞察）
- [ ] 评估是否需要动态角色轮换（如每 N 轮重新分配）防止角色固化
