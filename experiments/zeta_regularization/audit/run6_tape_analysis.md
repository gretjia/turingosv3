# ζ(-1) = -1/12 Run #6 Tape 深度分析报告

**运行时间**: 2026-03-20 01:24:44 → 03:07:48 (103 分钟)
**配置**: N=15, MaxSteps=100, Pro/DeepSeek-V3.2, SiliconFlow
**变更**: 全部 d646b66 修复 + sandbox stderr 修复 + Big Bang Multiverse (ms jitter + collect-all) + root tombstone 注入 + P2 payload 日志 + P4 禁止 native_decide prompt
**结果**: 100/100 步全部 REJECTED, Tape 为空, WAL 为空, 未证明

---

## 一、原始数据总览

| 指标 | 值 |
|------|-----|
| Swarm 轮次 (Computing Step) | 7 |
| Kernel 步数 (queued outputs consumed) | 100 |
| Multiverse 分支 (agent 返回有效输出) | 105 |
| SELF-STAKE 事件 | 100 |
| Compiler VETO | 100 |
| 成功 Append | **0** |
| Bankruptcy | 0 |
| OMEGA | 0 |

### 架构变更验证：真并发生效

| 对比维度 | Run 3 (蒸馏版) | Run 6 (V3.2) |
|----------|---------------|-------------|
| Agent 分布 | Agent 0-4 (5个) | Agent 0-14 (全部15个) |
| 最不活跃 agent | Agent_5~14 = 0 次 | 最少 6 次 (均匀) |
| 每 agent 贡献 | 极度不均 (42:29:13:9:7) | 均匀 (全部 6-7 次) |

**ms jitter + collect-all-survivors 完全解决了 N=15 利用率问题。**

### 时间分析

| Swarm 轮次 | 起止时间 | 耗时 | 瓶颈 |
|-----------|---------|------|------|
| Step 1 | 01:24 → 01:32 | 7.4 min | Agent_3 慢返回 + 15次 Lean 编译 |
| Step 2 | 01:32 → 01:35 | 3.0 min | 正常 |
| Step 3 | 01:35 → 01:41 | 6.0 min | 正常 |
| Step 4 | 01:41 → 02:06 | **25.5 min** | Agent_4 耗时 8 min (可能 thinking 很长) |
| Step 5 | 02:06 → 02:10 | 3.6 min | 正常 |
| Step 6 | 02:10 → 02:32 | **22.8 min** | Agent_8 网络超时 + Agent_4 耗时 8 min |
| Step 7 | 02:32 → 03:07 | **35 min** | 未完成，100 步在此轮耗尽 |

**collect-all 的代价**：必须等最慢的 agent 返回才能推进。Agent_4 和 Agent_8 反复成为瓶颈 (8-20 min)，拖慢了整体节奏。

---

## 二、编译错误分类

| 错误类型 | 次数 | 占比 | 含义 |
|---------|------|------|------|
| `rewrite failed: pattern not found` | 118 | **48.8%** | `-1 ≠ -(↑1)` coercion 不匹配 |
| `unsolved goals` | 40 | 16.5% | tactic 部分执行但未关闭 goal |
| `Invalid argument name n` | 30 | 12.4% | lemma 参数名是 `k` 不是 `n` |
| `exact? could not close` | 26 | 10.7% | 搜索失败 |
| `No goals to be solved` | 6 | 2.5% | goals 被关闭但有其他错误 |
| `Function expected` | 6 | 2.5% | 类型错误 |
| `simp made no progress` | 4 | 1.7% | simp 无效 |
| `Type mismatch` | 4 | 1.7% | 类型不匹配 |
| `Application type mismatch` | 2 | 0.8% | 参数类型错误 |
| `Unknown identifier bernoulli_two` | 2 | 0.8% | 幻觉 lemma 名 |

**核心瓶颈**: 48.8% 的失败都是同一个根因 — `rw [riemannZeta_neg_nat_eq_bernoulli' 1]` 失败，因为 goal 中的 `-1` 是 `(-1 : ℂ)` 而 lemma 的 pattern 是 `-(↑(1 : ℕ) : ℂ)`。

---

## 三、策略分析

### 18 种独特策略（按频率排序）

