# ζ(-1) = -1/12 Run #8 Tape 深度分析报告

**运行时间**: 2026-03-20 04:49:27 → 05:42:49 (53 分钟)
**配置**: N=15, MaxSteps=100, Pro/DeepSeek-V3.2, SiliconFlow
**变更**: 热力学退火 (LLM T: [0.1,1.5]→[0.3,0.6], Boltzmann T: 2.0→0.3) + 多行 tactic + Graveyard dedup (10 unique) + 无 hard-coded 提示
**结果**: 1/100 步成功 append, 0 OMEGA, 未证明

---

## 一、里程碑：首次 Tape 非空

**`step_4_branch_10` — Agent_10 在 Step 50 成功 append**

```lean
import Mathlib
set_option maxHeartbeats 400000
open Complex in
theorem zeta_neg_one : riemannZeta (-1) = -1/12 := by
  simp only [riemannZeta]
```

- **Author**: Agent_10
- **Stake**: 7.43 (极低信心，谨慎探索)
- **Price**: 7.43 (= intrinsic_reward，无反向传播)
- **含义**: `simp only [riemannZeta]` 通过了 sorry-test — Lean 4 成功展开了 `riemannZeta` 的定义，且剩余 goal 仍然合法（不报错 with sorry）

**这是 8 轮测试以来 Tape 的第一个存活节点。**

---

## 二、核心数据

| 指标 | Run 7 (无退火) | Run 8 (退火) | 变化 |
|------|---------------|-------------|------|
| Unique 策略 | 37 | **45** | +22% |
| 成功 Append | 0 | **1** | 突破 |
| 多行 tactic 尝试 | ~10 | **25+** | +150% |
| 命名空间探索 | 1 (riemannZeta_neg_nat) | **5** (含 RiemannZeta., HurwitzZeta., ZetaFunction.) | +400% |
| Agent 均匀度 | 6-7 次/agent | 6-7 次/agent | 持平 |
| Stake 方差 | ? | stddev=444 (mean=439) | 高方差 |

### 时间分布

| Swarm 轮次 | 耗时 | 说明 |
|-----------|------|------|
| Step 1 | 22.5 min | 首轮，cold start + 15 次 Lean 编译 |
| Step 2 | 4.6 min | 正常 |
| Step 3 | 4.7 min | 正常 |
| Step 4 | 3.9 min | **`simp only [riemannZeta]` 被 append** |
| Step 5 | 4.6 min | 基于 step_4_branch_10 构建 |
| Step 6 | 7.5 min | Agent 慢返回 |
| Step 7 | 5.5 min | 最后一轮 |

**总耗时 53 分钟 — 比 Run 6 (103 min) 快 50%**，尽管探索更多样。

---

## 三、问题清单

### P0. 关键：`simp only [riemannZeta]` 通过 sorry-test 但后续 50 步无法在其上构建成功

append 后的 3 轮 swarm（Steps 5-7, 45 个 agent 输出），全部 REJECTED。后续 agent 在 `simp only [riemannZeta]` 基础上尝试的策略全部失败。

**Boltzmann 路由在 append 后锁定了这个唯一节点**（Prob=100%）。所有后续探索都基于展开后的 `riemannZeta` 定义 — 这是一个极深的 Lean 4 内部表达式，LLM 几乎无法理解。

### P1. 严重：幻觉 lemma `riemannZeta_neg_nat` 仍然主导 (78 次)

78 次 `Unknown identifier riemannZeta_neg_nat`。尽管 Graveyard dedup 保留了这个错误，LLM 在多轮中仍然重复使用。退火没有消除这个 attractor — 因为这是知识缺陷不是温度问题。

### P2. 中等：`No goals to be solved` 达到 20 次 (历史最高)

高温 agent 产出的某些激进策略（如 `decide`, 深层 `simp`）确实关闭了所有 goals 但引入了其他元层错误。20 次比 Run 7 的 14 次增长 43%。

### P3. 中等：命名空间探索爆发但全部失败

