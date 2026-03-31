# TuringOS v3 — Handover State
**Updated**: 2026-03-31
**Session Summary**: 做空实验四部曲 (Run 11-14) — 从强制到尊重，发现"人性激发"prompt 最优；AIME P15 运行中

## Current State
- **zeta_sum_proof: PROVED (4 runs)** — 1+2+3+...=-1/12 正则化证明，Gemini 数学审计 10/10
- **AIME 2025 I P15: RUNNING** — 470/1000 tx, Step 13 (3-adic case 分支计数), 未达 OMEGA
- **AIME 2025 I P1: PROVED** — Lean 4 验证 (不变)
- **做空机制: 已验证最优方案** — "尊重自主" prompt (Run 14 哲学) 产生 Drift=249 (四轮最低)
- **Hook 脚本: 已修复** — 改为 git rev-parse 动态路径，可移植

## Changes This Session
- `89bf932` — **Run 11 zeta_sum_proof + Gemini 双审计**
  - OMEGA: 85 tx, 4 步 Golden Path, Gemini math 10/10, econ 6-8-9/10
  - evaluator: 移除 SF R1 client, DEEPSEEK_API_KEY 改 required
  - Hook 脚本: 3 个 .sh + settings.json 从硬编码路径改为 `git rev-parse --show-toplevel`
- `9b02874` — **Run 11-14 做空实验四部曲**
  - Run 12 (强制做空): 29 SHORT/14 INVEST, Drift=2392 — 中央计划失败
  - Run 13 (利润诱导): 68 SHORT/0 INVEST, Drift=18969 — 矫枉过正
  - Run 14 (尊重自主): 10 SHORT/41 INVEST, Drift=249 — 最优方案
  - SKILL LAW 5: "TWO SACRED DUTIES OF A MATHEMATICIAN: To BUILD / To SCRUTINIZE"
  - Forced invest: "You are a mathematician reviewing proof steps"
  - 归档架构师洞察: zero-shorting-explicit-prompt.md
- *(uncommitted)* — **minif2f_v2 evaluator prompt 对齐 Run 14 哲学**
  - SKILL LAW 4: invest→"endorse", short→"challenge"
  - Strategy Guide: 加入 "TWO SACRED DUTIES" 段落
  - Falsifier learned.md: 从 "PROFIT by betting against" 改为 "sacred duties"
  - Forced invest prompt: 统一为 "mathematician reviewing proof steps"，移除 is_falsifier 分支

## Key Decisions
- **做空不可强制也不可利诱**: LLM 对 prompt 措辞极度敏感。Run 12-13 证明：强制→假数据，利诱→全做空。唯一正确方案是激发内生活力。
- **"尊重生命"prompt 设计原则**: 把 agent 当数学家而非交易员。告诉他们"审查是崇高职责"而非"做空很赚钱"。
- **Drift 是经济健康的核心指标**: Run 14 的 Drift=249 远低于 Run 11 的 349，说明多空自然平衡反而让经济更稳定。

## Architect Insights (本次会话)
- **空头需要内生活力激发，不能靠强制或利诱**: 4 轮对照实验验证 → `handover/architect-insights/2026-03-31_zero-shorting-explicit-prompt.md`

## Next Steps
1. **AIME P15 run 完成后**: 审计数学进展 + 经济数据，commit 结果
2. **[OPEN SPRINT] bus.rs: 强制投资 + `<step>` 防抢跑** — 上次会话遗留，待实现
3. **MiniF2F 批量测试**: 244 题 baseline

## Warnings
- **W1**: AIME P15 当前 run 后台进行中 (bmjk2d69r)，勿 kill evaluator 进程
- **W2**: Qwen3-32B (SiliconFlow) 产出较低 (21/470 tx)，可能需要排查延迟或格式兼容性
- **W3**: minif2f_v2 evaluator.rs 有未 commit 的改动 (Run 14 哲学移植)
- **W4**: bus.rs 强制投资 + `<step>` 防抢跑仍未实现 (上次会话 W1 延续)

## Audit Trail
- `experiments/zeta_sum_proof/audit/run2_math_audit_gemini.md` — Gemini 数学审计全文
- `experiments/zeta_sum_proof/audit/run2_econ_audit_gemini.md` — Gemini 经济审计全文 (含哈耶克/中本聪/德鲁克点评)
- Run 11-14 对照数据见 commit `9b02874` message