| 频率 | 策略 | 分析 |
|------|------|------|
| **51** | `rw [riemannZeta_neg_nat_eq_bernoulli' 1]; norm_num` | 主策略，coercion 墙 |
| **14** | `rw [riemannZeta_neg_nat_eq_bernoulli' (n := 1)]; norm_num` | 参数名错 (k 不是 n) |
| **13** | `exact?` | 搜索失败 |
| 3 | `exact (riemannZeta_neg_nat_eq_bernoulli' 1).trans (by norm_num)` | 聪明但还是 coercion 墙 |
| 3 | `rw [riemannZeta_neg_nat_eq_bernoulli' (1 : ℕ)]; norm_num` | 显式标注 ℕ，还是不够 |
| 3 | `rw [riemannZeta_neg_nat_eq_bernoulli' 1]` | 无 norm_num |
| 2 | `rw [riemannZeta_neg_nat_eq_bernoulli' 1 (by norm_num)]; norm_num` | 多余参数 |
| 1 | `rw [show (-1 : ℂ) = -((1 : ℕ) : ℂ) by norm_num, riemannZeta_neg_nat_eq_bernoulli'...]` | **最接近正确！** |
| 1 | `simpa using (riemannZeta_neg_nat_eq_bernoulli' 1)` | simpa 路径 |
| 1 | `simpa using riemannZeta_neg_nat_eq_bernoulli' 1` | 同上 |
| 1 | `simp [riemannZeta_neg_nat_eq_bernoulli' 1, bernoulli'_two]; norm_num` | 尝试带入 bernoulli 值 |
| 1 | `simp [riemannZeta_neg_nat_eq_bernoulli']; norm_num` | simp 展开 |
| 1 | `rw [riemannZeta_neg_nat_eq_bernoulli' 0]` | 参数错 (应该是 1) |
| 1 | `rw [riemannZeta_neg_nat_eq_bernoulli' 1]; norm_num [bernoulli_two]` | 幻觉 bernoulli_two |
| 1 | `norm_num [riemannZeta_neg_nat_eq_bernoulli'...]` | norm_num 展开 |
| 1 | `exact (riemannZeta_neg_nat_eq_bernoulli' 1 (by norm_num)).trans (by norm_num)` | 多余参数 |
| 1 | `apply?` | 探索性，得到了有价值的 `Try this:` 建议 |

### 定价行为分析

| 价格区间 | 次数 | 占比 |
|---------|------|------|
| 5-50 | 3 | 3% |
| 50-200 | 6 | 6% |
| 200-1000 | 41 | 41% |
| 1000-2000 | 48 | 48% |
| 2000+ | 2 | 2% |

**定价范围**: 7.51 → 5432.19，方差巨大，anti-bot 协议完全生效。
**但定价信号失真**: LLM 对同一个失败策略 (`rw [...]; norm_num`) 反复给出高价 (487~1892)。信心与编译结果完全不相关。

---

## 四、问题清单

### P0. 致命：100% 的尝试撞在同一面 coercion 墙上

51/100 次使用完全相同的策略 `rw [riemannZeta_neg_nat_eq_bernoulli' 1]; norm_num`。所有都失败于：

```
Tactic `rewrite` failed: Did not find an occurrence of the pattern
  riemannZeta (-↑1)
in the target expression
  riemannZeta (-1) = -1 / 12
```

**根因**：`-1` 在 goal 中是 `(-1 : ℂ)` 字面量，但 `riemannZeta_neg_nat_eq_bernoulli' 1` 产生的 pattern 是 `riemannZeta (-(↑(1 : ℕ) : ℂ))`。Lean 4 的 `rw` 不会自动做 coercion unification。

**已验证的正确解法**是用 `have` + `simp` + `convert`：
```lean
have h := riemannZeta_neg_nat_eq_bernoulli' 1
simp at h
convert h using 1
norm_num
```

### P1. 严重：Graveyard feedback 生效但 LLM 未能逃离 attractor

root tombstone 修复确实生效了（Run 3 只有 4 种策略，Run 6 有 18 种）。但 LLM 在看到 "rw failed" 错误后，仍然在 51% 的时间里重复同一策略。

**原因分析**：Graveyard 只保留最近 3 条 tombstone（`bus.rs:18: if entry.len() > 3 { entry.pop_front(); }`），对于 15 个 agent 同一轮产生的 15 条相似错误，只有最后 3 条被保留。前 12 条的失败经验被覆盖。

### P2. 中等：collect-all 架构引入长尾 agent 瓶颈

Step 4 耗时 25.5 min、Step 6 耗时 22.8 min，原因是必须等最慢的 agent 返回。Agent_4 (8 min)、Agent_8 (网络超时 20 min) 是瓶颈。

**对比**：first-wins 模式下这些 step 会在 ~2 min 内完成。collect-all 用并行度换了等待时间。

### P3. 中等：`exact?` 搜索策略占 13% 但全部失败

`exact?` 无法 one-shot 关闭 goal，因为 Mathlib 中没有直接的 `riemannZeta (-1) = -1/12` 定理（它存在但以 `-↑k` 形式参数化）。但 `apply?` 返回了有价值的 `Try this:` 建议（`refine Complex.ext_iff.mpr ?_` 等），说明拆解为实部/虚部也是可行路径。

### P4. 低：`show` cast 路径只出现 1 次

最接近正确答案的策略 `rw [show (-1 : ℂ) = -((1 : ℕ) : ℂ) by norm_num, ...]` 只出现了 1 次。即使 Graveyard 提示了 coercion 问题，LLM 也没有系统性地收敛到 cast 策略。

---

## 五、深度洞察

### 洞察 1：这是一个 "最后一英里" 问题 — LLM 知道答案但说不出来

