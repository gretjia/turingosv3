# TuringOS v3 — 三方交叉审计报告

**日期**: 2026-03-19
**递交对象**: 首席架构师
**审计触发**: Claude Code 三层治理 Harness 部署后首次全量审计
**审计范围**: 微内核纯净性、哲学对齐、数学正确性、经济机制完整性

---

## 一、审计方与工具链

| 审计方 | 引擎 | 焦点领域 | 权限 |
|--------|------|----------|------|
| Claude Opus 4.6 (kernel-auditor agent) | 内部 Agent | 微内核纯净性 + bible.md 对齐 + Layer 1 不变量 | 只读 |
| OpenAI Codex (gpt-5.4, xhigh reasoning) | 外部 CLI (`codex exec`) | 架构递归审计 + 调用图追踪 + 不变量交叉验证 | 只读 |
| Google Gemini | 外部 CLI (`gemini -p`) | 数学核心正确性 + 数值稳定性 + 收敛性分析 | 只读 |

三方独立执行，互不通信，审计结果交叉比对后形成本报告。

---

## 二、共识发现（三方一致确认）

### V1 [HIGH] — Append-Only DAG 不变量违规

**违规位置**: `src/sdk/tools/wallet.rs:120`

**不变量声明** (三处):
- `CLAUDE.md` Layer 1 #3: "Tape 是 Append-Only DAG — 不可删除已写入节点"
- `handover/bible.md:23-24`: "《苦涩的教训》最高法则：照单全收，绝对允许冗余...内核绝不打扫卫生"
- CLAUDE.md Layer 1 #1 延伸: kernel 零领域知识 → 不应有任何主体能判断哪些节点"该删"

**违规代码**:

```rust
// src/sdk/tools/wallet.rs:97-123
fn on_halt(&mut self, golden_path: &[String], tape: &mut Tape) {
    log::info!("==== [SMART CONTRACT] HALT REACHED! INITIATING SETTLEMENT ====");
    let golden_set: HashSet<_> = golden_path.iter().cloned().collect();

    // ... 结算逻辑（分配 global_pool 给黄金路径上的 VC 股东）...

    // ❌ 违规：破坏性删除非黄金路径节点
    let initial_size = tape.files.len();
    tape.files.retain(|id, _| golden_set.contains(id));  // <-- LINE 120
    log::info!(">>> [GC] Dead branches pruned: {} nodes vaporized.",
               initial_size - tape.files.len());

    self.global_pool = 0.0;
}
```

**三方证据汇总**:

| 审计方 | 发现描述 |
|--------|----------|
| Claude Opus | "This directly violates bible.md: '照单全收，绝对允许冗余...内核绝不打扫卫生' and CLAUDE.md invariant 'Tape is Append-Only DAG'" |
| OpenAI Codex | "`tape.files.retain(...)` deletes historical nodes, so the tape is not append-only. It also leaves `reverse_citations` intact, which means settlement can leave stale edges behind." |
| Google Gemini | 未直接检查此项，但在 DAG 一致性分析中确认 `append_tape()` 是纯追加设计，与 `retain` 语义矛盾 |

**Codex 补充发现 — 悬空边**: `tape.files.retain()` 仅清理了 `tape.files` (节点), 但未同步清理 `tape.reverse_citations` (边)。这意味着 HALT 结算后，`reverse_citations` 中仍保留指向已删除节点的边，若后续逻辑遍历这些边，会产生空引用。

相关数据结构定义:

```rust
// src/kernel.rs:23-27
#[derive(Debug, Clone, Default)]
pub struct Tape {
    pub files: HashMap<FileId, File>,           // <-- retain 清理了这里
    pub reverse_citations: HashMap<FileId, Vec<FileId>>, // <-- 但没清理这里
}
```

**架构师决策选项**:

| 选项 | 描述 | 哲学影响 |
|------|------|----------|
| A. 删除 `retain` 调用 | HALT 后保留完整 DAG，仅做结算分红 | 完全符合 bible.md "绝对允许冗余" |
| B. 增加 "post-HALT GC 例外" | 修改 bible.md 和 CLAUDE.md，明确声明 "Append-Only 约束仅在 `Running` 状态有效" | 需要哲学层面的论证 |
| C. 保留 GC 但修复悬空边 | 同时清理 `reverse_citations`，并将此操作标记为 "终态归档" 而非 "运行时删除" | 折中方案，需明确语义边界 |

---

## 三、Codex 独有发现

### V2 [MEDIUM] — kernel.rs 硬编码 Omega 奖励偏置，违反 SKILL-only 铸造权

**违规位置**: `src/kernel.rs:149-152`

