# TuringOS v3 — Handover State
**Updated**: 2026-04-05
**Session Summary**: P0 WAL 修复 + AutoResearch v6 harness 重建 + Live AI Researcher 启动 + 首批 15 runs 数据

## Current State
- **AutoResearch v6 正在 Mac 运行** (tmux session `autoresearch`)，Life #6, 15 runs completed
- **P0 修复**: zeta evaluator 接入 bus.save_wal()/restore_wal() — Tape 不再丢失
- **Live AI Researcher**: sweep.py (DeepSeek R1 researcher + V3 auditor) 自主运行中
- **当前最佳**: Run 014, ERS=0.845, qwen3-32b, 5 agents (3M/1B+/1B-), 600s wall_clock
- **核心发现**: qwen3-8b 经济机制弱 (36% traded)，qwen3-32b 质的飞跃 (traded=98, YES:NO=80:156)
- **Researcher 正在测试**: qwen3-235b (Run 015, 进行中)

## Changes This Session

### P0: WAL Tape Persistence (核心修复)
1. `evaluator.rs` — 接入 bus.save_wal() 每次 append 后 + restore_wal() 启动时 (`ba4e5ba`)
2. `evaluator.rs` — WAL_PATH env var, 默认项目本地路径 (非 /tmp/) (`ba4e5ba`)
3. `evaluator.rs` — TAPE_OUTPUT env var, 带 timestamp 防覆盖 (`ba4e5ba`)
4. `evaluator.rs` — LIBRARIAN_INTERVAL 默认 100→8 固化架构师决定 (`ba4e5ba`)
5. `evaluator.rs` — generation 类型 u32→usize 匹配 bus.rs API (`ba4e5ba`)
6. `evaluator.rs` — File::create borrow fix for TAPE_OUTPUT String (`c455e4e`)

### AutoResearch v6 Harness
7. `run_experiment.py` — 全面重写: per-run WAL/tape/log/config 隔离, 自动 run_id, config 快照 (`c455e4e`)
8. `sweep.py` (NEW) — Live AI Researcher: DeepSeek R1 + V3, 大宪章优先, re-init/Markov memory (`c455e4e`)
9. `run_experiment.py` — proxy provider 支持 (Mac 需要 proxy, V-007) (`0c03236`)
10. `run_experiment.py` — timeout stderr 捕获修复 + WAL fallback metrics (`8d5b643`)
11. `config.json` — 起始配置: qwen3-8b, 5 agents, proxy 模式
12. `prompt/problem.txt`, `prompt/skill.txt` — agent 可见 prompt 文件
13. `results.tsv` — v6 新格式 (19 列, 含 config_json)
14. v5 归档: `experiments/zeta_sum_proof/audit/autoresearch_v5_{results,summary,logs}.tar.gz`

### V-010: Tape 丢失违规 (发现 + 修复)
- **根因**: zeta evaluator 从未调用 bus.save_wal()，而 minif2f_v2 正确使用。bus.rs 注释 "A Turing machine without persistent tape is not a Turing machine" 但 zeta 没接上。
- **修复**: 参照 minif2f_v2 接入 WAL restore (startup) + save (每次 append 后)
- **验证**: WAL JSON 文件正确持久化 (45KB, 含完整 tape + markets + wallets)

## Key Decisions
- **WAL 是 tape 的物理存在**: WAL JSON = 完整状态 (tape + markets + wallets + portfolios). Markdown dump 降级为可选人类可读导出
- **Karpathy AutoResearch 范式**: LLM IS the search algorithm. sweep.py = researcher's body, DeepSeek R1 = researcher's brain. 不是调参脚本。
- **大宪章优先**: Researcher prompt 第一段永远是三大立法 + 四大引擎 + LOCKED 参数
- **最小模型起步**: 从 qwen3-8b 开始，向上探索最小可行规模
- **全程 Aliyun API**: Mac 走 proxy (V-007 TLS deadlock)，默认 DashScope

## Architect Insights (本次会话)
- **Turing Machine 没有 tape 就不是 Turing Machine**: bus.rs 已有 WAL 但 zeta 没接。根因是 fork 遗漏，不是设计缺陷。
- **假深度的根源**: 7B/8B 模型的 depth=17-29 是 step numbering 绕过 dedup ("Step 10: Formalize" vs "Step 14: Formalize" 算不同前缀)
- **最小可行模型 ≈ qwen3-32b**: 8b 经济机制弱 (36% traded), 32b 质的飞跃 (traded=98). 模型规模比 swarm 数量更关键。
- **Researcher re-init 是理性的**: 面对矛盾数据 (metrics bug 导致 0 appends vs 实际 168) 选择重跑验证，这是科学方法，不是 bug

## AutoResearch v6 实验数据 (截至 Run 015)

| Run | Model | Swarm | ERS | Depth | Traded | 状态 |
|-----|-------|-------|-----|-------|--------|------|
| 001 | qwen3-8b | 5 (3/1/1) | 0.486 | 17 | 24 | baseline |
| 011 | **qwen3-32b** | 5 (3/1/1) | 0.659 | 12 | 39 | 模型升级突破 |
| 014 | **qwen3-32b** | 5 (3/1/1) 600s | **0.845** | 18 | 98 | **当前最佳** |
| 015 | qwen3-235b | 5 | 进行中 | - | - | Researcher 自主决策 |

## Next Steps
1. **[RUNNING] AutoResearch v6 继续** — Mac tmux `autoresearch`, Researcher 自主探索模型规模 + swarm 配置
2. **[TODO] 验证 qwen3-32b 经济机制**: 检查 Run 014 WAL 的 market activity, 确认 depth=18 是否真实
3. **[TODO] Dedup 改进**: 30-char prefix 被 step numbering 绕过，需要更语义化的去重
4. **[OPEN SPRINT] bus.rs tick_map_reduce 重构**
5. **[OPEN SPRINT] math_membrane.rs 语义断头台对齐**
6. **[DEFERRED] ThermodynamicHeartbeatTool 接入** — bus.rs 生命周期变更需人工确认

## Warnings
- **AutoResearch 在 Mac tmux 运行中**: `ssh zephrymac-studio "tmux attach -t autoresearch"` 查看实时输出
- **Mac proxy 需提前启动**: 如 proxy 挂了, evaluator 会 timeout
- **Dead code**: `experiments/zeta_sum_proof/src/wal.rs` 是旧 WAL 实现, 未删除, 与 bus.rs WAL 无关
- **sweep.py Researcher 可能 re-init 过频**: 当前无 re-init 冷却期, 如果连续失败会反复重生
- **LIBRARIAN_INTERVAL=8 已固化**: Rust 默认值 + harness 都传 8
