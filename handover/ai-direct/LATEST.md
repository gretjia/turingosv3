# TuringOS v3 — Handover State
**Updated**: 2026-03-20
**Session Summary**: ζ(-1)=-1/12 正则化实验 — 从零搭建到异构 R1+V3.2 swarm，8 轮迭代验证多 agent 涌现

## Current State
- **zeta_regularization 实验项目** (`experiments/zeta_regularization/`) 完全独立于 minif2f
- **Run 12 已完成**: 50 append / 50 reject (50% 通过率), 0 OMEGA, 4 tape 节点存活
- **Mac Studio tmux**: `zeta-test` session 已结束, `minif2f-sota-run` 由另一个 agent 管理
- **异构 swarm 验证成功**: R1 (8 agents, key_primary) + V3.2 (7 agents, key_secondary)
- **Hayekian 价格引擎首次全速运转**: 8 次 Phase Transition, 最高价格 65,400
- **定理未证明**: LLM 无法独立发现 `riemannZeta_neg_nat_eq_bernoulli'` (Mathlib 中存在)

## Changes This Session
- `08d0094` — zeta_regularization 独立实验项目 (Cargo.toml, evaluator, swarm, WAL, membrane)
- `294b546` — Big Bang Multiverse sync + port to zeta
- `557f433` — 热力学退火 + 多行 tactic + Graveyard dedup (3→10)
- `122ef57` — 异构 swarm (R1+V3.2) + depth-weighted frontier + 双 API key 路由
- `llm_http.rs` — `ResilientLLMClient::with_key()` 显式 key 构造 + `model_name()` accessor
- `sandbox.rs` — **根因修复**: Lean 4 error 写到 stdout 非 stderr, 合并两流
- `bus.rs` — Graveyard dedup (相同错误不重复) + `extract_wallet_balances()`
- 另一个 agent 的变更已同步: `fb26dfb` frontier ticker, `1ae1d2c` pool redistribution, `a6bc24c` depth-weighted, `a5a89be` SelfHeal 修复

## Key Decisions
- **项目隔离**: 每个实验独立 Cargo 项目, 不共享 API 配置 (反教训: Run 1 混在 minif2f 里)
- **不 hard-code 答案**: 删除了 lemma 名提示, 让 swarm 自己发现 (Run 7 vs Run 6 对比验证)
- **热力学退火**: 物理温度参数 (顶层白盒), 非策略干预 (中间黑盒), 符合反奥利奥架构
- **异构优于同构**: R1 知道精确 lemma 名, V3.2 知道定义展开路径, 互补覆盖

## 8 轮测试演化链
| Run | 模型 | Append | 关键发现 |
|-----|------|--------|---------|
| 1 | R1-Distill-32B | 13 | sandbox error 空, N=3 |
| 3 | R1-Distill-32B | 0 | sandbox 修复, 50% termination trap |
| 5 | DeepSeek-R1 | 0 | R1 找到正确 lemma 名, coercion 墙 |
| 6 | V3.2 | 0 | 18 种策略, coercion 墙 |
| 7 | V3.2 (无提示) | 0 | 37 种策略, 幻觉 lemma 主导 |
| 8 | V3.2 (退火) | 1 | 首次 append! 45 种策略 |
| 12 | R1+V3.2 异构 | **50** | 50% 通过率, 价格 65,400 |

## Next Steps
1. **混合更多模型** (Qwen3.5 等) 扩大知识覆盖 — 突破 Mathlib API 名称盲区
2. **尝试不同定理** 验证 swarm 架构的通用性 (不仅限于 ζ 函数)
3. **分析 Run 12 DAG 拓扑** — 50 个节点的证明树结构是否有收敛趋势
4. **minif2f 全量测试重跑** — OMEGA 检测修复后的真实 pass rate

## Warnings
- `zeta/src/harness.rs` + `zeta/src/swarm.rs` 有未提交的 fb26dfb port + SelfHeal 修复
- `.env` 中 `SILICONFLOW_API_KEY_SECONDARY` 和 `VOLCENGINE_API_KEY` 粘连 — tmux 启动时必须手动分割 (正确 key: `sk-vhmaluxrfdqqnpjududaptmvhjasrervmetppawlwwbbpwya`)
- Mac 的 `.env` 没有 SECONDARY key — 必须在 tmux 命令中显式 export
- 审计报告在 `experiments/zeta_regularization/audit/` (run1, run3, run6, run7, run8)

## Architect Insights (本次会话)
- **退火的真正价值是温度梯度** — 低温 agent 的保守策略 (`simp only`) 存活为高温 agent 提供 DAG 立足点 → 已归档到 audit/run8_tape_analysis.md
- **异构涌现需要共享 Tape** — R1 的 lemma 知识 + V3.2 的展开能力只有通过 Tape DAG 才能互相可见 → 已归档到 audit/run7_tape_analysis.md
- **N 的增大不能突破单一模型的知识分布边界** — 苦涩的教训在 LLM swarm 中的映射 → 已归档到 audit/run7_tape_analysis.md
