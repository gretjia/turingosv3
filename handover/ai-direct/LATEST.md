# TuringOS v3 — Handover State
**Updated**: 2026-03-28
**Session Summary**: Magna Carta vFinal 全面对齐 + minif2f_v2 实验 (Lean 4 Oracle) + AIME 2025 首证 + 强制投资轮经济突破

## Current State
- **经济引擎: Turing-Polymarket vFinal** — 三层纵深防御 (内核黑名单 + SKILL 安全 + Prompt), Oracle ONLY at OMEGA, 免费 append (Law 1)
- **minif2f_v2 实验**: Lean 4 Oracle + Polymarket + Engine 4 (per-agent skills). Mac (zephrymac-studio) 运行
- **AIME 2025 I P1: PROVED** — 首次在训练数据外新题上通过 Lean 4 形式化验证 (Run 4)
- **AIME 1983 P1: PROVED** — Lean 4 验证, 10 分钟内完成
- **zeta_sum_proof Run 9: OMEGA** — 强制投资轮, 7 市场, 310 Coins 流通, 完整经济循环
- **Lean 4 + Mathlib**: omega-vm (7849 jobs built) + Mac (6996 olean) 双机就绪
- **NVIDIA NIM API**: key 已存 .env, 测试通过, 40 RPM 限速, 备用

## Changes This Session

### 架构级
- `580a0f2` — **Oracle ONLY at OMEGA** — 中间 append 免费, Oracle 不拦截 (Law 1 + 苦涩教训)
- `29a2595` — **废除所有创世后印钞** — fund_agent/redistribute_pool/global_pool 删除
- `1c7bb12` — **内核级暴力搜索黑名单** — bus.rs Phase 0 封锁 decide/omega/native_decide
- `4c073ca` — **Conservation invariant check** — halt_and_settle 前后 Coins 总量验证
- `cfabcf1` — #print 注入封锁 + global_pool 字段删除
- `5bf661d` — **强制投资轮** — 每回合结尾额外 LLM 调用, 强制经济参与

### minif2f_v2 实验
- `577422f` — Engine 4 (per-agent skills) + OMEGA nonce 防注入 + Veto 退款
- `3969166` — 错误反馈循环 + search-first prompt + 300s timeout
- `bb669c5` — UTF-8 安全截断 (Lean 输出含 Unicode)
- `86ef7e2` — LEAN_PATH 自动发现所有 Lake packages
- `51a541e` — Strategy Guide prompt (引导投资, 不强制)
- `b7e0bcc` — 2025 AIME I P1 问题 (post-training-cutoff)
- `e4f1db9` — 问题形式化重写 (∀ b:ℕ 替代 Finset.range, CLAUDE.md #21)
- `fd3134b` — ban decide + omega (SKILL 层)
- `31e25ca` — load_problem 自动检测暴力搜索空间

### 审计报告
- `e88c839` — AIME 2025 P1 数学审计 (DAG 树状图)
- `2c400d3` — AIME 2025 P1 双重审计 (math VALID + economics first convergence)
- `f1f063c` — Run 9 双重审计 (math VALID + economics breakthrough)

### CLAUDE.md 规则
- `ef61142` — #20 苦涩的教训: 禁止 Over-Alignment
- `e4f1db9` — #21 形式化不可引入暴力搜索空间

## Key Decisions
- **Oracle 从守门员变为信息源**: 中间步骤自由上链 (Law 1), Oracle 只在 [COMPLETE] 时编译全证明链
- **三层纵深防御**: 内核黑名单 (bus.rs) > SKILL 安全 (Lean4Oracle) > Prompt 引导
- **强制投资轮**: 每轮行动后追加投资专属 LLM 调用, 保证经济活动
- **ban decide/omega**: 暴力搜索不是构造性证明, 从内核+SKILL+Prompt 三层封锁
- **问题形式化原则**: 用 ∀ 全称量词而非 Finset.range, 逼迫构造性推理
- **Conservation check**: halt_and_settle 前后验证系统 Coins 总量

## Next Steps
1. **修复 Conservation 计量**: compute_total_system_coins 需要追踪 market-locked funds
2. **做空涌现**: 0/9 次运行出现 SHORT — 可能需要在强制投资轮中增加做空引导
3. **AIME 2025 P1 重跑**: 新形式化 (∀ b:ℕ, 无 Finset.range) + 内核黑名单, 需要构造性证明
4. **AIME 2025 P15**: 3-adic 数论, 极难
5. **MiniF2F v2 批量测试**: 244 题系统性测试 (需要 batch evaluator)
6. **Engine 4 深化**: per-agent skills 已实现文件系统, 需要验证 autopsy/victory 效果

## Warnings
- **Conservation drift**: halt_and_settle 报 drift = 投入额 (计量 bug, 非真实铸币)
- **SiliconFlow R1 API**: 5 个 Agent 401 失效
- **做空未涌现**: 9 次运行, 0 SHORT — LLM 建设性偏差 + 无负面价格信号
- **Lean 4 版本**: minif2f_data_lean4 用 v4.24.0, elan stable 是 v4.29.0 — 需要显式 LEAN_CMD 路径
- **omega-vm Mathlib**: 已完整构建 (7849 jobs), 可作为备用运行环境
- **3 个非活跃实验 100B 残留**: minif2f_swarm, number_theory_min, zeta_regularization 的 lean4_membrane_tool.rs

## Architect Insights (本次会话)
- **刺客的远征**: "空头利润锁死→被迫建设→推动 OMEGA" → 已归档 `2026-03-27_assassins-expedition.md`
- **零成本拓扑**: "建树零成本, 投资独立, 拓扑金融彻底剥离" → 已归档 `2026-03-27_zero-cost-topology.md`
- 架构师指令归档:
  - `directives/2026-03-27_polymarket-final-four-laws.md` (四大宪法级物理法则)
