---
name: lesson-to-rule
description: "将新违规/教训自动转化为可执行规则 — Living Harness Ω4 amplifier"
user_invocable: true
---

# /lesson-to-rule — 违规→规则 自动转化

> 核心: 每次宪法违规不只被记录, 还被编码为自动防护。
> TuringOS 特色: 规则覆盖 3 Laws + kernel purity + economic invariants

## 输入

`/lesson-to-rule <V-xxx>` 或 `/lesson-to-rule` (自动检测最近新增的未执法违规)

## 流程

### Stage 1: 读取违规上下文

1. 读取 `incidents/V-xxx_*/meta.yaml` — 获取 pattern, axiom, severity
2. 读取 `incidents/V-xxx_*/trace.md` — 获取完整执行记录
3. 读取 `incidents/V-xxx_*/root_cause.md` — 获取因果链
4. 如果 incidents/ 中没有此事件 → **自动创建 trace 目录**:
   a. 从 VIA_NEGATIVA.md 或 handover/ 读取上下文
   b. 从 `git log --all --oneline --grep="V-xxx"` 提取相关 commit
   c. 创建 `incidents/V-xxx_<slug>/` (meta.yaml + trace.md + root_cause.md + resolution.md)
   d. 更新 `incidents/INDEX.yaml`

### Stage 2: 分类违规模式

将事件归入已知模式:
- `constitutional_violation` — Law 1/2/3 违反
- `kernel_purity` — kernel.rs 域知识泄漏
- `economic_bug` — 市场/钱包/LP 逻辑 bug
- `design_failure` — 架构级错误决策
- `architecture_violation` — Engine 分离原则违反
- `measurement_bug` — 指标/日志测量错误
- `api_failure` — 外部 API (SiliconFlow/Volcengine) 失败
- `proof_depth_regression` — 证明深度退化

### Stage 3: 搜索现有规则

```bash
grep -l "source_incidents.*V-xxx" rules/active/*.yaml
```

- 已有覆盖 → 输出 "✅ 已有规则 R-yyy"
- 同 pattern 但不覆盖 → 提议扩展
- 没有 → 生成新规则

### Stage 4: 生成规则

生成 `rules/active/R-xxx_<slug>.yaml`, 遵循 `rules/RULE_SCHEMA.yaml`。

**约束**:
1. Rust 代码规则用 grep -P 检查 (正则匹配)
2. enforcement 默认 warn, 除非 severity=critical 或重复 2+ 次 → block
3. 规则总数 hard cap 30 条 (Karpathy 极简)

### Stage 5: 回归测试

```bash
# 对 incidents/ 中所有事件运行新规则 dry-run
for incident in incidents/V-*; do
    # 验证规则能 catch 目标事件
done
```

### Stage 6: 输出 → 等待用户确认

```
=== LESSON-TO-RULE REPORT ===
违规: V-xxx — <title>
模式: <pattern>
生成规则: R-zzz (enforcement: block/warn)
等待用户确认: [Y/n]
```
