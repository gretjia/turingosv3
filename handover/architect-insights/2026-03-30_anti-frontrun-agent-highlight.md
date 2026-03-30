---
date: 2026-03-30
status: proposed
related_commits: []
---

## 原话
> Agent 主动高亮单一步骤上链，用自决机制代替粗暴的物理截断

## 浓缩
防抢跑：Agent自主高亮单步，系统只认标记

## 架构含义
- 替代原有 max_payload_chars/lines 物理截断方案
- 控制权从系统下放到 Agent: Agent 用 `<step>...</step>` 标记上链内容
- 每次只允许高亮一个步骤 — 系统提取高亮内容上链
- 保留 Law 1 信息平权: Agent 可自由长篇推理，但上链内容是自选的原子步骤

## 行动项
- [ ] bus.rs: 实现 `<step>` 标签解析，提取高亮内容作为上链 payload
- [ ] 评估是否完全废除 max_payload_chars/lines 或保留为兜底
