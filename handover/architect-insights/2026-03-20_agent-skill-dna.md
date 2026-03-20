---
date: 2026-03-20
status: proposed
related_commits: []
---

## 原话
> 幸存 Agent 的 skill 没沉淀，需要 DNA 角色。Agent skill = DNA，多样性需要传承而非随机数。

## 浓缩
幸存Agent的skill组合是演化资产，必须像DNA一样传承给后代而非每次随机重置。

## 架构含义
- **Layer 1**: 不违反——DNA传承是SKILL层逻辑，不触碰kernel
- **Layer 2**: 可能需要新参数控制DNA遗传率vs变异率
- **机制扩展**: 需要在Agent生命周期中增加「继承」阶段——新Agent从高适应度Agent克隆skill配置，而非从零开始

## 行动项
- [ ] 设计 Agent DNA 数据结构（skill组合 + 权重 + 适应度历史）
- [ ] 在 swarm.rs 中实现「选择-继承-变异」循环
- [ ] 添加 DNA 持久化到 WAL/Tape
