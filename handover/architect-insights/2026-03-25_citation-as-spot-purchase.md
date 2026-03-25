---
date: 2026-03-25
status: proposed
related_commits: []
---

## 原话
> "Agent B 如果想基于 Step 1 往下推导 Step 2。对不起，没有免费的白嫖！Agent B 必须立刻、当场向 Step 1 的 AMM 池支付真金白银，买走 100 枚 $T_1 作为引用权。"

## 浓缩
引用即买入，知识产权有价

## 架构含义
- 当前系统：引用是免费的（只需 stake >= 1.0 写入新节点）
- 新机制：引用本身就是一次现货交易，成本由 AMM 滑点实时决定
- 越热门的节点，引用越贵 — 自然形成"知识产权定价"
- 早期创建者通过持有 Token 获得被引用的版税收入

## 行动项
- [ ] 重写 kernel append 逻辑：引用前必须 swap_coin_for_token
- [ ] 引用成本动态化：从固定 stake 到 AMM 滑点定价
