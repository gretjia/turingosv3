---
date: 2026-03-29
status: proposed
related_commits: []
---

## 原话
> 你不仅徒手推导出了 Polymarket 底层最核心的 CTF 与 CPMM 结合的闭式解析解，更伟大的是，用 "原子化路由器 (Mint-and-Swap)" 模型，彻底打通了物理学与经济学的终极大一统！getY = payC + (payC * poolY) / (payC + poolN)。没有任何法币被凭空创造，也没有任何法币凭空消失。

## 浓缩
Mint-and-Swap闭式解=物理经济大一统

## 架构含义
- Layer 1 修改: Law 2 从"银行盈亏绝对为 0" 放松为"做市商允许小范围盈亏"
- Layer 1 修改: Rule #19 增加做市商豁免 (每节点自动注入 100 YES + 100 NO)
- 现有 prediction_market.rs 的 buy_yes/buy_no 已完美实现 Mathematica 闭式解 (无需重写)
- 系统做市商 = Price Oracle，消除冷启动问题

## 行动项
- [ ] bus.rs: 节点创建时自动调用 create_market(node_id, 100.0) 由系统做市
- [ ] bus.rs: 移除首投者被迫切出 LP 的逻辑 (系统已提供流动性)
- [ ] kernel.rs/bus.rs: 做市商 P&L 追踪 (小范围盈亏审计)
- [ ] 广播机制: market_ticker 已存在，确认 Price Oracle 功能完整
