# ζ(-1) = -1/12 Run #3 Tape 深度分析报告

**运行时间**: 2026-03-19 15:32:10 → 16:54:14 (82 分钟)
**配置**: N=15, MaxSteps=100, DeepSeek-R1-Distill-Qwen-32B, SiliconFlow
**变更**: sandbox stderr 修复 (合并 stdout+stderr), d646b66 三修复 (自由定价+破产清算+断路器)
**结果**: 100/100 步全部 REJECTED, Tape 为空, WAL 为空

---

## 一、原始数据总览

### 100 步，零节点存活

| 类别 | 次数 | 占比 |
|------|------|------|
| Compiler/Sandbox Error | 99 | 99% |
| Bankrupt | 1 | 1% |
| 成功 Append | **0** | **0%** |

### Agent 分布

| Agent | 被选中次数 | 说明 |
|-------|-----------|------|
| Agent_0 | 42 | 0s stagger, 最快 |
| Agent_1 | 29 | 10s stagger |
| Agent_2 | 13 | 20s stagger |
| Agent_3 | 9 | 30s stagger |
| Agent_4 | 7 | 40s stagger |
| Agent_5~14 | 0 | 从未被使用 |

### 时间分布

- 总耗时 82 分钟，平均每步 49 秒
- 15:32 → 16:54，100 步匀速推进，无加速也无卡顿

---

## 二、编译错误分类（核心发现）

### 错误模式 A："No goals to be solved" + "fail to show termination" (50 次, 50%)

```
/dev/stdin:8:2: error: No goals to be solved
/dev/stdin:6:8: error: fail to show termination for
  zeta_neg_one
with errors
failed to infer structural recursion:
no parameters suitable for structural recursion

well-founded recursion cannot be used, `zeta_neg_one` does not take any (non-fixed) arguments
```

**这是最关键的发现。** 拆解：

1. `error: No goals to be solved` — 证明目标已经被关闭！**定理实际上被证了！**
2. 但 `fail to show termination for zeta_neg_one` — Lean 4 的 termination checker 拒绝接受

**根因**：LLM 生成的 tactic 成功关闭了所有 proof goals，但 Lean 4 认为 `zeta_neg_one` 的定义涉及某种递归结构（可能是 `native_decide` 或 `decide` 引入的），而该"递归"无法证明终止。

**关键**：这不是 tactic 错误，是**定理声明或 tactic 引入了伪递归**。可能 LLM 用了 `native_decide` 或 `omega` 等宏，它们在内部构造了 Lean 认为需要 termination proof 的结构。

### 错误模式 B：`simp [h1, h2]` — Unknown identifier (41 次, 41%)

```
/dev/stdin:7:13: error: Unknown identifier `h1`
/dev/stdin:7:17: error: Unknown identifier `h2`
/dev/stdin:7:2: error: `simp` made no progress
```

LLM 在 `simp only [h1, h2]` 中引用假设 `h1`, `h2`，但定理声明中没有假设。`riemannZeta (-1) = -1/12` 是一个纯等式命题，没有任何局部假设可引用。

### 错误模式 C：幻觉 Mathlib lemma 名 (16 次)

| 尝试的名称 | 次数 | 实际是否存在 |
|-----------|------|------------|
| `riemannZeta_neg_one` | 10 | **未知** — 可能是正确名称！ |
| `Complex.riemannZeta_neg_one` | 2 | 未知 |
| `zeta_neg_eq_Bernoulli` | 2 | 大概率不存在 |
| `zeta_two_eq_pi_sq_div_six` | 1 | 大概率不存在 |
| `eta_zeta_rel` | 1 | 大概率不存在 |

**注意**：`riemannZeta_neg_one` 出现 10 次，如果这个 lemma 确实存在于 Mathlib，`exact riemannZeta_neg_one` 就是 one-shot 解法。

### 错误模式 D：其他 (2 次)

- `linarith failed` (1 次) — 不适用于此类非线性问题
- `simp_all made no progress` (1 次)

---

## 三、问题清单

### P0. 致命：50% 的尝试实际上"证了"但被 termination checker 杀死

LLM 找到了关闭所有 goals 的 tactic，但 Lean 4 报 `fail to show termination`。这不是 LLM 能力不足，而是**定理形式化有缺陷**或 **LLM 使用了错误的宏 tactic**。

需要确认：LLM 在这 50 次中具体使用了什么 tactic？（当前日志没有记录被 reject 的 payload 内容）

### P1. 严重：Membrane 未识别 "No goals to be solved" 在 Err 分支

`lean4_membrane_tool.rs` 只在 `Ok(output)` 分支中检查 `"error: No goals to be solved"`。但 sandbox 修复后，这个字符串出现在 `Err(e)` 分支（因为 exit code != 0）。

```rust
// 当前代码
Ok(output) => {
    if output.contains("error: No goals to be solved") {
        // OMEGA!  ← 只在这里检查
    }
}
Err(e) => {
    // VETO!  ← "No goals to be solved" 出现在这里但被忽略
}
```

**但要注意**：这 50 次的 "No goals" 伴随着 "fail to show termination"，不是真正的 OMEGA — 证明在逻辑层面关闭了 goals 但在类型检查层面不合法。如果简单地把它当 OMEGA 会接受不合法的证明。

### P2. 严重：被 reject 的 tactic payload 未被记录

