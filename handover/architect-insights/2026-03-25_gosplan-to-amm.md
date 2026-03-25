---
date: 2026-03-25
status: proposed
related_commits: []
---

## 原话
> "用一个全局遍历的 map_reduce 函数事后发钱，本质上就是苏联国家计划委员会（GOSPLAN）的年终奖核算系统。它系统性地惩罚了在黑暗中摸索的先驱，奖励了最后一步搭便车的寄生虫。"

## 浓缩
事后分配 = 中央计划，事前定价 = 资本主义

## 架构含义
- hayekian_map_reduce() 是事后全局遍历，本质上是 GOSPLAN
- 风险倒挂问题：早期投资者承担最大风险，却因后来者稀释而获得最少回报
- AMM 恒定乘积 (x*y=k) 实现事前定价：引用即买入，价格由供需实时决定
- 从"事后分配正义"到"事前市场定价"的范式转换

## 行动项
- [ ] 新建 src/amm.rs (UniswapPool)
- [ ] 废除 hayekian_map_reduce() 全局遍历
- [ ] kernel.rs 引入 execute_invest + liquidate_omega
- [ ] Agent 新增 portfolio 持仓状态
