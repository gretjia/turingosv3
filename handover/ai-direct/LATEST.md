# TuringOS v3 — Handover State
**Updated**: 2026-03-26
**Session Summary**: 三代经济引擎进化 (Hayekian→AMM→Polymarket) + Run 5/6 实战验证 + harness 升级

## Current State
- **经济引擎: Turing-Polymarket** — 二元条件代币 CPMM (YES/NO)，Split-Ignition 点火，Oracle 二元清算
- **zeta_sum_proof Run 6: OMEGA** — 53 tx, 35 nodes, 6-step GP, 第三条独立代数路径 (对称双指数 z₁,z₂)
- **Run 5 (TuringSwap AMM): OMEGA** — 15 tx, 12 nodes, 3-step GP, Gemini 审计 VALID WITH MINOR GAPS
- **Layer 1 不变量**: 全部通过 (kernel-auditor CLEAN)
- **P0 Bug 已识别**: MathStepMembrane 100B OMEGA 铸币 (Hayekian 遗产) 破坏 Polymarket 零和守恒 — **尚未修复**
- **Harness 已升级**: dev-cycle Stage 4.5 MIGRATION SCAN 防止 experiments/ 兼容性遗漏

## Changes This Session
- `b187ba3` — Run 4 全局节点报告 (26 nodes, DAG 拓扑, 市场效率)
- `59d647a` — TuringSwap AMM 指令归档 + Gemini 审计 (REJECT) + 3 条架构师洞察
- `9ce1639` — **feat: TuringSwap AMM 经济引擎** — amm.rs, kernel 6 方法, bus 重写, Split-Ignition
- `28af785` — Run 5 TuringSwap 分析 (OMEGA 3 步, AMM 引用即买入验证)
- `ca989ac` — Run 5 Gemini 独立审计 (VALID WITH MINOR GAPS)
- `94f6ae1` — **feat: Turing-Polymarket** — 替换 AMM 为二元预测市场, YES/NO CPMM, Oracle 解算
- `88d8732` — Run 6 Polymarket 分析 (OMEGA 6 步, 第三条独立代数路径)
- `f182de7` — Run 6 经济学深度分析 (100B bug 发现, 五层做空缺失分析)
- `9471d75` — Run 6 对齐审查 (Layer 1 4/4, Polymarket 铁律 1 VIOLATED)
- `3e77554` — **fix: harness 升级** — Stage 4.5 MIGRATION SCAN + CLAUDE.md 规则 #16

## Key Decisions
- **三代经济引擎进化**: Hayekian (GOSPLAN 事后分配) → TuringSwap AMM (事前现货) → Polymarket (二元概率市场)
- **不回滚，原地重构**: AMM 引入的基础设施 (portfolios, as_any, bus helpers) 在 Polymarket 中复用
- **拓扑与金融解耦 (大宪章复辟)**: 引用零成本 (AMM 时代要付费)，金融行为独立
- **Split-Ignition 两步点火**: 1 Coin 中性 LP + 剩余 stake auto-long (架构师补丁)
- **Harness 根因修复**: CLAUDE.md #16 + dev-cycle Stage 4.5 防止 experiments/ 兼容性遗漏

## Next Steps
1. **P0: 修复 MathStepMembrane 100B 铸币** — 删除 YieldReward 100B，改为 Modify (零铸币)。覆盖全部 4 个实验 (16 处)
2. **P1: 暴露 short 动作 + 价格概率可见** — SKILL prompt 增加 short 选项，snapshot 显示 P_yes/P_no
3. **P1: 引导跨节点投资** — SKILL prompt 引导 "invest in promising nodes you didn't create"
4. **P2: Polymarket 零和悖论** — 保守投注 + 低 LP = 低利润率，需要架构师思考激励设计
5. **工程化测试**: MATH Benchmark L3 Number Theory + Terminal Oracle
6. **Terminal Oracle (Lean 4)**: [COMPLETE] 仍为字符串匹配，无数学验证

## Warnings
- **MathStepMembrane 100B 铸币 BUG 未修复** — experiments/ 中 16 处 100_000_000_000 仍在。下次 run 前必须修复
- **OMEGA 仍无数学验证**: [COMPLETE] 标签即触发，Lean 4 验证未实现
- **做空机制未激活**: 协议层无 short 动作，agent 看不到 P_yes/P_no，LLM 无攻击性偏向
- **价格信号退化**: 所有节点 P_yes≈1.0 (auto-long 后)，无差异化贝叶斯信号
- **SiliconFlow R1 API Key 失效**: 5 个 R1 agent 在 Run 5/6 中全部 401
- **hanoi_1m 测试预存 bug**: `run_turing_os` import 失效 (非本次引入)

## Architect Insights (本次会话)
- **GOSPLAN→AMM 范式转换**: "map_reduce 事后发钱 = 苏联国家计划委员会年终奖" → 已归档 `handover/architect-insights/2026-03-25_gosplan-to-amm.md`
- **引用即买入**: "没有免费的白嫖！Agent B 必须当场买入引用权" → 已归档 `2026-03-25_citation-as-spot-purchase.md`
- **Init AI 迭代引导**: "多次初始化，先试跑，让顶层看问题" → 已归档 `2026-03-25_init-ai-iterative-boot.md`
- **孤立热力学孤岛**: "每个节点是绝对物理隔离的热力学孤岛，资金绝对不跨池" → 已归档 `2026-03-26_polymarket-isolated-thermodynamic-islands.md`
- 架构师指令归档:
  - `directives/2026-03-25_turingswap-amm-economic-engine.md`
  - `directives/2026-03-26_turing-polymarket-prediction-engine.md`
  - `directives/2026-03-26_split-ignition-lp-model.md`