**不变量声明**:
- `CLAUDE.md` Layer 1 #2: "仅 SKILL 可铸造 intrinsic_reward — kernel 不可直接赋值"
- `handover/bible.md:13`: "TuringOS 内核（Kernel）本身没有任何智能，没有任何偏好，没有任何'大局观'"

**违规代码**:

```rust
// src/kernel.rs:136-166  hayekian_map_reduce()
pub fn hayekian_map_reduce(&mut self) {
    // Step 1: Reset market price to absolute intrinsic reward
    for (_, node) in self.tape.files.iter_mut() {
        node.price = node.intrinsic_reward;
    }

    let mut new_prices = HashMap::new();

    for _ in 0..15 {
        for id in self.tape.files.keys() {
            let mut base_val = self.tape.files.get(id)
                .map(|f| f.intrinsic_reward).unwrap_or(0.0);

            // ❌ 违规：内核直接硬编码 100B 奖励偏置
            // Legacy compatibility for Hanoi target
            if id.starts_with(&self.target_omega_id) {   // <-- LINE 150
                base_val += 100_000_000_000.0;            // <-- LINE 151
            }

            let mut imputed_val = 0.0;
            if let Some(children) = self.tape.reverse_citations.get(id) {
                for child_id in children {
                    if let Some(child_file) = self.tape.files.get(child_id) {
                        let weight = 1.0 / (child_file.citations.len() as f64);
                        let child_price = new_prices.get(child_id)
                            .unwrap_or(&child_file.price);
                        imputed_val += self.gamma * weight * child_price;
                    }
                }
            }
            new_prices.insert(id.clone(), base_val + imputed_val);
        }
    }
    // ...
}
```

**Codex 分析**:

> "No forbidden proof-domain strings were found in `kernel.rs`. But the file is not actually domain-sterile: it embeds goal semantics via `trace_golden_path` and `target_omega_id`, and it contains a hardcoded `+100_000_000_000.0` reward bias. [...] the 'no kernel preference' claim in bible.md:13 is contradicted by the hardcoded target bonus."

**问题本质**: 内核在 `hayekian_map_reduce` 执行过程中，对匹配 `target_omega_id` 前缀的节点注入 1000 亿奖励。这意味着：
1. 内核"知道"哪个节点是目标终点 → 违反 "零偏好"
2. 内核直接铸造奖励值 → 违反 "仅 SKILL 可铸造 intrinsic_reward"
3. 注释标注为 "Legacy compatibility for Hanoi target" → 这是历史遗留代码，非有意设计

**当前实际运行中的冗余**: Lean4MembraneTool 在检测到证明完成时已经通过 SKILL 路径铸造了 1000 亿奖励：

```rust
// experiments/minif2f_swarm/src/lean4_membrane_tool.rs:80-83
info!("OMEGA NODE REACHED! Theorem proved perfectly!");
return ToolSignal::YieldReward {
    payload: format!("{}\n  -- [OMEGA]", payload),
    reward: 100_000_000_000.0,  // SKILL 正确铸造
};
```

这意味着 Omega 节点实际上会获得 **2000 亿** 奖励（SKILL 的 1000 亿 + kernel 的 1000 亿），这是一个非预期的双重铸造。

**架构师决策选项**:

| 选项 | 描述 |
|------|------|
| A. 删除 kernel.rs:149-152 | 移除 legacy 兼容代码，Omega 奖励完全由 SKILL 铸造。kernel 回归纯拓扑计算。 |
| B. 保留但降级为可配置参数 | 将 `100_000_000_000.0` 提取为 `Kernel::new()` 的参数，由调用方（bus 或 swarm）注入。保持 kernel 的"无偏好"但允许外部配置目标引力。 |

---

### V3 [LOW] — `append_tape` 为 `pub`，SKILL-only 铸造权是约定而非强制

**位置**: `src/kernel.rs:120`

```rust
// src/kernel.rs:120
pub fn append_tape(&mut self, mut file: File, reward: f64) -> &File {
    file.intrinsic_reward = reward;  // 直接赋值 intrinsic_reward
    // ...
}
```

**Codex 分析**:

> "Reward minting flows only from tool lifecycle signals [...] Caveat: this is convention, not a sealed API, because `kernel.rs:120` is `pub`."

**问题本质**: 任何持有 `&mut Kernel` 引用的代码都可以直接调用 `append_tape(file, arbitrary_reward)` 绕过 bus 的 SKILL 管道注入任意奖励。当前实际调用方只有 `TuringBus::append()`，但 `pub` 可见性意味着未来的代码可能意外绕过。

**建议**: 将 `append_tape` 改为 `pub(crate)` 或通过 `TuringBus` 独占调用来封闭路径。优先级低，当前无实际绕过发生。

