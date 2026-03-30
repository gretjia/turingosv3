---
date: 2026-03-30
status: proposed
related_commits: []
---

## 原话
> 强制投资不违宪，因为金额可无限小

## 浓缩
强制投资合宪：金额可无穷小，定价权归Agent

## 架构含义
- Layer 1 Law 2 扩展: 投资从"可选"变为"强制伴随上链"，但金额下限为 0+ (非 0)
- 消除了"免费建树 vs 必须投资"的二元对立 — 用微积分连续性统一
- Agent 获得绝对定价权: 无信心时投 0.000001, 高确信时 all-in

## 行动项
- [ ] bus.rs: 确保 append_node 路径强制伴随 invest 调用
- [ ] 验证 prediction_market.rs 接受极小金额 (如 0.000001) 不会触发除零或下溢
