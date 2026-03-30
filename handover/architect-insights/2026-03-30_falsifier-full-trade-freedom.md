---
date: 2026-03-30
status: proposed
related_commits: []
---

## 原话
> Falsifier 享有完全平等的交易权限，物理规则完全允许其在确信时买 YES

## 浓缩
Falsifier完全平权：协议绝不歧视资金方向

## 架构含义
- Falsifier 在 Prompt 策略上倾向"买 NO"（猎杀幻觉），但物理规则不限制方向
- 废除任何在代码层面限制 Falsifier 只能买 NO 的逻辑
- 自由市场原则: 协议是中性管道，不歧视任何参与者的资金方向

## 行动项
- [ ] 审查 bus.rs / skill prompt 中是否存在限制 Falsifier 交易方向的代码
- [ ] 确保 Falsifier 的 invest 调用与 Builder 使用相同路径，无方向性限制
