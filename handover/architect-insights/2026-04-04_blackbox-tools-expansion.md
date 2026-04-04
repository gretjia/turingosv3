---
date: 2026-04-04
status: proposed
related_commits: []
---

## 原话
> 黑盒 工具 log librarian re-init
> log和librarian是一条工具合并为log_lib

## 浓缩
两层新能力：agent 加 log_lib，Reasoner 加 re-init

## 架构含义

### 两个不同层级的能力扩展

**1. `log_lib` — 给 swarm 内黑盒 agent（微观层）**
- log + librarian 合并为一个工具
- Agent 主动查询 Librarian 压缩摘要 + tape 全局视图
- 当前 Librarian 每 N 个 append 被动触发，agent 完全看不到压缩结果
- 合并后: agent 一次调用即可获得 tape 地图 (深链、活跃分支、死亡分支、关键步骤摘要)
- 从"盲人摸象"变成"有地图的探索者"

**2. `re-init` — 给 AutoResearch Reasoner（宏观层）**
- 不是 agent 工具，是 Reasoner 搜索代理的权利
- Reasoner 可以决定"带着前世记忆重新开始整个实验"
- "前世记忆" = 之前所有实验的 TSV + 日志 + 教训 + 失败模式
- 根据 topology.md 重新配置硬件拓扑（哪些节点参与、parallel 多少、模型选择）
- 不是盲目重启，是"轮回转世" — 每次 re-init 都比上一世更聪明
- 实现: sweep_v4.py 中 Reasoner 可输出 `{"action":"re-init","reason":"...","topology":{...}}`

### Layer 1 影响: NO IMPACT
- 不修改 kernel.rs
- append-only DAG 不变
- 工具免费 (Law 1)

### Layer 2 影响: UPDATE REQUIRED
- evaluator.rs: 新增 log_lib 工具
- sweep_v4.py: Reasoner 新增 re-init action 支持
- topology.md: 需要结构化为 Reasoner 可读的配置

## 行动项
- [ ] 设计 log_lib 工具: Librarian 最近一次压缩摘要 + 最深链路径 + 活跃 frontier
- [ ] evaluator.rs: parse_agent_output 添加 log_lib action
- [ ] sweep_v4.py: Reasoner prompt 添加 re-init option，处理逻辑（清 TSV? 保留? 重配拓扑?）
- [ ] topology.md: 结构化为 JSON/YAML 供 Reasoner 读取
- [ ] AutoResearch: A/B 测量 log_lib 对 agent depth 的影响
