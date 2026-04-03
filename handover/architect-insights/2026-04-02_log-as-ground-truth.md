---
date: 2026-04-02
status: proposed
related_commits: []
---

## 原话
> log作为ground Truth不可篡改（这个意见作为用于避免大模型幻觉的指南）

## 浓缩
Log=不可篡改的事实，反幻觉锚点

## 架构含义

### Log vs Memory 的认识论地位
```
Log  (Ground Truth)  → 发生了什么 — 事实，不可改，不可删
Memory (Compressed)  → 我们学到了什么 — 解读，可更新，可修正
```

### 反幻觉设计原则
1. **LLM 压缩 log → memory 时可能产生幻觉** — 但 log 本身不可篡改
2. **当 memory 与 log 矛盾时，log 为准** — memory 是可证伪的
3. **任何 agent 可以引用 log 原文来反驳 memory 中的结论**
4. **Librarian 压缩时必须附带 log 引用** — 让结论可溯源

### 与大宪章的对齐
- **Tape = Append-Only DAG** (Law, 已有) — 架构层保证不可篡改
- **Log = Tape 的持久化** — WAL 机制已保证 crash recovery
- 新增: **Memory 不可覆盖 Log 的事实地位**

### 实践指导
- Librarian 输出格式: "根据 log [node_id] 显示..."（带引用）
- 审计时: 先读 log，再读 memory，发现矛盾即 flag
- 禁止: Librarian 在压缩时"修正"或"美化" log 中的事实

### Popperian 扩展
- Log = 观察事实 (不可证伪)
- Memory = 理论 (可证伪)
- 新 log 可以推翻旧 memory — 这就是科学方法

## 行动项
- [ ] Librarian 压缩输出必须包含 source node_id 引用
- [ ] 审计步骤: 抽样验证 memory 声明是否与 log 原文一致
- [ ] 文档化: 在 CLAUDE.md 中增加 "Log = Ground Truth" 准则
