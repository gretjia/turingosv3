---
date: 2026-03-30
status: implemented
related_commits: []
---

## 原话
> 对于那 5 个（33%）爆仓的 Agent，立刻触发大宪章引擎四。提取它们在 Run 8 中破产的审计日志，强制写入它们的专属 `skills/agent_X/autopsy.md`：
> "【FATAL 记忆】你在 P_yes = 99% 的劣质节点高位接盘，无视 APMM 流动性深度，导致滑点穿仓破产。下一世代，你必须严格使用凯利公式 (Kelly Criterion) 管理仓位，并专门寻找 P_yes < 20% 的带 sorry 的高赔率洼地，用严密的 Lean 4 推理补全它，去收割空头的资金！"
> 下一世代，它们将不再是盲目的跟风者，而是极其狡猾的华尔街价值投资者！

## 浓缩
尸检写入skills：破产→价值投资者演化

## 架构含义
- Engine 4 (拉马克演化) 已在 evaluator.rs 实现：破产时写 autopsy，胜利时写 victory
- 架构师确认此机制方向正确，重点表扬
- 尸检内容应包含具体的失败模式（高位接盘、忽视滑点）和具体的生存策略（Kelly Criterion、低 P_yes 洼地猎手）
- 当前实现的 autopsy 内容较泛（仅写 "BANKRUPT at balance X"），可强化为包含具体失败原因

## 行动项
- [x] 基础 autopsy 机制已实现 (evaluator.rs L698-713)
- [ ] 强化 autopsy 内容：从审计日志提取具体失败模式（高位接盘节点 ID、滑点数据）
- [ ] 强化 autopsy 策略建议：写入 Kelly Criterion + 低 P_yes 洼地猎手策略
