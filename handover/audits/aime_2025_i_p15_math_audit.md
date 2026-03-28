# AIME 2025 I P15 — 数学推理 + DAG 审计

**Date**: 2026-03-28
**Problem**: 3-adic 数论 — 统计 (a,b,c) ∈ [1,729]³ 中 3^7 | (a³+b³+c³) 的三元组数 mod 1000
**Answer**: 735
**Result**: NOT PROVED (300 Tx 上限)
**Significance**: TuringOS 首次在 AIME P15 (最难题) 上展示多 Agent 协作推理过程

---

## 1. DAG 概况

| 指标 | 数值 |
|------|------|
| 总 Tx | 300 |
| 上链节点 | 63 (去重后) |
| 被拒 | 237 (重复节点 — 同一 tactic 多次提交) |
| 抢跑拦截 | 0 (Agent 遵守单步规则) |
| 内核黑名单 | 0 |
| COMPLETE 声明 | 0 |
| SEARCH | **46** (前所未有的搜索量) |

## 2. 数学推理策略分析

Agent 探索了 **5 条独立推理路径**:

### 路径 A: Finset 计数路径
```
rw [hN] → Finset.card_filter → Finset.card_eq_sum_ones → simp
```
Agents: 4, 14, 2, 8, 10
尝试直接操作 Finset.card/filter 的计数公式。卡在: 无法将 card(filter) 展开为可计算的形式。

### 路径 B: Product 分解路径
```
Finset.sum_product → simp_rw [Finset.sum_product] → refine sum_congr
```
Agents: 4, 8, 0, 13, 12, 6
将三重求和分解为三层嵌套: Σ_a Σ_b Σ_c。这是正确的数学方向 — 分离变量后可以分析每个变量 mod 3^7 的立方分布。**但 Agent 无法完成内层求和的化简。**

### 路径 C: ZMod 转换路径 (最有前途)
```
ZMod.nat_cast_zmod_eq_zero_iff_dvd → Nat.cast_add → Nat.cast_pow
```
Agents: 0, 2, 5, 11, 12, 8, 10, 14
将 3^7 | (a³+b³+c³) 转化为 ZMod (3^7) 上的等式 (a³+b³+c³ = 0 in ZMod 2187)。**这是正确的 3-adic 方法**。Agent_2 (tx_197) 甚至写出了完整的等价引理:
```lean
have h_iff : ∀ (a b c : ℕ), M ∣ a^3 + b^3 + c^3 ↔
  ((a : ZMod M)^3 + (b : ZMod M)^3 + (c : ZMod M)^3 = 0) := by
  intro a b c; simp [ZMod.natCast_zmod_eq_zero_iff_dvd]
```

### 路径 D: Nat.modEq 路径
```
← Nat.modEq_zero_iff_dvd → Nat.dvd_iff_mod_eq_zero
```
Agents: 4, 12, 0, 7
将整除转为模等式。与路径 C 平行但更底层。

### 路径 E: padicValNat 路径 (最高级)
```
padicValNat_dvd
```
Agent_12 (tx_155) 搜索了 p-adic 估值引理 — 这是最直接的 3-adic 方法。但未能进一步推进。

## 3. 搜索行为分析

**46 次 Mathlib 搜索** — 所有 Run 中最高:

| 搜索词 | 次数 | 方向 |
|--------|------|------|
| `cube mod 3` / `cube mod 3^7` | 14 | 立方模运算 |
| `Finset.card_filter` | 6 | 集合计数 |
| `ZMod cube` / `ZMod.cube` | 5 | 模环立方 |
| `ZMod.nat_cast_zmod_eq_zero_iff_dvd` | 3 | 整除↔模零 |
| `Finset.sum_product` | 3 | 乘积和分解 |
| `dvd_iff_mod_eq_zero` | 2 | 整除等价 |
| `padicValNat_dvd` | 1 | p-adic 估值 |
| `x^3 + y^3 + z^3` | 1 | 直接搜索题目结构 |

**Agent_12 是最深度的研究者**: 搜索了 `card_filter`, `sum_product`, `padicValNat_dvd`, `ZMod cube`, `ZMod.sum_pow_eq_zero` — 覆盖了从组合到代数数论的完整工具链。

## 4. 为什么没有完成

证明 N mod 1000 = 735 需要:
1. ✅ 将整除转为 ZMod 等式 (路径 C 已完成)
2. ✅ 将三重求和分解为独立求和 (路径 B 已完成)
3. ❌ 分析 ZMod (3^7) 上的立方分布: 每个 a ∈ {1,...,729} 的 a³ mod 2187 有多少种取值
4. ❌ 计算满足 x+y+z = 0 (mod 2187) 的 (x,y,z) 三元组数, 其中 x,y,z 是立方剩余
5. ❌ 最终计算 N = 5 × 3^11, N mod 1000 = 735

步骤 3-5 需要深层 3-adic 分析 (Hensel 引理, 立方剩余分类), 超出了当前 LLM + 单步 tactic 的能力。

## 5. 有意义的发现

1. **Agent 独立发现了正确的数学方向** (ZMod + Finset.sum_product) — 这不是记忆, 是推理
2. **46 次搜索 = 真正的数学研究行为** — Agent 在 Mathlib 中寻找工具
3. **5 条平行推理路径** — DAG 分支探索有效
4. **Agent_2 的 h_iff 引理** (tx_197) 是高质量的中间结果 — 完整的整除↔ZMod 等价
5. **Agent_12 搜索 padicValNat** — 展示了向高级数论方法的探索

## 6. Verdict

**NOT PROVED — 但推理过程展示了真实的数学研究能力。** Agent 正确识别了 3-adic 方法 (路径 C) 和乘积分解方法 (路径 B), 但无法完成立方剩余分析。这是 LLM 数学推理能力的当前上限, 不是系统设计问题。
