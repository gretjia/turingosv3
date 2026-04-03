---
date: 2026-04-02
status: proposed
related_commits: []
---

## 原话
> 压缩功能作为管理层，Librarian作为管理层agent，在此使用最好的模型（目前看是deepseek v3.2 reasoner）

## 浓缩
Librarian=管理层，用最强模型压缩

## 架构含义

### 管理层 vs 执行层 分离
```
执行层 (Worker Agents)    → 弱模型 (Qwen 7B/8B) × N 并发
  ↓ [raw tape output]
管理层 (Librarian Agent)  → 强模型 (DeepSeek Reasoner) × 1 离线
  ↓ [compressed memory]
执行层 (next generation)  → 带着压缩记忆的弱模型
```

### 经济学原理
- **执行层**: 量大、廉价、可并行 — 苦涩教训的体现 (scale > intelligence)
- **管理层**: 质高、昂贵、低频 — 压缩需要理解力，不能用弱模型
- **类比**: 公司里大量初级员工 + 少量高级经理。经理不写代码，经理压缩经验成指导手册

### 模型选择原则
- 管理层任务 (压缩、审计、摘要) → 用当前最好的模型
- 执行层任务 (推理、投资、做空) → 用最便宜够用的模型
- 两层模型可以独立升级，互不影响

### 与 DeepSeek Halt Gate 的关系
- Halt Gate = DeepSeek Reasoner 做**验证** (Engine 3)
- Librarian = DeepSeek Reasoner 做**压缩** (Engine 4)
- 同一模型，不同职责，不同触发时机

## 行动项
- [ ] Librarian 压缩时调用 DeepSeek Reasoner (而非本地规则)
- [ ] 压缩 prompt: "从这些 log 中提取可复用的数学推理策略和常见错误模式"
- [ ] 成本控制: 每次压缩一次 API 调用 (batch 处理)
