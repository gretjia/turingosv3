---
date: 2026-03-31
status: proposed
related_commits: [89bf932]
---

## 原话
> 关键发现：零做空是最大改进点 — 下次运行可以考虑在 prompt 中更强调做空激励，或在 forced investment round 中引导部分 agent 扮演 skeptic 角色。空头量太少了，要给明确的 prompt。

## 浓缩
空头需要明确 prompt 引导，不能靠自发涌现

## 架构含义
- Layer 1 无影响 — 做空是 Law 2 合法行为，不触碰永恒不变量
- Layer 2 影响 — SKILL prompt 和 forced investment round 逻辑需调整
- Run 11 Gemini 经济审计确认：零做空是病态信号，市场效率仅 6/10
- 大宪章 Engine 2 设计意图中空头刺客是核心角色（"猎杀幻觉"），但 LLM 天然偏向建设而非批判
- 两个改进方向：(1) SKILL prompt 显式强调做空盈利场景 (2) forced investment round 中随机指定 skeptic 角色

## 行动项
- [ ] 在 SKILL prompt 中添加做空盈利的具体示例和场景描述
- [ ] forced investment round 的 prompt 中对部分 agent 注入 skeptic 角色指令
- [ ] 考虑奇数轮强制做空投票（至少评估一个节点的 NO 价值）