从 Run 3 到 Run 6 的策略进化：
- Run 3 (蒸馏版): `simp [h1, h2]` (幻觉假设, 41%)、`native_decide` (termination trap, 50%)
- Run 6 (V3.2): `rw [riemannZeta_neg_nat_eq_bernoulli' 1]; norm_num` (正确 lemma + 正确参数, 51%)

LLM 从"完全错误"进化到了"99% 正确但最后 1% coercion 搞不定"。这不是模型能力问题，是 Lean 4 类型系统的一个 **syntactic trap**：`-1` 和 `-(↑1)` 在数学上完全相同，但在 Lean 4 的 type theory 中是不同的 term。

**没有任何 LLM 能通过随机 tactic 试错来学会 `have h := ...; simp at h; convert h` 这个 pattern** — 这需要理解 Lean 4 的 coercion 语义，而不是 tactic 空间的随机搜索。

### 洞察 2：Swarm 的拓扑结构在 "全灭" 场景下退化为串行

当 Tape 为空（所有节点被 VETO），swarm 的 Boltzmann 路由器无节点可选，每轮都从初始 problem statement 出发 → 所有 agent 看到完全相同的 prompt → 产生高度相似的输出。

这解释了为什么 51/100 是同一策略。swarm 的 DAG 探索能力需要至少一个 "立足点"（已 append 的节点）才能展开。在全灭场景下，swarm 退化为 "N 次独立采样同一分布"。

### 洞察 3：Graveyard 的 3 条限制 vs 15 agent 的信息爆炸

每轮 15 个 agent 全部失败时，Graveyard 收到 15 条 tombstone，但只保留最后 3 条。这意味着：
- 12 条失败经验被丢弃
- 保留的 3 条很可能是同一种错误（因为策略趋同）
- LLM 在下一轮看到的 Graveyard 信息量极低

**更好的设计**：Graveyard 应该 dedup（相同错误只保留 1 条 + 计数），保留更多独特的失败模式。

### 洞察 4：`apply?` 的 `Try this:` 输出是金矿

`apply?` 返回了 Lean 4 的建议：
```
Try this:
  refine Complex.ext_iff.mpr ?_
  -- Remaining subgoals:
  -- ⊢ (riemannZeta (-1)).re = (-1 / 12).re ∧ (riemannZeta (-1)).im = (-1 / 12).im
```

这暗示了另一条证明路径：把复数等式拆成实部和虚部分别证明。如果 `riemannZeta (-1)` 的实部和虚部可以被 `norm_num` 或 `native_decide` 计算出来，这条路径可能更 LLM-friendly。

但更重要的是：**`Try this:` 输出被写入了 Graveyard 但 LLM 从未利用它**。因为 Graveyard 的 3 条限制 + 策略趋同，这些 `Try this:` 建议可能已经被覆盖了。

### 洞察 5：定价信号完全失真 — 高信心 ≠ 高质量

LLM 对 `rw [...]; norm_num` 反复给出 487-1892 的高价（表示高信心），但 100% 的时间这个策略都失败了。定价行为变成了 "复读 prompt 示例中的数字"（487.15 出现 19 次，正好是 SKILL 示例的数值）。

**anti-bot 精度协议** 成功防止了 `.0` 结尾，但没有防止 "复读示例精确值"。19 次 `487.15` 暴露了 SKILL 示例值被当成了新的 anchor。

### 洞察 6：定理形式化是根本解法

100 步 × 15 agent = 1500 次 LLM 调用，没有一次产生正确的 tactic 组合。问题不在模型、不在 swarm 架构、不在经济机制 — 而在于**定理的 Lean 4 形式化让正确答案不可达**。

已验证的正确证明需要 4 行、3 个不同的 tactic (`have`/`simp at`/`convert`/`norm_num`)。swarm 的单步 tactic 架构一次只提交 1 个 tactic，无法表达这种多步组合。即使 LLM 知道所有 4 个步骤，它也只能提交第一步 — 但第一步 `have h := riemannZeta_neg_nat_eq_bernoulli' 1` 会通过 sorry-test（不改变 goal），然后第二步 `simp at h` 需要在已有 `h` 的上下文中执行。

**这揭示了 swarm 架构的一个结构性限制**：当正确答案是多步 tactic 序列而非单步 tactic 时，swarm 需要支持多行 tactic 提交。

---

## 六、优先修复建议

| 优先级 | 修复 | 预期收益 |
|--------|------|---------|
| **P0** | 修改定理形式化：用 `(-(1 : ℕ) : ℂ)` 替代 `(-1)` 消除 coercion 墙 | 让 `rw` 直接匹配，可能 one-shot |
| **P0 备选** | 允许 LLM 提交多行 tactic（用 `\n  ` 分隔） | 释放 `have/simp/convert` 组合路径 |
| **P1** | Graveyard dedup：相同错误只保留 1 条 + 计数，扩大独特错误容量 | 保留 `Try this:` 等高价值信息 |
| **P2** | collect-all 增加超时：单个 agent 超过 3 min 不等，直接跳过 | 消除长尾 agent 瓶颈 |
| **P3** | SKILL 示例值随机化，防止 anchor 效应 | 定价信号更真实 |