日志只显示 compiler error，不显示 LLM 实际生成的 tactic。无法做 post-hoc 分析来理解 LLM 的策略。这对改进 prompt 和形式化至关重要。

### P3. 中等：LLM 对此定理的 tactic 策略空间极窄

100 步中只产生了 4 种策略模式：
1. `simp only [h1, h2]` — 幻觉假设 (41%)
2. 某种触发 termination error 的 tactic (50%)
3. `exact riemannZeta_neg_one` 变体 — 幻觉 lemma (16%)
4. `linarith` / `simp_all` — 完全不适用 (2%)

没有尝试 `exact?`、`apply?`、`search_proofs` 等探索性 tactic。

### P4. 中等：有效 N 依然只有 5

Agent_0~4 被使用了 100 次，Agent_5~14 为零。10s stagger 在 ~50s 步长下意味着 Agent_5+ 还没出结果就被抢先了。

### P5. 低：单次 Bankrupt (Step 88)

Agent_0 在 Step 88 bankrupt (balance=150)。说明自由定价在生效（不是全部扣 500），但 99 次扣款累计还是耗尽了 10000 coins。

---

## 四、深度洞察

### 洞察 1：LLM 半只脚已经踏入了正确答案

50 次 "No goals to be solved" 说明 LLM 确实能找到关闭 proof goals 的 tactic。这比 Run 1（0 次成功 append）有了质的飞跃——**但 Lean 4 的 termination checker 是最后一道隐形墙**。

这个问题不在 LLM 能力，而在于：
- `native_decide` 或 `decide` 可能在内部引入了 Lean 认为需要终止证明的递归结构
- 或者 LLM 用了某种 `by { ... ; exact ... }` 模式，意外触发了定义递归

**如果知道 LLM 用了什么 tactic，就能精确修正 prompt 来避免 termination trap。** 这回到了 P2（payload 未记录）。

### 洞察 2：`riemannZeta_neg_one` 可能就是 one-shot 答案

LLM 尝试了 `exact riemannZeta_neg_one` 10 次，全部报 `Unknown identifier`。但 Mathlib 命名惯例高度一致（函数名_参数_结果），这个名字是非常合理的猜测。

**两种可能**：
1. Mathlib 确实有但名称不同（可能叫 `riemannZeta_neg_nat_one` 或在不同 namespace 下）
2. Mathlib 没有这个精确值的定理

**在 Mac 上 `grep riemannZeta` Mathlib 就能确认**。如果存在，整个测试就变成 one-tactic 问题。

### 洞察 3：sandbox 修复产生了质的差异——但还不够

Run 1/2：100% 步骤报空 error → 零学习 → 纯盲猜
Run 3：100% 步骤有真实 error → Graveyard 有内容 → 但 LLM 还是不断重复 `simp [h1, h2]`

**为什么 Graveyard 反馈没生效？** 因为 Tape 为空（没有任何节点通过），Graveyard 的 tombstone 挂在 "root" 节点上，但 swarm 的 prompt 模板只在有 parent_id 时才注入 tombstones。当 Tape 为空时，parent_id 为空，tombstones 不被注入。

```rust
// swarm.rs
if !parent_id.is_empty() {
    if let Some(graves) = input.s_i.tombstones.get(&parent_id) {
        tombstones_str = graves.clone();
    }
}
```

**这是一个 feedback loop 断裂**：所有失败挂在 "root"，但 root 是虚拟节点，swarm 不查它的 tombstones。LLM 永远看不到之前的失败。

### 洞察 4：定理形式化需要调整

当前形式化：
```lean
open Complex in
theorem zeta_neg_one : riemannZeta (-1) = -1/12 := by
```

问题：
1. `-1/12` 在 Lean 4 中的类型推断可能不是 `ℂ` — `riemannZeta` 返回 `ℂ`，但 `-1/12` 可能被推断为 `ℚ` 或 `ℤ`，需要显式 cast
2. `open Complex` 可能不够，可能需要 `open Nat Int Complex` 或 `open scoped`
3. 如果 Mathlib 有现成 lemma，形式化可以简化为 `exact` one-liner

### 洞察 5：投入产出比分析

| 资源 | 消耗 |
|------|------|
| API 调用 | 100 步 × ~1 LLM call = ~100 次 SF API |
| 时间 | 82 分钟 |
| 资金 | 15 × 10000 = 150,000 TuringCoins 总预算，几乎全部烧毁 |
| 有效探索 | 4 种策略模式，全部失败 |

**但 sandbox 修复的价值被证明了**：我们现在确切知道了"为什么失败"，而不是像 Run 1/2 那样一无所知。

---

## 五、优先修复建议

| 优先级 | 修复 | 预期收益 |
|--------|------|---------|
| **P0** | 在 Mac Mathlib 中 `grep -r riemannZeta` 确认已有 lemma 名 | 可能直接 one-shot 解决 |
| **P1** | 修复 root tombstone 注入 — Tape 空时也要把 root 的 Graveyard 注入 prompt | 打通 feedback loop |
| **P2** | 日志记录被 reject 的 payload（至少 truncated 前 200 字符） | 用于 post-hoc 策略分析 |
| **P3** | 调整定理形式化 — 确认类型匹配 (`ℂ` vs 字面量) | 消除类型推断导致的隐性错误 |
| **P4** | 在 prompt 中显式禁止 `native_decide` / `decide` | 避免 termination checker 陷阱 |
