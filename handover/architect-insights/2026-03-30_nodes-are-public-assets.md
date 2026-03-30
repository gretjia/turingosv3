---
date: 2026-03-30
status: implemented
related_commits: []
---

## 原话
> 虽然agent创立了节点，但节点并不是该agent的盘子，而是公共资产。agent也无需护盘。因此初始流动性LP=1000由系统提供。

## 浓缩
节点=公共资产，非创建者的盘，系统提供LP

## 架构含义
- Layer 1 Law 1 精神延伸: 节点是公共知识图谱的一部分，不是创建者的私有财产
- Agent 创建节点后无"护盘"义务 — 节点的价格由市场共识决定，非创建者维护
- LP=1000 由 SYSTEM_MM 提供的合法性根源: 系统做市是为公共品提供流动性，非为某个 Agent 护盘
- 这消除了"creator auto-long = 护盘"的错误心智模型: creator 买 YES 是表达信心，不是维护资产

## 行动项
- [x] bus.rs SYSTEM_MM 已实现系统级 LP 注入 (LP=1000)
- [ ] 审查 SKILL prompt 中是否存在暗示 Agent 应"护盘"或"维护自己节点"的措辞
