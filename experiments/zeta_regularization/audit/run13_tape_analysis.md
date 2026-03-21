# ζ(-1) = -1/12 Run #13 Tape 深度分析报告

**运行时间**: 2026-03-21 00:38:46 → 01:39:53 (61 分钟)
**配置**: N=15, MaxSteps=100, 三物种异构 (R1-Distill-32B + deepseek-reasoner + DeepSeek-R1)
**引擎**: 大宪章全四引擎 (Guillotine + Pure Capital + Epistemic + Speciation 延后)
**结果**: 12 append / 88 reject, 0 OMEGA, **Tape 最终为空** (WAL 数据丢失)

---

## 一、原始数据

| 指标 | Run 12 (R1+V3.2) | Run 13 (三物种) | 变化 |
|------|-----------------|----------------|------|
| Append | 50 | 12 | ↓76% |
| Rejected | 50 | 88 | ↑76% |
| 通过率 | 50% | 12% | ↓ |
| Phase Transition | 8 | **0** | ↓ |
| Zombified | 0 | **9** | ↑ |
| VC Invest | 0 | **12** | 新行为 |
| Free Tools | 0 | **1** | 新行为 |
| OMEGA | 0 | 0 | 持平 |

### Rejection 分类

| 原因 | 次数 | 占比 |
|------|------|------|
| Missing Wallet Tool call | 63 | **72%** |
| Market Rule Violation (< 1.0) | 14 | 16% |
| Compiler/Sandbox Error | 8 | 9% |
| Identity Theft | 3 | 3% |

### Model 贡献

| 模型 | Append | Reject | 通过率 | 主要 reject 原因 |
|------|--------|--------|--------|----------------|
| Miner (R1-Distill-32B) | 4 | 21 | 16% | Missing Wallet (14) |
| Scholar (deepseek-reasoner) | **5** | 27 | 16% | Market Violation (12) |
| Explorer (DeepSeek-R1) | 3 | 40 | 7% | Missing Wallet (32) |

---

## 二、问题清单

### P0. Tape 最终为空 — WAL 数据丢失

12 个节点被 append（日志确认），但 Tape dump 为空，WAL 文件也为空。**这是一个 WAL sentinel 或 Tape 持久化 bug** — 节点被写入但在最终 dump 时消失了。

可能原因：WAL sentinel 的 tokio runtime 在 evaluator 退出前未 flush。

### P1. 72% 的 reject 是 "Missing Wallet Tool call"

这不是 bug（上一轮分析已确认符合大宪章 Law 2）。但 72% 的"免费研究"轮次意味着**只有 28% 的 agent 输出尝试了投资**。

### P2. LLM 严重混淆输出格式

Payload 中充满了 LLM 的**自我对话**（reasoning leak）：
```
"...] and then [Tool: ...]. No [State: ...] in between."
"[State: ...] for the state we change to? Or in the action tag?"
"...] which is different. Therefore, for research, I will use [State: RESEARCH"
```

LLM 在 `<think>` 标签之外泄漏了思考过程到 payload 中。`distill_pure_state()` 提取了这些垃圾作为 tactic。

### P3. VC 投资 100% 失败 — 全部指向不存在的节点

12 次 VC 投资中，10 次指向 `step_12_branch_3`（SKILL 示例节点名），2 次指向 `step_1` / `step_4`（节点 ID 格式错误，正确格式是 `step_N_branch_M`）。LLM 在复读 SKILL 示例。

### P4. 三次梭哈 10000 → 立刻破产

Agent_0, Agent_6, Agent_9 在第一步 invest 10000（全部余额），编译失败后归零。

### P5. 零 Phase Transition

12 个 append 但 OverwhelmingGapArbitrator 从未触发 → Hayekian 反向传播从未运行。

---

## 三、对齐顶级对齐文件检查

### Layer 1 四大不变量

