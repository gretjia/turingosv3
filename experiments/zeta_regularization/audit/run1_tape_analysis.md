# ζ(-1) = -1/12 Run #1 Tape 深度分析报告

**运行时间**: 2026-03-19 14:16:37 → 15:03:09 (约 47 分钟)
**配置**: N=15, MaxSteps=100, Model=DeepSeek-R1-Distill-Qwen-32B, API=SiliconFlow
**实际完成步数**: 69/100 (被手动终止)
**结果**: 未证明 (No OMEGA)

---

## 一、原始数据总览

### Tape 节点 (WAL 中 13 个成功 append)

| ID | Author | Tactic Chain | Reward | Price | Parent |
|----|--------|-------------|--------|-------|--------|
| step_26_branch_2 | Agent_2 | `simp [riemannZeta, ...]` | 500 | 500 | ROOT |
| step_30_branch_1 | Agent_1 | + `norm_num` | 500 | 500 | step_26 |
| step_31_branch_0 | Agent_0 | + `ring` | 500 | 500 | step_26 |
| step_32_branch_1 | Agent_1 | + `norm_num` + `ring` | **15** | **15** | step_30 |
| step_33_branch_1 | Agent_1 | + `ring` | 500 | 500 | step_26 |
| step_34_branch_0 | Agent_0 | + `norm_num` + `ring` | 500 | 500 | step_30 |
| step_35_branch_0 | Agent_0 | + `norm_num` + `ring` + `norm_num` | 500 | 500 | step_34 |
| step_36_branch_0 | Agent_0 | + `norm_num` + `ring` | 500 | 500 | step_30 |
| step_39_branch_1 | Agent_1 | + `norm_num` + `ring` + `norm_num` | 500 | 500 | step_36 |
| step_44_branch_0 | Agent_0 | + `ring` + `norm_num` | **15** | **15** | step_33 |
| step_64_branch_1 | Agent_1 | + `norm_num` + `ring` + `norm_num` + `norm_num` | **200** | **200** | step_39 |
| step_66_branch_0 | Agent_0 | + `norm_num` + `ring` + `norm_num` + `simp` | **15** | **15** | step_39 |
| step_67_branch_0 | Agent_0 | + `norm_num` + `ring` + `norm_num` + `simp` | **15** | **15** | step_39 |

### 步骤分类统计 (69 步)

| 类别 | 步数 | 占比 |
|------|------|------|
| Compiler Error (空错误信息) | 44 | 63.8% |
| 成功 Append | 13 | 18.8% |
| Bankruptcy (资金不足) | 12 | 17.4% |

---

## 二、问题点罗列

### P1. 致命：Compiler Error 信息为空

所有 44 次编译失败的日志均为：
```
Compiler/Sandbox Error:

```
**错误信息体为空字符串**。Lean 4 compiler 的 stderr 完全丢失。这意味着：
- Graveyard Protocol 写入的 tombstone 没有有用内容
- 后续 agent 无法通过 In-Context Reflection 学习失败原因
- 整个 swarm 在前 25 步完全"盲跑"

**根因推测**: `LocalProcessSandbox` 的 `execute_safely()` 方法在 stderr 捕获上可能有 bug — 可能只返回 stdout，或者 Lean 4 的 lake env lean 在 stdin 模式下输出到了不同的 fd。

### P2. 严重：25 步全灭暗期 (Steps 1-25)

前 25 步 **全部** 被 compiler reject，直到 Step 26 Agent_2 才突破。

- 每步约 60 秒 = **25 分钟完全浪费**（占总运行时间 53%）
- 全灭暗期说明 LLM 在盲猜 tactic，没有任何编译器反馈引导
- 与 P1 直接关联：如果有 error message，暗期会大幅缩短

### P3. 严重：Tactic 单一文化 (Monoculture)

