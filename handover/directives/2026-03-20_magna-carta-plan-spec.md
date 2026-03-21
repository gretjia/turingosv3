# PLAN & SPEC: TuringOS vFinal《大宪章》四大引擎实施方案

**Date**: 2026-03-21 (Final)
**Architect Directive**: `handover/directives/2026-03-20_magna-carta-vfinal.md`
**Audit**: Codex kernel-auditor — CONDITIONAL PASS → 4 条件已解决
**Expert Ruling**: Gemini (Lean 4 形式化证明专家) — Phase 1 最终裁决
**Anti-Oreo Compliance**: 所有数值由 LLM 自主决定，零硬编码

---

## Phase 1: 引擎三 — 热力学截断引擎 (Semantic Guillotine)
**优先级**: P0 — 最小变更，最大收益
**复杂度**: 低
**涉及文件**: `experiments/zeta_regularization/src/lean4_membrane_tool.rs`, `experiments/minif2f_swarm/src/lean4_membrane_tool.rs`
**决策状态**: ✅ 已裁决 (Gemini 最终裁决 + 对抗性讨论)

### Gemini 裁决要点
1. `"No goals to be solved"` = 所有 goals 已关闭（不是局部 sub-goal）— Lean 4 只有在绝对没有任何 goal 剩下时才抛此错误
2. Termination failure = Lean 4 kernel 拒绝 — 不算证明成功
3. 致命漏洞：LLM 可以用自己的 `sorry` 伪造 "No goals" → 需检查 `declaration uses 'sorry'` 警告
4. 我们的 sorry 防火墙（membrane lines 78-102）已在上游拦截，但 Gemini 的双重检查更安全

### 最终实施方案（三层防御）

**文件**: `lean4_membrane_tool.rs` Err 分支 (当前 lines 158-178)

替换为：
```rust
Err(e) => {
    if e.contains("No goals to be solved") {
        // Gemini ruling: check if "No goals" is the ONLY error
        // AND no sorry-cheating warning
        let has_other_errors = e.lines()
            .any(|l| l.contains("error:") && !l.contains("No goals to be solved"));
        let has_sorry_warning = e.contains("declaration uses 'sorry'");

        if !has_other_errors && !has_sorry_warning {
            // Safe OMEGA: sorry purely redundant, no other errors, no cheating
            info!("OMEGA (Guillotine): No goals + clean output");
            return ToolSignal::YieldReward {
                payload: format!("{}\n  -- [OMEGA]", payload),
                reward: 100_000_000_000.0,
            };
        } else {
            // Ambiguous: other errors or sorry warning → double-check
            if let Ok(omega_output) = self.sandbox.execute_safely(payload, gas_limit) {
                if !omega_output.contains("error:") {
                    info!("OMEGA (double-check verified in Err branch)");
                    return ToolSignal::YieldReward {
                        payload: format!("{}\n  -- [OMEGA]", payload),
                        reward: 100_000_000_000.0,
                    };
                }
            }
            warn!("Lean4 Membrane VETO: No goals but other errors present.");
        }
    }
    warn!("Lean4 Membrane VETO: Compiler rejected the tactic or timed out.");
    ToolSignal::Veto(format!("Compiler/Sandbox Error:\n{}", e))
}
```

**三层防御**:
1. Sorry 防火墙 (上游, lines 78-102): 拦截 LLM payload 中的 sorry/sorryAx
2. Gemini 条件检查 (中间): No goals 唯一 error + 无 sorry 警告 → 直接 OMEGA
3. Double-check (兜底): 有其他 error 时重新编译验证

---

## Phase 2: 引擎二 — 纯粹资本引擎 (Pure Capital Engine)
**优先级**: P1
**复杂度**: 中
**涉及文件**: `src/sdk/tools/wallet.rs`, `skills/economic_operative.md`, `experiments/zeta_regularization/src/swarm.rs`
**决策状态**: ✅ 审计通过

### 变更 2.1: Wallet 术语统一 "Stake" → "Invest"
**文件**: `src/sdk/tools/wallet.rs`
- `parse_payment()`: 同时识别 `Action: Stake` 和 `Action: Invest`（向后兼容）
- 新增识别 `Action: Invest` 作为首选格式
- 日志从 `SELF-STAKE` → `SELF-INVEST`
- **不改变任何经济逻辑** — 仅术语对齐

### 变更 2.2: SKILL 重写
**文件**: `skills/economic_operative.md`
- 统一术语："投资"替代"质押"
- 强调三角色自由选择：矿工 / VC / 包工头
- 免费工具预告（为 Phase 3 铺垫）
- 示例中使用 `Action: Invest`

