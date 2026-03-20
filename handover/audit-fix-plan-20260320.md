# Gemini Lean 4 全面审计修复计划 (v2 — Gemini PLAN 审计后修正)

**日期**: 2026-03-20
**触发**: Gemini -p Lean 4 专家审计发现 4 CRITICAL + 3 HIGH + 4 MEDIUM
**修复范围**: experiments/ 层 + src/sdk/ 层（不触碰 kernel.rs 纯数学逻辑）
**PLAN 审计**: Gemini 第二轮审计 FAIL，4 个 blocking 项已修正（见下方 ⚡ 标记）

---

## 红线合规预检

| 红线 | 状态 |
|------|------|
| kernel.rs 纯数学逻辑 | **零改动** — 所有修复在 SKILL/sandbox/experiment 层 |
| bus.rs SKILL 生命周期钩子 | **零改动** — bus.rs 的 append/tool chain 逻辑不变 |
| WAL 文件/实验数据 | **零删除** |
| git push | 需人工确认 |

---

## 修复清单

### C2 — sorry 欺骗绕过 OMEGA [CRITICAL]

**问题**: LLM 输出的 payload 若包含 `sorry`，二次验证（无 sorry 编译）只产生 warning 不产生 error，`!contains("error:")` 为 true → 假 OMEGA。

**SPEC** ⚡ (Gemini 修正：全局扫描，非仅 tactic 区域):
- 在 `Lean4MembraneTool::on_pre_append` 中（Identity Theft 检查之后），增加 sorry 拦截
- 用 `\bsorry\b` 和 `\bsorryAx\b` 正则匹配全局 payload（不限于 `:= by` 之后）
- sorry 既是 tactic 也是 term（`exact sorry`, `let a := sorry`），必须全局拦截
- 使用 word boundary 避免误伤 `sorry_lemma` 等合法标识符
- 匹配则直接 `ToolSignal::Veto("sorry/sorryAx is forbidden")`
- **同步修复**: minif2f_swarm + zeta_regularization 两侧

**改动文件**:
- `experiments/minif2f_swarm/src/lean4_membrane_tool.rs`
- `experiments/zeta_regularization/src/lean4_membrane_tool.rs`

---

### C3 — 子进程泄露 (Timeout 未 kill) [CRITICAL]

**问题**: `sandbox.rs:81` Timeout 分支未 kill 子进程。`child` 被 move 到等待线程，主线程无法 kill。

**SPEC** ⚡ (Gemini 修正：process_group 是前提条件，非备选):
- **前提**：spawn 时必须使用 `CommandExt::process_group(0)` 创建独立进程组
- 在 spawn 后立即提取 `child.id()` 保存 PID
- Timeout 分支使用 `libc::kill(-(pid as i32), libc::SIGKILL)` 发送 SIGKILL 到进程组
- 必须先创建独立进程组，否则负 PID kill 会杀死主进程自身
- macOS 和 Linux 均支持 `process_group(0)`（POSIX 标准）
- Disconnected 分支同样需要 kill

**改动文件**:
- `src/sdk/sandbox.rs`

---

### C1 — RCE 沙箱逃逸 [CRITICAL, 部分修复]

**问题**: Lean 4 的 `#eval`、`#check`、`run_tac` 等可执行任意 IO。无容器化。

**SPEC — 阶段 1（本次修复，软防御）**:
- 在 `Lean4MembraneTool::on_pre_append` 中增加危险 Lean 4 关键字黑名单检查
- 黑名单：`#eval`, `#check`, `#reduce`, `#exec`, `native_decide`, `IO.Process`, `IO.FS`, `System.FilePath`, `run_tac`, `unsafe`, `import` (在 tactic 区域出现的动态 import)
- 匹配则 VETO
- **注意**：这是 LLM-level 防御，不是系统级隔离。恶意精心构造的 payload 可能绕过。完整容器化隔离（nsjail/bwrap）作为后续独立任务。

**改动文件**:
- `experiments/minif2f_swarm/src/lean4_membrane_tool.rs`
- `experiments/zeta_regularization/src/lean4_membrane_tool.rs`

---

### C4 — Identity Theft 检测可绕过 [CRITICAL]

**问题**: 只拦截 `theorem` 关键字，LLM 可用 `lemma`/`def`/`example`/`instance` 绕过。

**SPEC** ⚡ (Gemini 修正：扩展元编程关键字):
- 扩展 `check_identity_theft` 的关键字集合：`["theorem ", "lemma ", "def ", "example ", "instance ", "abbrev ", "axiom ", "constant ", "class ", "structure ", "macro ", "syntax ", "elab "]`
- 对每个关键字，检查其后的名称是否是 `self.theorem_name`
- 如果发现任何非目标名称的定义 → 返回 true（检测到盗用）
- 特别注意 `axiom`（直接宣称公理可绕过所有证明）和 `macro`（替换解析树）