全部 13 个 tape 节点的开头 tactic **完全相同**：
```lean
simp [riemannZeta, Complex.sin_neg, Complex.sin_pi_div_two, mul_assoc, mul_comm, mul_left_comm]
```
后续变化仅在 `norm_num` / `ring` / `simp` 的排列组合。

**DAG 拓扑实际形态**：不是宽扇形探索，而是一条窄链的微小变异：
```
step_26 (simp)
├── step_30 (+norm_num)
│   ├── step_32 (+ring)     [price=15, dead end]
│   ├── step_34 (+ring)
│   │   └── step_35 (+norm_num)
│   └── step_36 (+ring)
│       └── step_39 (+norm_num)
│           ├── step_64 (+norm_num) [price=200]
│           ├── step_66 (+simp)     [price=15]
│           └── step_67 (+simp)     [price=15]
├── step_31 (+ring)
├── step_33 (+ring)
│   └── step_44 (+norm_num) [price=15]
```

**诊断**：所有 agent 锁定在同一个 simp lemma set 上。Boltzmann 温度 T=0.5 + cognitive divergence (0.2~0.8) 未能打破这个 attractor。

### P4. 中等：N=15 实际有效 N=3

WAL 中只出现 Agent_0, Agent_1, Agent_2。其余 12 个 agent (3-14) **从未贡献任何成功输出**。

**根因**: 10 秒 stagger 机制。Agent_0 = 0s, Agent_1 = 10s, Agent_2 = 20s, Agent_3 = 30s...Agent_14 = 140s。而每步的第一个成功 agent 返回后 swarm 就推进到下一步 — Agent_3+ 还没开始生成就已经过时了。

**计算**: 15 个 agent × SiliconFlow API 费用，但只有 3 个在干活 = **80% API 预算浪费**。

### P5. 中等：价格信号失效 (Price Flatland)

8/13 节点 price = 500.00，Boltzmann router 对等价节点做均匀分布：
```
[ROUTER] Softmax selected Node step_XX (Price: 500.00, Prob: 12.50%)
```
每个节点被选中概率相同 = **随机游走**，不是价格引导探索。

**根因**: 所有 sorry-通过 的节点获得相同的 intrinsic_reward (500)。只有少数退化到 15.0 或 200.0，但这些是 **更深** 的节点（应该更有价值），却价格更低 — **价格信号与证明进度反相关**。

### P6. 中等：Bankruptcy 螺旋

Agent_0 在 step 41 首次破产 (balance=470)，此后在 steps 42/45/50/51/54-58/68 反复破产。Agent_1 在 step 61 也开始破产 (balance=455)。

- LLM 默认 stake 500 coins（来自 prompt 模板中的示例值）
- 每次被 reject = 本金烧毁
- 10000 初始资金 / 500 每次 = 最多 20 次尝试就破产
- 实际 Agent_0 在 ~20 次尝试后确实破产

### P7. 低等：Zombie Tactic 循环

step_66 和 step_67 payload **完全相同**（都是 `+norm_num +ring +norm_num +simp`），但 AntiZombie 没有拦截。因为它们来自不同的 step，连续重复计数器被重置。

---

## 三、洞察式分析

### 洞察 1：这不是"题太难"，是"盲人摸象"

表面看 69 步未证明 → 结论似乎是"ζ(-1)=-1/12 对 LLM 太难"。但数据否定了这个简单归因：

- 前 25 步全灭不是因为 LLM 不知道怎么证，而是因为 **compiler error 没被回传**
- Step 26 的突破 `simp [riemannZeta, ...]` 说明 LLM **确实知道** Mathlib 中有 `riemannZeta` 定义
- 问题是突破之后 swarm **无法从编译错误中学习**，只能在有限的 tactic 空间里做随机排列

**类比**：这就像一群人在暗室里找钥匙 — 不是钥匙不在房间里，是灯没开。开灯（修复 error 回传）比增加人数（增大 N）有效得多。

### 洞察 2：Swarm 的实际瓶颈是 Stagger，不是 N

