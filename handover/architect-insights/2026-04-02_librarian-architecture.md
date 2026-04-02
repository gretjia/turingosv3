---
date: 2026-04-02
status: proposed
related_commits: []
---

## 原话
> 这里说的不是autoresearch，说的是整个turingosv3架构。要专门有一个librarian agent, 定期整理common error。记忆库是压缩出来的。定期把log压缩成memory。成功和失败都要log，而且分开log。

## 浓缩
Tape=原始日志，Memory=压缩智慧。Librarian定期蒸馏。全架构级。

## 架构含义

### Tape 与 Memory 的关系
```
Tape (Engine 1)          → 原始体验，append-only，永不删除
  ↓ [Librarian 离线压缩]
Memory (Engine 4)        → 压缩智慧，写入 learned.md
  ↓ [Agent 读取]
Next Run                 → 带着记忆的新一代 agent
```

### 三种日志，三种记忆
| 日志类型 | 来源 | 压缩产出 | 用途 |
|---------|------|---------|------|
| Success Log | 高价节点、Golden Path、OMEGA 链 | "什么推理策略有效" | 正面引导 |
| Failure Log | 被做空节点、死分支、rejected 内容 | "什么不该做" (via negativa) | 负面约束 |
| Market Log | 交易记录、价格变动、破产事件 | "什么时候该投资/做空" | 经济直觉 |

### 实现层级
1. **bus.rs / kernel.rs** — Tape 本身就是 raw log（已有）
2. **librarian.rs** — 新 TuringTool，定期读 tape，压缩成结构化记忆
3. **learned.md** — Agent 的长期记忆存储（Engine 4 已有路径）
4. **Cron / 代际触发** — 每次 generation 切换时或定时触发 librarian

### 与现有架构的对齐
- **Engine 1 (Tape)** = 免费 append = 原始日志积累
- **Engine 2 (Market)** = 价格信号 = 实时短期记忆
- **Engine 3 (Oracle)** = 真理仲裁 = 验证记忆的正确性
- **Engine 4 (Evolution)** = learned.md = **长期压缩记忆** ← Librarian 写入这里

### 不是什么
- 不是实时公告板（那是短期记忆，已实现为 global_bulletin）
- 不是 autoresearch 的实验管理工具
- 是 agent 跨 run 的**认知进化基础设施**
