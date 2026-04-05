# TuringOS v3 — Handover State
**Updated**: 2026-04-05
**Session Summary**: 宪法审计 + 11 项违规修复 + AutoResearch v5 (60 轮实验) + 模型规模筛查 Spec

## Current State
- **AutoResearch v5**: 已停止 (linux1)。60 轮快速迭代完成，depth 卡在 2-3。
- **宪法审计**: Codex exec 审计 26 个 Rust 文件，发现 13 个违规 → 修复 8 个 → 重审 7/7 PASS
- **模型规模筛查**: Spec 已写 (MODEL_SCALING_SPEC.md)，待开始
- **四节点**: linux1 (主力 parallel=20) + Mac (辅助 parallel=5) + Win1 (备用) + omega-vm (Git)
- **所有实验进程已停止**，等待模型筛查实验启动

## Changes This Session

### Constitutional Fixes (Codex 审计 → 修复 → 重审 PASS)
1. **wallet.rs**: Transfer ABOLISHED (Law 2) + 最低投资 1.0 移除 (Law 2)
2. **bus.rs**: conservation 修复 — CTF 公式 min(yes,no)+abs(yes-no)+lp
3. **kernel.rs**: ghost citation 从静默剥离改为 Err 回滚 (topology ∏p=0)
4. **librarian.rs**: 分化记忆 — 保留 agent 个人内容，只替换 LIBRARIAN MEMORY section
5. **evaluator.rs**: autopsy/victory 实时触发 + 插入到 LIBRARIAN MEMORY 之前避免被覆盖
6. **swarm.rs**: 删除死代码 + lib.rs 移除 mod
7. **skill.txt**: 补充被动信息源 (Librarian memory, market prices, rejection feedback, bulletin)
8. **problem.txt**: 精简为纯题目一行
9. **context.txt**: 移除透题
10. **evaluator.rs DEFAULT_PROBLEM/CONTEXT**: 移除完整 MATHEMATICAL TOOLKIT 透题

### AutoResearch v5 进展
- 60 轮快速迭代 (300s/轮)
- 2 个 KEEP: exp52 (MATH_COUNT=8, depth=3) + exp56 (DEPTH_WEIGHT=50, depth=3)
- 核心发现: 9B 模型不足以在纯市场经济 (0/0/0) 下有效参与
- Researcher 角色: DeepSeek Reasoner (researcher) + DeepSeek Chat (auditor)
- research_notes.txt 与 context.txt 分离 (agent 不可见 researcher 笔记)

### Committed
- `1fe3980` feat: AutoResearch v5
- `9efed5a` fix: Gemini audit false positive + TypeError crash + linux1 cron monitor

## Key Decisions
- **0/0/0 是宪法正确的**: FRONTIER_CAP=0, DEPTH_WEIGHT=0, PRICE_GATE_ALPHA=0
- **9B 模型经济制度失灵**: 需要模型规模筛查
- **Transfer 废除**: 违反 Law 2
- **swarm.rs 删除**: 死代码
- **Librarian 是 topology map-reduce**: LIBRARIAN_INTERVAL=8 暂定
- **Prompt 纯净化**: problem.txt 只有题目，skill.txt 只有工具说明，context.txt 不透题

## Architect Insights (本次会话)
- **群体即思考机器** → `2026-04-04_swarm-is-the-thinker.md`
- **PRICE_GATE_ALPHA>0 = bug** → `2026-04-04_price-gate-alpha-bug.md`
- **黑盒加 log_lib + Reasoner 加 re-init** → `2026-04-04_blackbox-tools-expansion.md`
- **re-init 前宪法对齐** → `2026-04-04_reinit-constitutional-guard.md`
- **9B 经济制度失灵** — 本 session 实验发现

## Next Steps
1. **[IMMEDIATE] 写 model_scaling_sweep.py 并启动模型规模筛查**
   - Qwen 0.6B → 397B MoE 全系列
   - 纯市场设置 (0/0/0)，THINKING_MODE=off
   - Spec: `experiments/zeta_sum_proof/MODEL_SCALING_SPEC.md`
2. **[OPEN SPRINT] bus.rs tick_map_reduce 重构**
3. **[OPEN SPRINT] math_membrane.rs 语义断头台对齐**
4. **[OPEN SPRINT] log_lib agent 工具**

## Warnings
- **linux1 只能通过 Mac 跳板**: `ssh zephrymac-studio "ssh zepher@192.168.3.113 '...'"`
- **linux1 llama-server**: `-c 65536 --parallel 20 -fa on`
- **LIBRARIAN_INTERVAL=8 暂定** — 见 memory `project_librarian_interval.md`
- **sweep_v4.py 部署需 patch 路径**: omega-vm `/home/zephryj/...` → linux1 `/home/zepher/...`
- **Gemini API 走 Mac proxy**: `192.168.3.93:7897`