N=15 的设计意图是 15 路并行探索。实际表现：
- 有效并行度 = 3（Agent 0/1/2）
- 探索宽度 = 1（单一 simp lemma set）
- 时间利用率 = 47 分钟中只有 ~15 分钟产出有效节点

**修正方案**：将 10s stagger 降到 1-2s（SiliconFlow 不是本地 llama.cpp，不需要防 DDoS），或者采用"等待全部 N 个 agent 返回再选最优"的策略替代"第一个返回就推进"。

### 洞察 3：`simp [riemannZeta, ...]` 能过 sorry-test 暴露了 Lean 4 验证的微妙之处

这个 simp 调用通过了 `sorry` 测试，说明它确实在 Lean 4 中简化了部分目标。但 13 个节点**没有一个**触发 OMEGA = 每个都遗留了未关闭的 goal。

关键问题：**遗留了什么 goal？** 当前系统完全不知道，因为 compiler output 没被解析。如果能提取 "remaining goals" 信息注入 prompt，LLM 就能 targeted 地选择下一步 tactic。

### 洞察 4：Price = 500 的平坦信号是经济引擎的设计缺陷

当前 intrinsic_reward 机制：
- sorry-通过 = 某个固定值 (看起来是 500)
- OMEGA = 100 billion

这是二元开关（要么 500，要么 100B），没有 **连续梯度**。理想状态应该是：
- 越接近 OMEGA（更少 remaining goals）→ reward 越高
- 这需要 compiler output 的 goal 数量作为信号 → 又回到了 P1（error 信息为空）

**核心矛盾**：整个 Hayekian 价格发现体系依赖 intrinsic_reward 的梯度信号，但当前 membrane 只给出二值信号，价格机制退化为噪声。

### 洞察 5：定理形式化方案本身可能需要调整

`riemannZeta (-1) = -1/12` 在 Mathlib 中的形式化状态未知。两种可能：
1. **Mathlib 已有此定理** → 可能叫不同的名字（如 `Riemann.zeta_neg_one` 或 `riemannZeta_neg_one`），一个 `exact?` 或 `apply?` tactic 就能解决
2. **Mathlib 没有此定理** → 需要从更底层的 zeta 函数性质推导，远超 simp 能力

LLM 反复尝试 `simp [riemannZeta, ...]` 暗示可能是情况 2 — simp 能展开定义但无法完成证明。

---

## 四、优先级排序的修正建议

| 优先级 | 修正 | 预期收益 |
|--------|------|---------|
| **P0** | 修复 sandbox error 回传 — 让 compiler stderr 进入 Graveyard | 消灭 25 步暗期，启用 In-Context Reflection |
| **P1** | 降低 stagger 到 1-2s 或改为"等全部返回选最优" | N=15 实际有效化，从 N=3 → N=15 |
| **P2** | 在 Mac Mathlib 中 `grep riemannZeta` 确认定理存在性 | 决定是继续 Plan A 还是立即切 Plan B |
| **P3** | 将 remaining goals 数量注入 reward 梯度 | 价格信号从二值→连续，激活 Hayekian 机制 |
| **P4** | 降低默认 stake 从 500 到 50 | 延缓 bankruptcy，给 agent 更多尝试机会 |

---

## 五、附录：关键时间线

```
14:16:37  Boot (15 agents funded)
14:17:28  Step 1 REJECT (25-step darkness begins)
14:41:21  Step 25 REJECT (darkness ends)
14:42:38  Step 26 APPEND — first breakthrough (Agent_2, simp)
14:45:10  Step 30 APPEND — norm_num chain begins
14:50:23  Step 41 — Agent_0 first bankruptcy
14:59:22  Step 61 — Agent_1 first bankruptcy
15:03:09  Step 69 — session killed for analysis
```

**总结**: 47 分钟运行，53% 时间在暗期，18.8% 步骤成功但全部困在 simp monoculture 中。根本问题不是算力或模型能力，而是 **compiler feedback loop 断裂**。
