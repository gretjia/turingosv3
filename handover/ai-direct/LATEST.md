# TuringOS v3 — Handover State
**Updated**: 2026-03-22
**Session Summary**: Actor Model 重构 + zeta_sum_proof 启动 — 首次无验证器纯市场推理 swarm

## Current State
- **zeta_sum_proof 正在运行** (Mac tmux `zeta-sum`, log: `/tmp/zeta_sum_run1.log`)
  - Actor Model: 15 agents 独立循环，无 drain，无阻塞
  - 5 分钟内 18 append + 5 ViewNode + 2 observe — 吞吐量远超旧架构
  - 纯市场验证（MathStepMembrane，无 Lean 4 中间检查）
- **Core SDK 已扩展**: `snapshot.rs` + `actor.rs` 加入 SDK
- **三个 OMEGA 已达成**: ζ(-1), n=5929, (zeta_sum_proof 运行中)
- **boot-experiment.sh**: 全自动实验启动脚本可用

## Changes This Session
- `c0caaa1` — zeta_sum_proof 项目 boot (MathStepMembrane + Order Book + chain prompt)
- `7b14db5` — 无锁 Actor Model plan & spec 归档
- `f11c171` — **Actor Model 实施**: snapshot.rs, actor.rs, evaluator async reactor
- `f11c171` — Guillotine 传播到 minif2f membrane
- `f11c171` — ALIGNMENT.md 加入无锁 + 无认知溢价原则
- 架构师指令归档: `2026-03-21_lockfree-austrian-naked-swarm.md`

## Key Decisions
- **Actor Model 替代 batch-synchronous**: watch/mpsc 消息传递，无锁，无 drain
- **纯市场验证 (无 Lean 4)**: 测试 Hayekian 价格发现能否替代编译器仲裁
- **MathStepMembrane 极简**: 只检查非空 + [COMPLETE]，市场处理质量
- **Order Book (Gemini 建议)**: 3 条竞争链 (consensus + alt + recent)
- **私有 Agent 上下文**: search/view 结果仅注入请求者，不泄漏
- **无认知溢价**: 系统不为思考时间买单 (奥地利学派主观价值论)
- **终局 Oracle**: [COMPLETE] 触发后用 Lean 4 做追溯验证

## Next Steps
1. **监控 zeta_sum_proof Run 1** — 分析 tape，验证市场能否产生正确推理链
2. **终局 Oracle 实现** — [COMPLETE] 后提取链 → Lean 4 最终验证
3. **裸核盲测** — Mock Membrane (图迷宫)，验证 OS 独立于大模型能力
4. **Speciation Engine** — per-agent DNA (Phase 4 deferred)
5. **drain timeout 传播到 zeta_regularization** (Actor Model 已解决此问题)

## Warnings
- **zeta_sum_proof 正在 Mac 上运行** — tmux `zeta-sum`, 不要 kill
- Cargo.lock 有未提交变更 (zeta_sum_proof 依赖新增)
- Actor Model 目前只在 zeta_sum_proof 中使用，minif2f/zeta_regularization 仍用旧 swarm
- zeta_sum_proof 的 swarm.rs 未被新 evaluator 使用（可删但保留作参考）
- 所有 agent 默认构建在最贵节点上（无 Boltzmann 路由 — 首次测试简化）

## Architect Insights (本次会话)
- **无锁物理学**: Append-Only DAG = 无锁。Agent 读快照，写子节点。不需要锁。 → 已归档到 `directives/2026-03-21_lockfree-austrian-naked-swarm.md`
- **无认知溢价**: 系统不为思考时间买单，价值由结果边际效用决定 → 已归档到 ALIGNMENT.md
- **裸核盲测**: 剥离 Lean，用 Mock Membrane 验证 OS 泛化能力 → 已归档到 directives
