---
date: 2026-04-02
status: proposed
related_commits: []
---

## 原话
> Boltzmann规则：仅当子节点价格高于父节点的时候，父节点才可以被mask，否则父节点也参与随机竞选。

## 浓缩
子不如父，则父不退位。适者生存前沿。

## 架构含义

### 当前规则 vs 新规则
```
当前: frontier = 没有子节点的叶节点
新规: frontier = 没有子节点 OR 所有子节点价格 ≤ 自身价格
```

公式:
```
parent ∈ frontier ⟺ (children = ∅) ∨ (∀ child ∈ children: child.price ≤ parent.price)
parent masked    ⟺ ∃ child ∈ children: child.price > parent.price
```

### 经济学含义
- **高质量节点是粘性的** — P=98% 的节点只有当某个子节点超过 98% 时才退出前沿
- **低质量节点快速淘汰** — P=20% 的节点只要有任何子节点超过 20% 就被 mask
- **自然进化** — 前沿自动填充各深度层的最强节点 (survival of the fittest)

### 解决的问题
诊断: "前沿只有 2 个节点" → 15 agents 挤在 2 条路径上 → lineage score 无分化空间
修复: 高价祖先在其子孙不争气时重返前沿 → 前沿自动扩展 → 更多并行探索路径

### 与 Lineage Score 的协同
1. 新前沿规则: 确保高质量节点有资格被选中
2. Lineage Score: 在有资格的节点中，按血统强度分配算力
两者共同作用: 资本信号 → 前沿形态 → 算力分配 → 完整闭环

### 与大宪章的对齐
- Law 1 (信息平权): append 仍然零成本，任何 agent 仍可 append 到任何节点
- Law 2 (共识的代价): 使用 Engine 2 价格信号决定前沿形态 — 这正是价格应该做的事
- SKILL 层变更 (sdk/actor.rs), 不触碰 kernel

## 行动项
- [ ] 修改 `boltzmann_select_parent` 中的 frontier 过滤条件
- [ ] 同步修改 `build_chain_from_snapshot_with_temperature` 中的 Order Book frontier
