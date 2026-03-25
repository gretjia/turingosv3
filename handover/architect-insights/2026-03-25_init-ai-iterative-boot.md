---
date: 2026-03-25
status: proposed
related_commits: []
---

## 原话
> "可以采用多次初始化的办法，先试跑几轮，然后让顶层看看有什么问题，改，然后重新初始化。"

## 浓缩
Init AI 迭代引导，顶层观测后重启

## 架构含义
- 新增 Init AI 模块：负责实验初始化、参数选择、测试集推荐
- 迭代式引导：试跑 → 顶层审查 → 修复 → 重新初始化
- 与 Agent $HOME 结合：每次初始化可选择保留/清空 $HOME
- 顶层白盒增加"观测-干预"循环能力

## 行动项
- [ ] 设计 Init AI 接口（参数推荐、测试集选择、health check）
- [ ] 实现 Agent $HOME 文件系统隔离（读公开、写私有）
- [ ] 支持多次初始化 without full restart
