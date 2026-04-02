# TuringOS v3 — Handover State
**Updated**: 2026-04-02
**Session Summary**: AutoResearch (Karpathy 式) 20 实验 + 三层架构修复 (dedup/budget/bulletin) + Librarian Engine + DeepSeek Halt Gate

## Current State
- **AutoResearch 框架: DEPLOYED** — `autoresearch/zeta/` 含 program.md, run_experiment.py, librarian.py, results.tsv
- **Branch-aware Dedup: DEPLOYED** — bus.rs Phase 3b, 同分支去重, 跨分支允许复用
- **Append-only Budget: DEPLOYED** — max_tx 只计 append, 投资不挤压建树
- **Global Bulletin: DEPLOYED** — 全局公告板, 所有 agent 可见 common errors
- **Librarian Tool: DEPLOYED** — 每 100 appends 压缩 tape → learned.md (Engine 4)
- **DeepSeek Halt Gate: DEPLOYED** — [COMPLETE] 需 P>=90% + DeepSeek 验证才触发 OMEGA
- **LLM Provider 可配**: env var 切换 Aliyun/SiliconFlow, 模型可配
- **zeta_sum_proof: 未证明** (20 实验, 无 OMEGA — dedup 暴露 7B 真实深度 ~12-14)

## Changes This Session

### Committed (3 commits)
- `48d03d5` Role Trifecta (Math/Bull/Bear) + 弱模型实验 (Run 3-4)
- `f8a17cf` 90-agent scaling + OMEGA (Run 5-6, 6000tx)
- `7a57598` Control group — single 7B vs 90-agent swarm

### Uncommitted (major)
- **src/bus.rs**: Branch-aware dedup (Phase 3b) — 40-char 前缀, 分支内去重
- **src/drivers/llm_http.rs**: `enable_thinking: false` for Qwen3 + max_tokens 3072
- **src/sdk/tools/librarian.rs**: NEW — Librarian Tool (tape → memory 压缩)
- **src/sdk/tools/mod.rs**: 注册 librarian 模块
- **evaluator.rs**: env var 配置 + DeepSeek halt gate + global bulletin + append-only budget + Librarian 挂载
- **math_membrane.rs**: [COMPLETE] 不再自动 OMEGA, 交给 DeepSeek 验证
- **autoresearch/zeta/**: 完整 Karpathy 式 AutoResearch 框架

### Aliyun API 接入
- `.env` 新增 `DASHSCOPE_API_KEY`
- evaluator 支持 `LLM_PROVIDER=aliyun|siliconflow` + `LLM_MODEL` env var
- 测试: Aliyun qwen2.5-14b-instruct 30 并发 OK, 但长 prompt 慢 (~47s/tx)

## Key Decisions
- **5/5/5 均衡最优**: AutoResearch 20 实验确认, 偏比例配置都不如均衡 (depth 下降)
- **Dedup 必须分支感知**: 全局去重杀死跨分支复用 (depth 9→14)
- **投资与建树分离计数**: 95% tx 被投资消耗是之前的隐藏瓶颈
- **DeepSeek 做 halt 仲裁**: Agent 声称 [COMPLETE] → 市场定价 → P>=90% → DeepSeek 验证 → OMEGA
- **Librarian 是 Engine 4 基础设施**: 不是实验工具, 是架构级记忆压缩管道

## Architect Insights (本次会话)
- **角色分化 5/5/5**: 经济活动不足+重复节点 → 分角色对抗 → `2026-04-01_role-differentiation-15agents.md`
- **7 条苦涩教训**: 假深度/投资挤压/指标 bug/14B 分散 等 → `2026-04-02_autoresearch-bitter-lessons.md`
- **Librarian 记忆压缩**: Tape=原始日志, Memory=压缩智慧, 成功失败分开 → `2026-04-02_librarian-memory-compression.md`
- **Librarian 架构级定位**: Engine 4 的记忆基础设施, 不是 autoresearch 工具 → `2026-04-02_librarian-architecture.md`

## Next Steps
1. **[OPEN] Commit 所有未提交变更** — 大量架构改动未 commit
2. **[OPEN] 14B + Librarian + DeepSeek Gate 首次完整 run** — 全套新架构跑 zeta
3. **14B 深度引导** — 14B 太分散 (depth=4, roots=9), 需要 Boltzmann 偏好深链
4. **MiniF2F 迁移** — 当前架构改进应用到真正的定理证明任务
5. **Librarian 跨 run 测试** — 验证 learned.md 压缩记忆在下一次 run 中是否被 agent 利用

## Warnings
- **W1**: Run 6 tape 已永久丢失 (被后续实验覆盖). run_experiment.py 已加自动 persist, 但 evaluator 直接运行时仍需手动备份
- **W2**: Aliyun qwen2.5-14b-instruct 长 prompt 极慢 (~47s/tx), 建议用 SiliconFlow
- **W3**: Aliyun qwen2.5-math-7b-instruct 不可用 (冷启动无响应)
- **W4**: max_tokens 已从 8192 降至 3072 (兼容 math 模型), 可能影响长输出
- **W5**: 20 个 AutoResearch 实验的 results.tsv 存在, 但前 12 个实验使用了有 bug 的 novelty 指标 (parse_tape 空行 bug)
- **W6**: SiliconFlow TPM 限流在高并发 (>30 agents) 时会触发, 需要控制 agent 数量或错峰

## Audit Trail
- AutoResearch: 20 experiments, `autoresearch/zeta/results.tsv`
- Bitter Lessons: `handover/architect-insights/2026-04-02_autoresearch-bitter-lessons.md`
- Run 3-6 DAGs: `experiments/zeta_sum_proof/audit/run{3,4,5,6}*.md`
- Control group: `experiments/zeta_sum_proof/audit/run6_control_single_7b.md`
