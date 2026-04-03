# TuringOS v3 — Handover State
**Updated**: 2026-04-03
**Session Summary**: AutoResearch v4 运行 16 实验 (best ERS=0.058, depth=5) + Win1 NSSM 永久修复 + Living Harness 部署

## Current State
- **AutoResearch v4: RUNNING** (PID 718134) — Phase 1 Prompt Search
  - 16 实验完成, 2 次 KEEP, best ERS=0.05847 (depth=5)
  - DeepSeek V3 持续编辑 problem.txt, 当前 prompt 包含三路径 zeta 证明详细脚手架
  - TSV: `experiments/zeta_sum_proof/audit/autoresearch_v4.tsv`
- **本地推理双节点**:
  - Mac (18080): llama.cpp Qwen3.5-9B, --parallel 2, ~33 tok/s ✓ 稳定
  - Win1 (18081): **NSSM Windows 服务**, Vulkan GPU, 永久存活 ✓ **已修复**
- **Cron 监控**: 每 30 min 自动检测 + 重启 (`monitor.sh`)
- **Living Harness**: 12 条规则 + 宪法检查器 + 3 hook + 2 skill 已部署

## Changes This Session

### Committed
- `cc16921` feat: AutoResearch v4 + 6 architecture upgrades + dual local inference
- `393474c` feat: Living Harness — self-improving constitutional guardian

### Uncommitted (5 files, +69/-10)
- **sweep_v4.py**: ERS depth 计算修复 — Librarian STATS 日志为权威来源
- **librarian.rs**: 添加 `info!(">>> [LIBRARIAN] STATS: ...")` 输出 depth 到 evaluator 日志
- **prompt/problem.txt**: DeepSeek 搜索代理自主编辑的最新版本
- **windows1_llama_server_rootcause.md**: NSSM 解决方案文档化
- **monitor.sh**: NSSM 服务模式替代 nohup ssh

### Bug Fixes
1. **ERS 恒为 0**: LIBRARIAN_INTERVAL=999 禁用了 Librarian → 无 depth 数据 → ERS=0
   - Fix: LIBRARIAN_INTERVAL=20, Librarian 打 depth 到日志
2. **Win1 SSH 断开后死亡**: 裸 llama-server.exe 无法在 Session 0 存活
   - 根因: v1 用 Ollama (自带守护), v2 不用 Win1, v3 直接用裸 binary
   - Fix: NSSM + SERVICE_INTERACTIVE_PROCESS → Vulkan GPU 可用 + 永久存活

### Violation Found & Fixed
- **Ground Truth 违规**: LIBRARIAN_INTERVAL=999 在 sweep 中 bypass 了 Librarian → 违反 "log = Ground Truth" 指令
  - Fix: LIBRARIAN_INTERVAL=20 (必须在实验预算内触发)

## Key Decisions
- **Karpathy 核心对齐**: sweep_v3 (random mutation) → sweep_v4 (DeepSeek IS the search)
- **Prompt 为可变工件**: problem.txt/skill.txt/context.txt 从编译时 const → 运行时文件加载
- **固定 600s wall clock**: 不再按 thinking 模式变 timeout — comparable experiments
- **NSSM 取代 nohup ssh**: Windows 服务是 Win1 的正确持久化方案
- **Ollama 对 Qwen3.5 不兼容**: 架构师确认, 必须用 llama.cpp

## Architect Insights (本次会话)
- **根因分析强制令**: 禁止 "可能是", 必须找根因 → fix → document → improve harness
- 已归档到 memory: `feedback_root_cause_mandate.md`
（其余洞察在上一 session 已归档）

## AutoResearch Progress
| 指标 | 数值 |
|------|------|
| 总实验 | 16 |
| Best ERS | 0.05847 (depth=5, Exp 10) |
| KEEP | 2 次 |
| ERS=0 | ~40% (Librarian 未触发或 appends 太少) |
| 频率 | ~6/小时 |

DeepSeek 搜索策略: 持续编辑 problem.txt, 从泛泛要求逐步优化到三路径 + 分步计算详细脚手架。Phase 1 仍在进行。

## Research Plan
见 `experiments/zeta_sum_proof/RESEARCH_PLAN.md`
- **Phase 1 (当前)**: Prompt Search — 目标 depth > 10
- Phase 2: Thinking Mode sweep (off/on/budget)
- Phase 3: Economic mechanism tuning
- Phase 4: 27B model on Win1

## Living Harness Deployed (2026-04-03)
- **12 条数据驱动规则** (rules/active/*.yaml): 5 block + 7 warn
- **6 个完整违规 trace** (incidents/): V-001~V-006
- **宪法对齐检查器** (scripts/constitutional_check.sh): 10 项检查, 当前 PASS
- **3 个新 hook**: rule-engine.sh + pipeline-quality-gate.sh + post-lesson-trigger.sh
- **2 个新 skill**: /lesson-to-rule + /harness-reflect
- **架构文档**: LIVING_HARNESS.md
- 全部 hook 语法验证通过, 规则引擎 5 场景烟测通过

## Next Steps
1. **继续 AutoResearch v4 过夜** — Cron + NSSM 确保稳定性
2. **[OPEN SPRINT] 提交未 commit 的 5 个文件**
3. **Phase 2 启动条件**: depth > 10 的 prompt 稳定后
4. **Win1 Thinking 问题**: NSSM 服务中 Qwen3.5 thinking 太慢, 需要 evaluator 的 /no_think 通过 API 生效
5. **首次 /harness-reflect**: 建立 Harness Health Score 基线

## Warnings
- **Qwen3.5 thinking**: 本地 llama.cpp 不支持 `enable_thinking: false` API 参数, 只能靠 system prompt `/no_think`. 在 NSSM 服务中可能行为不一致
- **ERS=0 频率 40%**: 部分 DeepSeek prompt 编辑导致 appends < 20 → Librarian 不触发 → depth=0. 可能需要 LIBRARIAN_INTERVAL 进一步降低
- **SSH 隧道仍需重建**: NSSM 服务永久运行但 omega-vm→Win1 的 SSH 隧道在 omega-vm 重启后需重建. Cron monitor.sh 处理此问题