### 变更 2.3: Prompt 中 "Stake" → "Invest"
**文件**: `experiments/zeta_regularization/src/swarm.rs`
- prompt 模板中 `Action: Stake` → `Action: Invest`
- 保持向后兼容（wallet 同时识别两种）

---

## Phase 3: 引擎一 — 认识论引擎 (Epistemic Engine)
**优先级**: P2
**复杂度**: 高
**涉及文件**: `experiments/zeta_regularization/src/swarm.rs`
**决策状态**: ✅ 审计通过，采用方案 A (Pre-prompt 注入)

### 架构决策：方案 A — Pre-prompt 注入
- MathlibOracle / PythonSandbox **不作为 TuringTool 挂载**
- 在 `swarm.rs` 的 prompt 构建阶段处理免费工具请求
- Agent 在上一轮输出中用 `[Tool: MathlibOracle | Query: ...]` 请求
- swarm 在下一轮 prompt 中注入查询结果
- **零 core 层变更**：不改 ToolSignal enum，不改 bus.rs

### 变更 3.1: MathlibOracle 免费工具
**实现位置**: `experiments/zeta_regularization/src/swarm.rs` (prompt 构建阶段)
- 解析上一轮输出中的 `[Tool: MathlibOracle | Query: <text>]`
- 用 `SandboxEngine` 执行 `grep -r "<sanitized_query>" .lake/packages/mathlib/`
- 将结果（限制前 20 行）注入下一轮 prompt
- **不消耗余额**，**不写入 Tape**
- 输入消毒：移除 shell 元字符 (`; | & $ \` 等)

### 变更 3.2: PythonSandbox 免费工具
**实现位置**: `experiments/zeta_regularization/src/swarm.rs` (prompt 构建阶段)
- 解析 `[Tool: PythonSandbox | Code: <python_code>]`
- 用 `LocalProcessSandbox` (cmd=`python3`, args=[`-c`, code]) 执行
- 超时 10s，输出限制 50 行
- 结果注入下一轮 prompt
- **不消耗余额**，**不写入 Tape**

### CLAUDE.md 注释 (审计条件 2)
Layer 1 不变量 #4 加注释：
```
4. **质押必须 >= 1.0** — 零成本操作被禁止（适用于 Tape 写入操作；纯信息查询类免费工具不受此约束）
```

---

## Phase 4: 引擎四 — 拉马克物种演化引擎 (Speciation Engine) [延后]
**优先级**: P3
**复杂度**: 高
**决策状态**: ⏸️ 审计师 + 用户共识：先验证 Phase 1-3 效果，再实施

### 延后原因
1. Per-agent LLM 调用（尸检钩子）增加系统复杂度和成本
2. dna.md 无限膨胀需要截断策略（Layer 2 参数）
3. Agent 身份在跨题场景下的生命周期不明确
4. Phase 1-3 已足够产生显著改进

### 保留设计
- `skills/agent_N/dna.md` 目录结构
- 破产尸检钩子 (Autopsy Mutation)
- 胜利复盘钩子 (Victory Reinforcement)
- 待 Phase 1-3 验证后重新评估

---

## 实施顺序总览

```
Phase 1 (Semantic Guillotine)     → 1 文件改动, ~20 行     [P0]
Phase 2 (Pure Capital)            → 3 文件改动, ~30 行     [P1]
Phase 3 (Epistemic Engine)        → 1 文件 + CLAUDE.md, ~100 行  [P2]
Phase 4 (Speciation)              → 延后                   [P3]
```

## Layer 1 合规验证

| 不变量 | Phase 1 | Phase 2 | Phase 3 | Phase 4 |
|--------|---------|---------|---------|---------|
| kernel.rs 零领域知识 | ✅ | ✅ | ✅ | ✅ |
| Append-Only DAG | ✅ | ✅ | ✅ (不 append) | ✅ |
| SKILL-only reward | ✅ | ✅ | ✅ | ✅ |
| 投资 >= 1.0 | ✅ | ✅ | N/A (免费, 已注释) | ✅ |

## Anti-Oreo 合规验证

| 检查项 | 状态 |
|--------|------|
| Rust 代码不干预 LLM 策略 | ✅ 全部 Phase |
| 数值由 LLM 自主决定 | ✅ Amount = `<FLOAT>` |
| SKILL (Markdown) 是唯一软引导 | ✅ |
| 架构师示例 (500/800/2000/5000) | 不实施 — 仅为解释 |

## 安全验证

| 检查项 | 状态 |
|--------|------|
| MathlibOracle 输入消毒 | ✅ Shell 元字符过滤 |
| PythonSandbox 隔离执行 | ✅ 复用 SandboxEngine + 超时 |
| Sorry-cheating 防御 | ✅ 三层防御 (防火墙 + Gemini 条件 + double-check) |
| RCE 防御 | ✅ 所有外部执行走 SandboxEngine |
