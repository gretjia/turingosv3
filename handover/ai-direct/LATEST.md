# TuringOS v3 — Handover State
**Updated**: 2026-04-04
**Session Summary**: AutoResearch v5 — linux1 LAN 迁移 + 宪法守卫 + Reasoner 全自由探索 + 4 条架构师洞察

## Current State
- **AutoResearch v5: RUNNING** on linux1 (PID 145136) — Baseline 进行中
  - 编排: linux1 native Linux (192.168.3.113)
  - 推理: linux1 (parallel=20, Vulkan, 38 tok/s) + Mac (parallel=5, 33 tok/s)
  - 搜索代理: DeepSeek Reasoner (R1)
  - 宪法守卫: Rust --constitutional-check + Gemini 2.5 Flash (via Mac proxy)
  - 沙盒测试: 14/14 全通过
  - TSV: `/home/zepher/autoresearch/audit/autoresearch_v4_phase2.tsv`

## Hardware Topology (运维参考)

| 节点 | LAN IP | 角色 | llama parallel | 持久性 |
|------|--------|------|---------------|--------|
| linux1 | 192.168.3.113:8080 | 主力编排+推理 | 20 | **cron 每 30min 守护** |
| Mac | 192.168.3.93:8080 | 辅助推理 + VPN 代理 (:7897) | 5 | nohup (非持久) |
| Win1 | 192.168.3.112:8081 | 备用推理 | 2 | schtask `\llama9b` (持久) |
| omega-vm | GCP | 代码仓库 + Git | N/A | 永久 |

**SSH 到 linux1**: `ssh zephrymac-studio "ssh zepher@192.168.3.113 'cmd'"`
**VPN 代理**: `https_proxy=http://192.168.3.93:7897` (GitHub/Google/Gemini)
**详细硬件配置**: 见 memory `reference_hardware_config.md` + `reference_network_topology.md`

## Changes This Session

### Committed: 无新 commit (全部 uncommitted)

### Uncommitted (+525/-93, 12 files)
- **evaluator.rs**: +69 行, --constitutional-check 模式 (Rust 宪法硬守门)
- **sweep_v4.py**: +349 行, v5 重写 (Reasoner 全自由 + Markov re-init + Gemini 审计 + raw traces + WALL_CLOCK 可调)
- **RESEARCH_PLAN.md**: Phase 2 废弃, Phase 3 重写 (swarm scale + 退位机制)
- **monitor.sh**: Win1 从 NSSM 改为 Start-Process, TSV 路径更新
- **prompt/problem.txt**: 新 prompt (不透题 + 计算纪律)

### Violations Found & Fixed
- **PRICE_GATE_ALPHA>0**: 高价父节点永不退位 bug → 设为 0
- **LIBRARIAN_INTERVAL=20**: 65% 实验零 depth → 降为 8
- **FRONTIER_CAP/DEPTH_WEIGHT/PRICE_GATE_ALPHA 误判违宪**: 误将"设为 0"判定为违宪并恢复默认 → 架构师纠正: 0 才是宪法正确 (移除人工偏好)

## Key Decisions
- **群体即思考机器**: 个体不需 thinking mode → Phase 2 废弃
- **0/0/0 是宪法正确**: FRONTIER_CAP=0, DEPTH_WEIGHT=0, PRICE_GATE_ALPHA=0
- **Run 6 成功因素是吞吐**: 50min×6000tx, 不是参数调优
- **Reasoner 完全自由**: 所有 Layer 2 参数可探索, 唯一约束=大宪章+topology
- **双重宪法守卫**: Rust + Gemini, 每次变更前强制
- **Librarian 纳入 AutoResearch**: 压缩机制也是可探索工件

## Architect Insights (本次会话)
- **群体即思考机器, 个体只需快; 价格是唯一裁判** → `2026-04-04_swarm-is-the-thinker.md`
- **PRICE_GATE_ALPHA>0 = 高价节点永不退位 bug** → `2026-04-04_price-gate-alpha-bug.md`
- **黑盒加 log_lib + Reasoner 加 re-init** → `2026-04-04_blackbox-tools-expansion.md`
- **re-init 前宪法对齐, Rust 硬性守门** → `2026-04-04_reinit-constitutional-guard.md`

## Next Steps
1. **[RUNNING] AutoResearch v5** — Reasoner 自主运行, 不需干预
2. **[OPEN SPRINT] log_lib agent 工具** — evaluator.rs 添加 agent 查询 Librarian 摘要功能
3. **[OPEN SPRINT] Librarian 扩展** — 加入最深链内容、价格变化趋势、被弃节点
4. **[OPEN SPRINT] linux1 反向 SSH 隧道恢复** — 在 linux1 执行 `ssh -fN -R 2226:localhost:22 hk-wg`
5. **[OPEN SPRINT] 提交本 session 变更** — 12 个文件 +525/-93

## Warnings
- **linux1 只能通过 Mac 跳板访问**: 反向隧道 hk-wg:2226 断了
- **linux1 cron 守护**: `*/30 * * * * /home/zepher/autoresearch/monitor.sh` — 自动重启 sweep + llama-server
- **Mac llama-server 非持久**: 重启后需手动启动 (见 memory reference_hardware_config.md)
- **Gemini API 免费配额**: 每次变更调一次 Gemini, 频繁实验可能触限
- **sweep_v5 路径差异**: omega-vm 用 `/home/zephryj/...`, linux1 用 `/home/zepher/...` + LAN 端点, 同步时需 patch