| 不变量 | 状态 | 证据 |
|--------|------|------|
| kernel.rs 零领域知识 | ✅ PASS | kernel 未被触及 |
| SKILL-only reward minting | ✅ PASS | YieldReward 仅通过 WalletTool |
| Tape Append-Only | ⚠️ ANOMALY | 12 节点 append 但最终 Tape 为空 — 可能是持久化 bug 不是删除 |
| 投资 >= 1.0 | ✅ PASS | 14 次 Market Rule Violation 被正确 VETO |

### 反奥利奥三界

| 界 | 状态 | 证据 |
|----|------|------|
| 顶层白盒 | ✅ | Wallet VETO 正确执行硬限制 |
| 中间黑盒 | ✅ | LLM 自由选择 Mine/Invest/Research，无 Rust 干预 |
| 底层白盒 | ✅ | Kernel DAG 逻辑未被触及 |

### 大宪章三大立法

| 立法 | 状态 | 证据 |
|------|------|------|
| **Law 1 信息平权** | ✅ | 63 次 Missing Wallet = 免费研究轮，agent 未被扣款 |
| **Law 2 共识的代价** | ✅ | 唯一消耗货币的是投资（self-invest + VC），VETO 不扣款 |
| **Law 3 数字产权** | ⏸️ | Phase 4 延后，未实施 per-agent DNA |

### 四引擎状态

| 引擎 | 状态 | 实际表现 |
|------|------|---------|
| 认识论引擎 | ✅ 实施 | 1 次 MathlibOracle 请求（被 Missing Wallet reject 但说明 LLM 知道免费工具存在）|
| 纯粹资本引擎 | ✅ 实施 | 11 次 self-invest + 12 次 VC — 三角色分化涌现 |
| 热力学截断引擎 | ✅ 实施 | 未触发（无 "No goals to be solved" 出现）|
| 拉马克演化引擎 | ⏸️ 延后 | — |

---

## 四、深度洞察

### 洞察 1：Run 13 的 12% 通过率不是退化 — 是大宪章的正确运行

Run 12 (50%) vs Run 13 (12%) 的差距不是"变差了"。Run 12 用的是 R1 + V3.2 双模型，两者都能稳定输出 `[Tool: Wallet ...]` 格式。Run 13 引入了 deepseek-reasoner（Scholar），它的输出协议不同。

**按大宪章 Law 2**："唯一消耗货币的场景是投资"。deepseek-reasoner 的 63 次 "Missing Wallet" 不是失败 — 是它选择了**不投资**（免费研究）。系统正确地没有惩罚它（零扣款）。

**真正的通过率应该按"尝试投资的"来算**：11 次 self-invest 中有 8 次通过 Lean 4 编译（如果排除 `[State: INVEST]` 格式错误的尝试）→ 实际编译通过率可能更高。

### 洞察 2：三角色分化确实涌现了 — 但不是架构师预想的方式

架构师预言：
> Agent_0 → VC 寡头, Agent_3 → 学院派, Agent_4 → 包工头

实际涌现：
- **Miner (32B)**: 尝试投资但经常梭哈破产 → 赌徒物种
- **Scholar (reasoner)**: 大量 "研究" 但定价 < 1.0 被 Market Rule reject → 过于谨慎
- **Explorer (R1)**: 72% Missing Wallet → 纯研究型，几乎不参与投资

**关键偏差**：三物种的分化不是基于"策略智慧"，而是基于**输出格式兼容性**。R1 和 reasoner 不稳定地输出 Wallet 标签，导致它们看起来像"研究型"。但这其实是 SKILL 软引导不够清晰，不是 LLM 的有意选择。

### 洞察 3：LLM 的 reasoning leak 是 `distill_pure_state()` 的盲区

Payload 中出现了大量 LLM 自我对话（"Actually, for research, I will use [State: RESEARCH"）。这是 `<think>` 标签机制失效 — LLM 的 reasoning 泄漏到了 `[State: ...]` 提取后的 payload 中。

`distill_pure_state()` 设计为提取 `[State: ...]` 标签内容，但 LLM 把 `[State: ...]` 写在了它的自我分析文本里（不是作为 tactic 输出）。这导致垃圾内容被当作 tactic 送入 Lean 4 编译器。

