---
name: harness-reflect
description: "Harness 自省循环 — 评估规则效果、宪法健康、管线追踪"
user_invocable: true
---

# /harness-reflect — Living Harness 自我进化

> Karpathy: propose → evaluate → keep/discard → LOOP
> 没有反馈循环的 harness 只会膨胀, 不会进化。

## 触发时机

- 用户主动: `/harness-reflect`
- Session 结束时: stop-guard.sh 建议 (如果有新违规或规则触发)

## 流程

### Stage 1: 盘点

```
=== HARNESS INVENTORY ===
违规总数: N (incidents/)
规则总数: M (rules/active/)
已执法违规: X / N (X%)
最近触发: rules/enforcement.log
```

### Stage 2: 缺口分析

遍历 incidents/INDEX.yaml, 找 `enforcement: "none"` 的违规, 按 severity 排序 Top 5。

### Stage 3: 规则效果评分

```
对每条规则:
  trigger_rate = times_triggered / age_days
  bypass_rate = times_bypassed / (triggered + bypassed)
  
异常:
  trigger_rate == 0 且 age > 30d → 候选退休
  bypass_rate > 50% → 规则需重写
```

### Stage 4: 宪法健康 (TuringOS 特有)

```
=== CONSTITUTIONAL HEALTH ===
运行: scripts/constitutional_check.sh

Law 1 (Info Equality): PASS/FAIL
  - kernel.rs 域知识检查
  - SearchTool 零成本检查

Law 2 (Pure Capital Economy): PASS/FAIL
  - 无 fund_agent / mint_coins
  - CTF 守恒
  - 投资自愿 (有 PASS 选项)

Law 3 (Digital Property Rights): DEFERRED
  - Speciation engine 未实现

Engine 分离: PASS/FAIL
  - Engine 1 (tools) ≠ Engine 2 (markets) ≠ Engine 3 (oracle) ≠ Engine 4 (evolution)
```

### Stage 5: AutoResearch 健康 (如果在运行)

```
=== AUTORESEARCH STATUS ===
PID: <pid> (alive/dead)
Experiments: N runs
Best depth: X
ERS trend: ↑/↓/→
Last edit: <timestamp>
```

### Stage 6: 健康分

```python
enforcement_ratio = enforced_violations / total_violations
constitutional_pass = laws_passing / total_laws
rule_effectiveness = 1 - (bypass_rate)

health_score = enforcement_ratio * constitutional_pass * rule_effectiveness
# 目标: > 0.7
```

### Stage 7: 行动建议 (最多 5 条)

```
=== RECOMMENDED ACTIONS ===
[规则层]
1. 为 V-xxx 创建规则 — /lesson-to-rule V-xxx
2. 退休 R-yyy (从未触发)
[宪法层]  
3. kernel.rs 域知识泄漏检测到: <detail>
4. 运行 /validate 确认编译 + 测试
[运维层]
5. AutoResearch PID dead, 需要重启
```

## 输出存储

`rules/reflections/YYYY-MM-DD.md`