**改动文件**:
- `experiments/minif2f_swarm/src/lean4_membrane_tool.rs`
- `experiments/zeta_regularization/src/lean4_membrane_tool.rs`

---

### H1 — Boltzmann 路由可能退化为随机 [HIGH, 需验证]

**问题**: Gemini 指出 `intrinsic_reward` 可能为 0，导致 `0 * depth = 0`。但 WAL 数据显示 reward 为 100/500 等非零值。

**SPEC**:
- 验证 intrinsic_reward 数据流：`ToolSignal::Pass` → `bus.append(final_reward=0)` → `kernel.append_tape(reward=0)`
- 如果确认 `Pass` 路径 reward=0：赋予默认微小非零分数 `score = (intrinsic_reward + 0.01) * (1.0 + depth_alpha * d)`
- 避免纯加法破坏高 reward 节点（如 OMEGA=100B）的数量级关系
- 确保零 reward 节点仍有非零 score，深度权重生效

**改动文件**:
- `experiments/minif2f_swarm/src/swarm.rs` (minif2f 侧)
- `experiments/zeta_regularization/src/swarm.rs` (zeta 侧，如有同样公式)

---

### H2 — 硬编码缩进 [HIGH, 延迟]

**问题**: `format!("{}\n  {}", last_state, tactic)` 和 `format!("{}\n  sorry")` 的 2 空格硬编码。

**SPEC — 延迟到后续版本**:
- 这需要根本性重构 payload 拼接方式（解析当前缩进深度再动态对齐）
- 当前 MiniF2F 定理主体在 `by` 之后统一用 2 空格缩进，实际影响有限
- 标记为 KNOWN LIMITATION

---

### H3 — VC 投资导致 Head 幽灵节点 [HIGH]

**问题**: `InvestOnly` 走 `bus.rs:162 return Ok(())` 但外层循环仍执行 `current_head.paths.insert(action.file_id)`。

**SPEC** ⚡ (Gemini 修正：用结构化标志位替代字符串匹配):
- 问题在 `full_test_evaluator.rs:136`，不在 bus.rs
- 在 evaluator 中，当 `bus.append(file)` 返回 `Ok(())` 时，检查 Tape 中是否实际新增了该 file_id
- 用 `bus.kernel.tape.files.contains_key(&action.file_id)` 判断是否真正插入了节点
- 如果 Tape 中不存在该 ID（InvestOnly 路径不插入节点），跳过 Head 更新
- 这比字符串匹配更可靠，直接查数据源

**改动文件**:
- `experiments/minif2f_swarm/src/bin/full_test_evaluator.rs`

---

### M2 — Anti-Zombie 短循环检测 [MEDIUM]

**问题**: 只检测 A→A→A，不检测 A→B→A→B。

**SPEC**:
- 在 `AntiZombiePruningTool::on_pre_append` 中增加窗口检测
- 保留最近 6 条 tactic，检查是否存在周期 ≤ 3 的循环模式
- 例如：检查 `tactics[i] == tactics[i-2]` 且 `tactics[i-1] == tactics[i-3]`（周期 2）

**改动文件**:
- `src/sdk/tool.rs`

---

### M3 — Tactic 提取尾随空格 [MEDIUM]

**问题**: `[Tactic: exact h] `（尾随空格）导致 `ends_with("]")` 失败。

**SPEC**:
- 在 `swarm.rs` 的 tactic 提取逻辑中，对 `pure_state` 先 `.trim()` 再检查格式
- 在 `membrane.rs` 的 `distill_pure_state` 中，对 `slice` 先 `.trim()` 再检查 `]`

**改动文件**:
- `experiments/minif2f_swarm/src/swarm.rs`
- `experiments/zeta_regularization/src/swarm.rs`
- `src/sdk/membrane.rs`

---

## 不修复项

| 编号 | 原因 |
|------|------|
| H2 (硬编码缩进) | 需根本性重构，当前 MiniF2F 影响有限 |
| M1 (Mac 路径硬编码) | 已知，运行在 Mac Studio 上是正确行为 |
| M4 (lake 冷启动) | 架构限制，需 Lean REPL 常驻进程，独立任务 |

---

## 实施顺序

1. **C2** — sorry 拦截（最高优先，直接影响 OMEGA 正确性）
2. **C3** — 子进程 kill（防止 OOM 崩溃）
3. **C1** — 危险关键字黑名单（软安全层）
4. **C4** — Identity Theft 扩展
5. **H1** — Boltzmann 公式修复
6. **H3** — Head 幽灵节点
7. **M2** — Anti-Zombie 短循环
8. **M3** — 尾随空格容错

## 验证计划

1. `cargo check` 通过所有 crate
2. `cargo test` 通过
3. 调用 Gemini -p 对完整 Lean 代码再次审计确认