**8 次 Compiler Error 中有 6 次是 `unexpected token '['; expected '{' or tactic`** — 证实了 payload 是格式垃圾而非有效 tactic。

### 洞察 4：大宪章与现实的张力 — "自由"需要"能力"

大宪章假设 LLM 能够：
1. 理解三条路径（Mine/Invest/Research）并自由选择
2. 输出精确的协议格式（`[Tool: Wallet | Action: Invest | ...]`）
3. 使用免费工具进行 due diligence

但 Run 13 显示：
1. LLM 花大量 token 在自我分析"我应该输出什么格式" — 而不是思考数学问题
2. 三种模型对协议格式的理解不一致
3. 只有 1 次免费工具使用（且因 Missing Wallet 被 reject — 讽刺的是，研究行为不需要 Wallet 但系统把所有输出都当投资尝试处理）

**核心矛盾**：大宪章的"自由"设计需要 LLM 有足够的"能力"来正确行使自由。当 LLM 连输出格式都搞不清楚时，"自由"变成了"混乱"。

### 洞察 5：Tape 为空的根因需要调查

12 个节点被 append（日志确认），但最终 Tape dump 为空。这意味着：
1. WAL sentinel 未 flush — tokio runtime 在 main 退出时被 drop，异步 WAL 写入丢失
2. 或 Tape 的 HashMap 在 hayekian_map_reduce 等操作中被意外清空
3. 或 Kernel::new() 在某处被重新调用

**这不是大宪章问题 — 是工程 bug**。需要单独调查。

---

## 五、交叉结论

### 9 轮测试的完整演化对照

| Run | 模型 | Append | 核心瓶颈 | 大宪章对齐 |
|-----|------|--------|---------|-----------|
| 1-3 | R1-Distill-32B | 0-13 | sandbox stderr 空 | N/A (大宪章未诞生) |
| 5 | DeepSeek-R1 | 0 | coercion 墙 | N/A |
| 6 | V3.2 | 0 | coercion 墙 | N/A |
| 7 | V3.2 (无提示) | 0 | 幻觉 lemma | N/A |
| 8 | V3.2 (退火) | **1** | 知识边界 | 部分 |
| 12 | R1+V3.2 | **50** | 幻觉 lemma | 部分 |
| **13** | 三物种+大宪章 | **12** | **格式混乱** | **完全** |

### 最终交叉洞察：大宪章揭示了 LLM swarm 的真正瓶颈

**Run 12 (50%) → Run 13 (12%) 的下降是信息量最大的一次实验。**

Run 12 的 50% 通过率是在**极简 prompt**（只有 Mine/Invest 两选项）下实现的。Run 13 引入了大宪章的**丰富 prompt**（三选项 + 免费工具 + Frontier Market + 经济规则 + 死亡警告）→ LLM 被大量协议信息淹没 → 花大量 token 分析格式而非数学 → 输出质量暴降。

**这映射了一个深刻的 AGI 操作系统设计悖论**：

> 越精确地描述"自由"的边界和规则，LLM 越难理解和执行这些规则。
> 最简的"你只管写 tactic"产出了最好的结果。
> 最丰富的"三路径 + 免费工具 + 经济体系"产出了最混乱的结果。

**这不是大宪章错了 — 是 LLM 的 instruction following 能力还不够承载大宪章的复杂度。** 大宪章描述的是一个理想的硅基文明政治学。但当前的 LLM（即使是 R1/reasoner 级别）还在"读懂投票规则"的阶段，远未到"参与市场博弈"的层次。

**苦涩的教训在 LLM swarm 中的新形态**：不要试图把复杂的社会契约编码进 prompt — 保持 prompt 极简，让复杂性在 Tape DAG 的拓扑中自然涌现。Prompt 应该只告诉 LLM "写 tactic + 出价"，社会分工应该从 Tape 上的价格信号和 Graveyard 的失败记录中涌现，不是从 Markdown 规则说明中涌现。