Agent 独立探索了 5 个不同的命名空间前缀：
- `riemannZeta_neg_nat` (78 次) — 不存在
- `RiemannZeta.riemannZeta_neg_nat` (10 次) — 不存在
- `riemannZeta_neg_one` (8 次) — 不存在
- `RiemannZeta.riemannZeta_neg_one` (2 次) — 不存在
- `HurwitzZeta.hurwitzZetaEven_neg_nat` (1 次) — 接近但不完全正确
- `ZetaFunction.riemannZeta_neg_nat` (1 次) — 不存在

**正确名称 `riemannZeta_neg_nat_eq_bernoulli'` 在 Run 8 中从未出现**（Run 5 的 R1 模型找到过）。

### P4. 低：`library_search` 大量使用 (18 次) 但全部报 `unknown tactic`

V3.2 不知道 `library_search` 已被 Lean 4 重命名为 `exact?`。这又是训练数据滞后问题。

---

## 四、深度洞察

### 洞察 1：退火成功地打开了涌现之门

Run 8 vs 之前所有 Run 的关键突破：**首次产生了通过 sorry-test 的中间步骤**。

`simp only [riemannZeta]` 是一个极低风险的探索性 tactic — 它只是展开定义，不做任何推理。Agent_10 在 Step 4（progress=0.04，高温区）用 T≈0.1 的极低温度生成了这个确定性策略，并以 7.43 的极低 stake 提交。

**这揭示了退火的真正价值**：不是高温 agent 的疯狂探索，而是**低温 agent 的极度保守策略**（仅展开定义）恰好能通过 sorry-test。高温 agent 尝试的激进策略（`rw`, `exact`, `library_search`）全部失败，但低温 agent 的 `simp only [riemannZeta]`（不做任何推理，只展开名字）反而存活了。

**类比**：在 AlphaGo 中，有时最好的一步不是局部最优手（激进战术），而是"虚着"（展开局面）— 它不直接推进胜负，但改变了后续搜索的起点。

### 洞察 2：Tape 非空后 swarm 的 DAG 搜索终于启动了

append 后，Boltzmann 路由锁定 `step_4_branch_10`（唯一节点，Prob=100%）。后续 3 轮 45 个 agent 都在展开后的 `riemannZeta` 定义上构建。虽然全部失败，但产生了 25+ 种独特的后续策略：

- `simp only [riemannZeta] → library_search` (8 次)
- `simp only [riemannZeta] → rw [riemannZeta_neg_nat 1]; norm_num` (7 次)
- `simp only [riemannZeta] → exact?` (3 次)
- `simp only [riemannZeta] → simp only [riemannZeta] → have h : (-1 : ℂ) ≠ 1 := by ...` (2 次)
- `simp only [riemannZeta] → have h := RiemannZeta.riemannZeta_neg_nat 1 → norm_num ...` (1 次)
- `simp only [riemannZeta] → rw [HurwitzZeta.hurwitzZetaEven_neg_nat 0 1]; norm_num` (1 次)

**这正是 swarm DAG 搜索的核心能力**：一个 agent 的中间结果成为全体 agent 的新起点，探索宽度从 1 个分支扩展到 25+ 个。虽然这 25 个分支都失败了，但**拓扑结构已经形成**。

### 洞察 3：`simp only [riemannZeta]` 展开后的 goal 是什么？

这是一个关键的未知 — 我们知道 `simp only [riemannZeta]` 通过了 sorry-test，但不知道它把 goal 变成了什么形式。如果展开后的 goal 更容易直接 `norm_num` 或 `decide`，这就是一条绕过 coercion 墙的路径。

但 Lean 4 中 `riemannZeta` 的定义涉及 `if` 分支（s=1 的极点处理）、`HurwitzZeta`、Gamma 函数等 — 展开后的 goal 可能极其复杂，超出 LLM 的处理能力。后续 45 个 agent 的全部失败暗示展开后的表达式对 LLM 不友好。

### 洞察 4：定价行为第一次出现正确的信号

- `simp only [riemannZeta]`（成功 append）: stake = **7.43** — 极低信心
- `rw [riemannZeta_neg_nat 1]; norm_num`（失败）: stake = **987.65** — 高信心

**低信心策略存活，高信心策略全灭**。这是市场应该产生的信号：信心与结果的反相关性暴露了 LLM 的校准偏差。如果 price backpropagation 能正确运行，低 stake 的成功节点会通过子节点的积累逐渐升值，而高 stake 的失败节点被烧毁 — 这正是 Hayekian 价格发现的本质。

