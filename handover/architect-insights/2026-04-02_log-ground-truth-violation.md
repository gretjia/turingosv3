---
date: 2026-04-02
status: proposed
related_commits: []
---

## 原话
> success/failure 节点内容和 rejection 原因的 top 5，但 "too short" 的 rejection 被 Dedup 压过了
> 说明log没有履行ground Truth的职责

## 浓缩
压缩前必须有完整持久日志，否则 Ground Truth 形同虚设

## 架构含义

### 当前的 Ground Truth 缺口

| 事件类型 | 持久化？ | 完整性？ | 问题 |
|---------|---------|---------|------|
| 成功 append | Tape (kernel) ✓ | 完整 ✓ | — |
| 被拒 append | rejection_log (内存) ✗ | top 5 截断 ✗ | **Ground Truth 违规** |
| 市场交易 | stdout ✗ | 完整但非结构化 | 不可程序化读取 |

关键缺陷: `rejection_log` 是 Vec<(String, String)>，每次 Librarian 压缩后 `.clear()`。
这意味着:
1. 压缩前的原始数据被销毁 — 违反 "log不可篡改"
2. 压缩 prompt 只取 top 5 — "too short" (重要反馈) 被 Dedup (59%) 挤掉
3. 如果系统 crash，压缩前的 rejection 数据全部丢失

### 应有的架构

```
事件发生 → 写入持久日志 (append-only, 不可删)
              ↓
         Librarian 定期读取完整日志
              ↓
         压缩为 memory (可能有损，但日志仍在)
              ↓
         agents 读取 memory
         (任何 agent 可回溯日志验证 memory 的声明)
```

核心原则: **日志 ≠ 内存缓冲区。日志必须是持久文件，append-only，不可 clear()。**

### 三种日志 (对应 2026-04-02 librarian-architecture.md)

| 日志 | 文件 | 内容 | 写入时机 |
|------|------|------|---------|
| success.jsonl | 成功 append 记录 | node_id, author, payload, price, timestamp | bus.append() Ok |
| failure.jsonl | 被拒 append 记录 | author, payload, reason, timestamp | bus.append() Err |
| market.jsonl | 交易记录 | buyer, node_id, direction, amount, new_price, timestamp | buy_yes/buy_no |

所有日志:
- JSON Lines 格式 (一行一条，可 grep)
- Append-only (只追加，不修改，不删除)
- 与 Tape 同级别的 Ground Truth 地位
- Librarian 压缩时读取完整日志，不依赖内存缓冲区

### 直接后果
Librarian 的 `build_compression_prompt` 应该:
- 从 failure.jsonl 读取 ALL rejection 原因，不是从内存 Vec 读 top 5
- 按类别分组统计 (dedup / too_short / bankrupt / blacklist / ...)
- 每个类别至少保留 1 条代表性样本
- 这样 "too short" 不会被 Dedup 淹没

## 行动项
- [ ] 创建 success.jsonl / failure.jsonl / market.jsonl 持久日志
- [ ] bus.rs append 成功/失败时写入对应日志
- [ ] Librarian 从日志文件读取，不依赖内存 Vec
- [ ] 废除 rejection_log.clear() — 日志不可清除
