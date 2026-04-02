---
date: 2026-04-02
status: proposed
related_commits: []
---

## 原话
> 要专门有一个librarian agent, 定期整理common error。记忆库是压缩出来的。定期把log压缩成memory。首先前提要积累足够的log你才能压缩。比如你可以设置明天晚上人类睡觉的时候把今天的log压缩成记忆。所以成功和失败都要log。而且分开log。成功写成功log。失败写失败log。

## 浓缩
记忆=压缩后的日志。先积累，再压缩，成功失败分开存

## 架构含义
- **Memory ≠ Real-time** — 记忆不是实时产生的，是从积累的 log 中压缩蒸馏出来的
- **Log → Sleep → Memory** — 仿人类：白天体验(log)，睡眠压缩(librarian)，醒来带着记忆
- **Success/Failure 分离** — 成功 log 告诉你"什么有效"，失败 log 告诉你"什么无效"(via negativa)。混在一起会让成功信号被失败噪音淹没
- **Librarian Agent** — 不是实时公告板（那是短期记忆），而是定期离线压缩（长期记忆）
- **积累前提** — 没有足够的 raw log，压缩无意义。先跑实验，再蒸馏

## 行动项
- [ ] 创建 `logs/success/` 和 `logs/failure/` 分离存储
- [ ] 创建 `librarian.py` 压缩脚本
- [ ] 设置 cron：人类睡觉时自动压缩当天 log → memory
- [ ] 压缩产物写入 `autoresearch/zeta/memory/` 目录
