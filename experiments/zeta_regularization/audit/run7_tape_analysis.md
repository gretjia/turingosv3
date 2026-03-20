# ζ(-1) = -1/12 Run #7 Tape 深度分析报告

**运行时间**: 2026-03-20 03:22 → 仍在运行 (已 ~55 min)
**配置**: N=15, MaxSteps=100, Pro/DeepSeek-V3.2, SiliconFlow
**变更**: 删除所有 hard-coded 提示 + 多行 tactic 支持 + Graveyard dedup (3→10 unique)
**截至分析时**: 7 swarm 轮次, 90 kernel steps consumed, 0 appended, 0 OMEGA

---

## 一、关键数据

| 指标 | Run 6 (有提示) | Run 7 (无提示) |
|------|---------------|---------------|
| Unique 策略数 | 18 | **37** |
| 多行 tactic 尝试 | 0 | **10+** |
| `have h := ...` 模式 | 0 | **6** |
| `show` cast 模式 | 1 | **4** |
| `unfold` 模式 | 0 | **3** |
| `library_search` | 0 | **21** |
| 幻觉 lemma | 2 (bernoulli_two) | 多种探索性猜测 |

**策略多样性翻倍** — 删除 hard-coded 提示后，LLM 不再被锁定在 `riemannZeta_neg_nat_eq_bernoulli'` 上，探索空间显著扩大。

---

## 二、问题清单

### P0. 致命：LLM 幻觉了 `riemannZeta_neg_nat` — 这个 lemma 不存在

94 次 `Unknown identifier riemannZeta_neg_nat` 错误。Run 6 中 LLM 用的是 `riemannZeta_neg_nat_eq_bernoulli'`（存在），但 Run 7 中没有提示后，LLM 自己"发明"了 `riemannZeta_neg_nat`（不存在）。

**多行 tactic 中的关键策略链全部基于这个幻觉 lemma**：
```
have h := riemannZeta_neg_nat 1
norm_num at h
exact h
```
逻辑完全正确，如果 lemma 存在的话这就是正确答案。但名字错了。

### P1. 严重：`show` cast 策略找到了正确的 coercion 修复但 lemma 名错

```
rw [show (-1 : ℂ) = -((1 : ℕ) : ℂ) by norm_num]
rw [riemannZeta_neg_nat 1]    ← 这个不存在
norm_num
```

coercion 处理正确！但第二步的 lemma 名仍然是幻觉。

### P2. 中等：`library_search` 占 21% 但全部超时或失败

LLM 尝试了 21 次 `library_search`，这是正确的探索策略（让 Lean 4 自己搜索），但结果是 `unknown tactic` — 因为 Lean 4 中该 tactic 已被重命名为 `exact?`。

### P3. 中等：多行 tactic 解析成功但语义被截断

Payload 中可见多行 tactic（`have h := ...\n  norm_num at h\n  exact h`），说明 LLM 确实利用了多行能力。但部分 payload 在日志中被截断（200 字符限制），无法确认完整内容。

### P4. 低：`No goals to be solved` 出现 14 次

比 Run 6 的 6 次更多。多行 tactic 中有些组合关闭了所有 goals 但又引入了其他错误。

---

## 三、深度洞察

### 洞察 1：删除提示后涌现了真正的探索行为

Run 6（有提示）：51% 重复同一策略 `rw [riemannZeta_neg_nat_eq_bernoulli' 1]; norm_num`
Run 7（无提示）：最高频策略 `library_search` 只占 21%，其次 `rw [riemannZeta_neg_nat 1]; norm_num` 15%

**策略分布从单峰变成了长尾分布** — 这正是多 agent 探索应该产生的形态。37 种独特策略 vs Run 6 的 18 种，探索宽度翻倍。

但涌现出的"创新"策略全部基于幻觉 lemma 名，实际有效探索为零。

### 洞察 2：LLM 独立发明了正确的证明架构但叫错了名字

多个 agent 独立发明了这个策略：
```
have h := riemannZeta_neg_nat 1
norm_num at h
exact h
```

这与已验证的正确证明结构惊人地相似：
```
have h := riemannZeta_neg_nat_eq_bernoulli' 1
simp at h
convert h using 1
norm_num
```

**证明架构正确，lemma 名幻觉是唯一障碍。** 这说明 V3.2 的数学推理能力足够，但 Lean 4 Mathlib 的 API surface 记忆不精确。

### 洞察 3：Graveyard dedup 生效但没有改变结局

Run 7 中 `Unknown identifier riemannZeta_neg_nat` 被记入 Graveyard，但 LLM 看到这个错误后仍然继续使用同一个幻觉名字 — 因为它不知道正确的名字是什么。Graveyard 告诉 LLM "这条路不对"，但无法告诉 LLM "正确的路在哪"。

**这是 Graveyard 机制的结构性限制**：它只能做证伪（这个不行），不能做证实（那个可以）。在 LLM 的知识分布中，`riemannZeta_neg_nat` 是它能想到的最可能的名字，Graveyard 否决了它但 LLM 没有更好的替代品。

### 洞察 4：多行 tactic 释放了组合推理能力但受限于单轮成功

在 Run 6 中，LLM 永远只能提交 1 个 tactic → swarm 只在 1-tactic 空间搜索。
在 Run 7 中，LLM 提交了 `have → norm_num at → exact` 三步组合 → 搜索空间从 O(T) 扩展到 O(T^k)。

**但 0 个 append 意味着没有节点可以被后续 step 构建在上面**。swarm 的树搜索退化为反复从 root 出发的独立采样。多行 tactic 的真正价值需要在部分成功（有 append）的场景下才能展现。

### 洞察 5：问题的本质 — LLM 的 Mathlib API 知识边界

6 轮测试（Run 1-7）的所有数据指向同一个结论：

- **LLM 知道证明的数学逻辑**（have → simplify → conclude）
- **LLM 知道 Mathlib 中有 riemannZeta 相关定理**（命名猜测非常接近）
- **LLM 不知道精确的 Lean 4 Mathlib API 名称**（`riemannZeta_neg_nat` vs `riemannZeta_neg_nat_eq_bernoulli'`）

这不是推理能力的问题，是 **训练数据中 Lean 4 Mathlib 的覆盖率** 问题。任何模型（R1、V3.2、Claude）都可能有同样的盲区。

**swarm 能做的**：如果有一个 agent 碰巧猜对了名字（像 Run 5 的 R1 那样），多行 tactic 机制就能让它一步到位完成证明。swarm 的价值在于 **增加碰对的概率**（N 个独立采样），但如果正确名字不在任何 agent 的生成分布中，N 再大也无用。

---

## 四、结论

**multi-agent 涌现已经在发生**：
- 策略多样性：37 种独特策略，远超单 agent
- 独立发明了正确的证明架构（`have → simp → exact`）
- 探索了 `library_search`、`exact?`、`apply?`、`unfold`、`show` cast 等多条路径

**但涌现的天花板是 LLM 的知识边界**：
- 所有 agent 共享同一个模型的知识分布
- 如果 `riemannZeta_neg_nat_eq_bernoulli'` 不在这个分布的高概率区域，再多 agent 也无法采样到它
- 真正的涌现需要**异构 agent**（不同模型、不同 temperature、不同 prompt 策略）来突破单一分布的边界