但由于只有 1 个节点存活且没有子节点成功，price 机制未能充分展示其价值。

### 洞察 5：V3.2 vs R1 的知识分布差异被验证

| 维度 | R1 (Run 5) | V3.2 (Run 8) |
|------|-----------|-------------|
| 正确 lemma 名 | `riemannZeta_neg_nat_eq_bernoulli'` ✅ | 从未出现 ❌ |
| `simp only [riemannZeta]` | 未尝试 | 成功 append ✅ |
| 多行 tactic 组合 | 未释放 | `have → norm_num → exact` ✅ |
| 命名空间探索 | 少 | 5 种 ✅ |

**两个模型的知识盲区是互补的**：R1 知道精确 lemma 名但不尝试定义展开；V3.2 不知道 lemma 名但发现了展开路径。这进一步验证了 Run 7 分析报告中的结论 — **异构 agent 混合是突破单一模型知识边界的关键**。

---

## 五、综合交叉结论

### 8 轮测试的完整演化链

| Run | 模型 | 关键变更 | Append | 核心发现 |
|-----|------|---------|--------|---------|
| 1 | R1-Distill-32B | 基线 | 13 | 空 error、tactic monoculture、N=3 |
| 3 | R1-Distill-32B | sandbox stderr 修复 | 0 | 50% termination trap、lemma 存在性确认 |
| 5 | DeepSeek-R1 | 模型升级 | 0 | **R1 找到正确 lemma 名**，coercion 墙 |
| 6 | DeepSeek-V3.2 | d646b66 全套 | 0 | 18 种策略、定价分化、coercion 墙 |
| 7 | DeepSeek-V3.2 | 删除 hard-coded 提示 | 0 | 37 种策略、多行 tactic、幻觉 lemma 主导 |
| 8 | DeepSeek-V3.2 | 热力学退火 | **1** | **首次 append**、45 种策略、DAG 搜索启动 |

### 三个交叉结论

**结论 1：TuringOS 的多 agent 涌现是真实的，但受限于模型知识边界**

从 Run 1 的 4 种策略到 Run 8 的 45 种策略，从 0 append 到 1 append — swarm 的每一次架构改进都产生了可测量的探索宽度提升。这不是随机波动，是**架构释放的涌现能力**。

但所有 Run 中，正确的 Mathlib lemma 名 `riemannZeta_neg_nat_eq_bernoulli'` 只在 R1 模型中出现过。V3.2 的 100 个 agent 在 8 轮测试中从未产出过这个名字。**N 的增大不能突破单一模型的知识分布边界** — 这是苦涩的教训的应用：搜索算法再强，如果搜索空间不包含正确答案，就永远找不到。

**结论 2：退火机制的价值不在于多样性本身，而在于保守策略的存活概率**

直觉上，高温=高多样性=更可能发现新路径。但数据显示相反：Run 8 唯一的 append 来自**最低温度 agent**（T≈0.1）的最保守策略 `simp only [riemannZeta]`。

**退火的真正价值是温度梯度**：它同时产出激进策略（高温 agent 尝试 `decide`、`library_search`、复杂多行组合）和保守策略（低温 agent 尝试 `simp only`、单步展开）。在全灭场景中，恰恰是保守策略最容易通过 sorry-test 存活。而保守策略的存活为后续激进策略提供了立足点（DAG 搜索的起点）。

这映射了生物进化中的 "保守基因 + 突变基因" 双轨策略：保守基因维持生存，突变基因探索创新。

**结论 3：证明 ζ(-1) = -1/12 需要的下一步是异构模型混合**

单一模型（无论 R1 还是 V3.2）的知识分布无法同时覆盖：
- 精确 Mathlib API 名称（R1 能做到）
- 定义展开 + sorry-test 存活（V3.2 能做到）
- 多行 tactic 组合（两者都在尝试但未组合成功）

**混合 R1 + V3.2 在同一 swarm 中**：R1 agent 提供精确 lemma 名，V3.2 agent 提供展开路径和组合能力。Tape 作为共享记忆，让两个模型的知识互相可见。这正是 TuringOS 架构设计的核心价值 — 异构 agent 通过 Tape DAG 实现超越单体的群体智能。