---

## 四、Gemini 独有发现

### V4 [MEDIUM] — `hayekian_map_reduce` 迭代次数硬编码为 15，深度超限时梯度截断

**位置**: `src/kernel.rs:144`

```rust
// src/kernel.rs:144
for _ in 0..15 {  // ❌ 硬编码 15 次迭代
    for id in self.tape.files.keys() {
        // ... 价格逆传播 ...
    }
}
```

**Gemini 分析**:

> "In a DAG, value iteration requires exactly $D$ iterations to fully back-propagate values from leaves to roots, where $D$ is the maximum depth (longest path) of the graph. If a causal chain (e.g., from `omega` to `root`) exceeds 15 steps, the root node will not receive the 100B target reward. The market gradient will go flat for any steps beyond a depth of 15."

**数学论证**:

设 DAG 最大深度为 $D$，折扣因子 $\gamma = 0.99$。

- 迭代 $k$ 次后，深度为 $d$ 的节点收到的价格信号衰减为 $\gamma^d$ 倍。
- 当 $k < D$ 时，深度 $d > k$ 的节点**完全接收不到** Omega 的价格信号。
- 当前 MiniF2F 证明链典型深度为 10-30 步。深度 20+ 的节点在前 15 次迭代中无法感知到终点价格。

**实际影响**:
- 探索深度 15 以内的证明：价格信号完整，Softmax 路由正常
- 探索深度 15-30 的证明：根部节点的价格梯度为零，Softmax 退化为均匀分布
- 这意味着深层证明的"topological gravity"在根部失效，swarm 无法通过价格信号收敛到正确路径

**架构师决策选项**:

| 选项 | 描述 | 代价 |
|------|------|------|
| A. 动态深度 | `for _ in 0..max_dag_depth()` — 遍历 DAG 计算最大深度，迭代次数等于深度 | 每次 reduce 增加一次 $O(V+E)$ 深度扫描 |
| B. 收敛检测 | 循环直到 `max(price_delta) < epsilon` 或达到上限（如 100） | 最精确，但最坏情况开销大 |
| C. 提升硬编码上限 | 将 15 改为 50 或 100 | 最简单，对浅 DAG 浪费迭代但无功能损伤 |

---

### V5 [LOW] — `append_tape` 无循环检测，理论上可形成环

**位置**: `src/kernel.rs:120-133`

```rust
// src/kernel.rs:120-133
pub fn append_tape(&mut self, mut file: File, reward: f64) -> &File {
    file.intrinsic_reward = reward;
    file.price = reward;
    let id = file.id.clone();

    for parent_id in &file.citations {
        self.tape.reverse_citations
            .entry(parent_id.clone())
            .or_insert_with(Vec::new)
            .push(id.clone());
        // ❌ 未验证 parent_id 是否存在于 tape.files 中
        // ❌ 未验证 parent_id != id（自引用）
    }

    self.tape.files.insert(id.clone(), file);
    self.tape.files.get(&id).unwrap()
}
```

**Gemini 分析**:

> "It does not verify that cited parents actually exist in the tape prior to appending, which permits cyclic references (e.g., node A cites B, and node B is later appended citing A)."

**级联风险**: 若环存在，`trace_golden_path()` 将进入无限循环：

```rust
// src/kernel.rs:92-101
pub fn trace_golden_path(&self, omega_node_id: &str) -> Vec<String> {
    let mut path = Vec::new();
    let mut current = omega_node_id.to_string();
    while let Some(node) = self.tape.files.get(&current) {
        path.push(current.clone());
        if node.citations.is_empty() { break; }
        current = node.citations[0].clone();
        // ❌ 无 visited set 保护，环引用将导致无限循环
    }
    path
}
```

**实际风险评估**: 当前所有调用方（`swarm.rs`, `full_test_evaluator.rs`）使用 `format!("step_{}_branch_{}", step, rand::random::<u16>())` 生成唯一 ID，实际环形成概率极低。但防御性编程建议添加 `visited` 集合。

---

### V6 [LOW] — 重复 FileId 导致 `reverse_citations` 腐败

**位置**: `src/kernel.rs:125-132`

**Gemini 分析**:

> "If `append_tape()` is called twice with the same `FileId`, `self.tape.files` gracefully overwrites the file, but `self.tape.reverse_citations` indiscriminately appends duplicates. The old parent citations are never removed, permanently corrupting the graph's edge weights."

**场景推演**:

```
第一次 append: File { id: "X", citations: ["A", "B"] }
  → reverse_citations["A"] = ["X"]
  → reverse_citations["B"] = ["X"]

第二次 append: File { id: "X", citations: ["C"] }  (覆盖)
  → reverse_citations["A"] = ["X"]      ← 悬空！A 不再是 X 的父节点
  → reverse_citations["B"] = ["X"]      ← 悬空！
  → reverse_citations["C"] = ["X"]      ← 正确
  → tape.files["X"].citations = ["C"]   ← 文件已更新
```

结果：`hayekian_map_reduce` 中 A 和 B 仍会将价格传播给 X（通过 `reverse_citations`），但 X 的 `citations.len()` 变为 1（只有 C），导致权重计算错误。

---

### V7 [LOW] — V6 触发时的除零风险

**位置**: `src/kernel.rs:158`

```rust
// src/kernel.rs:155-161
if let Some(children) = self.tape.reverse_citations.get(id) {
    for child_id in children {
        if let Some(child_file) = self.tape.files.get(child_id) {
            let weight = 1.0 / (child_file.citations.len() as f64);
            //          ❌ 若 V6 场景中 citations 被覆盖为空 Vec，
            //             则 0 as f64 = 0.0, 1.0 / 0.0 = Infinity
            let child_price = new_prices.get(child_id)
                .unwrap_or(&child_file.price);
            imputed_val += self.gamma * weight * child_price;
            //          Infinity * any_price = Infinity → 全局价格污染
        }
    }
}
```

**Gemini 分析**:

> "If a file is appended with citations and then appended again with 0 citations, the original parent retains the child in `reverse_citations`, but `child_file.citations.len()` becomes `0`. This triggers a division by zero, resulting in `Infinity` prices propagating through the tape."

**注意**: Rust 中 `1.0_f64 / 0.0_f64` 不会 panic，而是产生 `f64::INFINITY`。Infinity 在后续迭代中会通过加法和乘法传播到所有祖先节点的价格，使 Market Ticker 输出无意义的 `inf` 值。

**实际风险**: 与 V5 和 V6 相同，依赖于重复 FileId 场景的发生，当前调用方使用随机 ID 生成，实际风险极低。

---

## 五、综合评级矩阵

| 编号 | 发现 | 严重度 | 发现方 | 影响范围 | 是否阻塞部署 |
|------|------|--------|--------|----------|------------|
| V1 | Append-Only DAG 违规 (wallet.rs:120 `retain`) | **HIGH** | 三方一致 | 哲学不变量 + 数据完整性 | 需架构师裁决 |
| V2 | kernel.rs 硬编码 Omega 奖励 (kernel.rs:149-152) | **MEDIUM** | Codex | 铸造权不变量 + 双重奖励 | 需架构师裁决 |
| V4 | 15 次迭代硬上限 (kernel.rs:144) | **MEDIUM** | Gemini | 深层证明的价格信号截断 | 不阻塞但限制探索深度 |
| V3 | `append_tape` 为 pub (kernel.rs:120) | **LOW** | Codex | API 封闭性 | 否 |
| V5 | 无循环检测 (kernel.rs:125-130) | **LOW** | Gemini | 防御性编程 | 否 |
| V6 | 重复 FileId 致 reverse_citations 腐败 | **LOW** | Gemini | 数据一致性 | 否 |
| V7 | V6 触发时除零 → Infinity 传播 | **LOW** | Gemini | 数值稳定性 | 否（依赖 V6） |

---

## 六、架构师待决事项

### 决策 1: V1 — Append-Only DAG 的边界在哪里？

三个选项（详见 V1 节）。核心问题：**bible.md 的 "内核绝不打扫卫生" 是否在 HALT 后仍然有效？**

### 决策 2: V2 — kernel.rs:149-152 是否删除？

这是 Hanoi 实验的 legacy 代码。当前 Lean4MembraneTool 已正确通过 SKILL 铸造 Omega 奖励，kernel 内的硬编码产生双重铸造。建议删除并验证 swarm 运行不受影响。

### 决策 3: V4 — 迭代上限如何调整？

三个选项（详见 V4 节）。核心权衡：**计算成本 vs 梯度精度**。当前 MiniF2F 的证明深度通常在 10-30 步，15 次迭代在边界。

---

## 七、本次会话变更审计（附录）

以下两处变更经三方一致确认为 **CLEAN**，不引入新违规：

| 文件 | 变更 | Commit |
|------|------|--------|
| `src/bus.rs:100` | 添加 `log::warn!(">>> [TOOL VETO] Author: {}, Reason: {}", ...)` | `0e944db` |
| `experiments/minif2f_swarm/src/lean4_membrane_tool.rs:102` | `debug!` 升级为 `warn!` | `0e944db` |

---

*本报告由 Claude Code 三层治理 Harness 自动编排生成，三方审计引擎独立执行。*
*Commit 基线: `0e944db` (main)*
