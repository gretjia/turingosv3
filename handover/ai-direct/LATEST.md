# TuringOS v3 — Handover State
**Updated**: 2026-04-05 (evening)
**Session Summary**: Multi-Researcher Swarm (4 AI) 上线 + 市场修复 (LP=200) + 对照实验 (4.6x emergence) + 宪法审计 + Bull/Bear 指南

## Current State
- **4 个自主 AI 研究员在 Mac Studio 运行中** (launch_swarm.sh)
  - α 阿里云 (qwen3-8b) — 39 exp, Life #8, 标准制定者
  - β 硅基α (Qwen2.5-32B/SiliconFlow) — 20 exp, Life #7, 全局最佳 ERS=0.964
  - γ 硅基β (Qwen2.5-32B/SiliconFlow) — 21 exp, Life #6, 验证者
  - δ 火山引擎 (doubao-seed-2.0-pro/Volcengine) — 3 exp, Life #1, 新上线探索者
- **共享白板**: `autoresearch/shared/bulletin.jsonl` (73 条), 四人通过白板自组织协作
- **市场修复**: LP 1000→200, 价格信号从死水 (41-59%) 变为活跃 (5-95%)
- **对照实验**: 单 agent qwen3-8b 仅 5 步循环, swarm 23 步 → **4.6x emergence multiplier**
- **宪法审计 PASS**: 零违宪行为, 经济系统零和守恒, 内核纯度完好

## Changes This Session

### Multi-Researcher Swarm (AutoResearch v7) — `bbb5b63`
1. `sweep.py` 重写 — identity.json 加载, bulletin R/W, OOM guards (subprocess→file, setrlimit), global semaphore, per-researcher PID lock
2. `launch_swarm.sh` + `monitor_swarm.sh` — POSIX sh (macOS Bash 3.2 safe), PID file 管理, 自动重启
3. `shared/` 知识公地 — problem.txt, skill.txt (只读), bulletin.jsonl (append-only fcntl 锁)
4. 4 个 researcher workspace: zeta/ (α), zeta-b/ (β), zeta-c/ (γ), zeta-d/ (δ)
5. 4 个 proxy 实例: :8088 dashscope, :8089/:8090 siliconflow, :8091 volcengine
6. `--provider` flag 加入 llm_proxy.py (Codex 审计发现路由 bug 后修复)
7. Codex exec 审计 x2: 首次 FAIL (3 CRITICAL + 5 HIGH) → 全部修复 → 二次 PASS

### 市场价格信号修复
8. `bus.rs:65` — SYSTEM_LP_AMOUNT 1000→200 (Codex 审计 APPROVED, CTF 守恒不变)
9. `actor.rs` — 兄弟节点可见性: Agent append 前看到同父兄弟的 payload+价格, 可选投资而非重复
10. Price Gate 保留不变 (Codex 审计 REJECT 了禁用方案: LP=200 下 Gate 自然恢复有效)

### 宪法修复
11. `context.txt` 删除 — 包含 "divergent series and regularization techniques" (答案泄露)
12. `evaluator.rs` DEFAULT_CONTEXT 清理 → "You are a reasoning agent collaborating on a mathematical proof"
13. `librarian.rs` prompt 清理 — "1+2+3+...=-1/12 (regularization)" → "a mathematical proof"

### Bull/Bear 投资指南
14. `shared/skill.txt` 全面重写 — 市场机制说明, 投注规模指南, 决策框架, 何时投资 vs 建树
15. `evaluator.rs` role prompt 升级 — Mathematician/Bull/Bear 都有具体投资策略
16. `evaluator.rs` invest prompt 升级 — 决策框架 + 投注规模 + "NEVER pass" 激励

### 429 限流修复
17. `llm_proxy.py` — exponential backoff retry (最多 4 次, 2s→3s→5s→9s)
18. α proxy 已处理 463 次 429 重试 (Aliyun TPM 限流)

### Bug 修复
19. `.env` 西里尔字母 `а` (U+0430) → ASCII `a` — γ 连续 10+ 次 ERS=0.0 的根因
20. MAX_CONCURRENT_EVALUATORS 3→4 — 加 δ 后其他三人被阻塞

### 对照实验
21. `control_group.py` — 单 agent qwen3-8b 基线对照: 一次性 (18 步假) + 迭代 (5 步循环)
22. `run027_dag_audit.md` — depth=23 DAG 完整审计 (340 节点, 114 traded, 价格 5-95%)

## Key Decisions
- **每个 Researcher 是独立生命体**: 自己的笔记/历史/个性, 白板是唯一社交空间, 无中央控制
- **LP=200 (非 100, 非 1000)**: LP=100 曾导致崩盘, LP=1000 杀死价格信号. 200 是 Codex 审计通过的中间值
- **Price Gate 不禁用**: Codex 证明 LP=200 让 Gate 自然恢复有效 (10 coin 触发)
- **Rust binary 必须在 Mac 上编译**: x86→arm64 不兼容 (Exec format error 教训)
- **对照实验协议**: Experiment B (迭代接龙) 是最公平对比, 4.6x emergence multiplier

## Architect Insights (本次会话)
- **苦涩的教训推论: 推理速度 > 参数量**: 72B 不如 32B, 因为 72B 在有限时间内产出步骤少. DeepSeek-V3 同理
- **市场流动性是涌现的瓶颈**: LP=1000 让价格信号死亡, Agent 的数学判断无法通过市场表达
- **协作不需要编排**: 4 个 AI 通过 append-only 白板自组织 — β 发布发现 → α 跟进验证 → γ 借鉴方向 → δ 探索新模型
- **透题比想象中微妙**: context.txt 里一个词 "regularization" 就足以引导整条证明路径

## Swarm 实验数据 (key runs)

| Researcher | Run | Model | ERS | Depth | 关键发现 |
|------------|-----|-------|-----|-------|---------|
| α | 024 | qwen3-8b, 600s | 0.819 | 16 | frontier=0, 100%被投资 |
| α | 027 | qwen3-8b, 900s | 0.512 | **23** | 最深链! 340节点零重复 |
| α | 033 | qwen3-8b, LP=200 | 0.436 | 12 | **价格 5%-95%** 市场修复确认 |
| β | 008 | Qwen2.5-32B, 900s | **0.964** | 17 | 全局最佳 ERS |
| β | 009 | Qwen2.5-32B, 2B- | 0.897 | 19 | 更多 bear → 深度增 |
| γ | 019 | Qwen2.5-32B, LP=200 | 0.733 | 9 | LP=200 后 |
| δ | 002 | doubao-seed-2.0-pro | 0.356 | 6 | 首次火山引擎数据 |

## Next Steps
1. **[RUNNING] 4 researcher swarm 自主运行** — 监控: `./launch_swarm.sh status`
2. **[TODO] LP=200 + Bull/Bear 指南后的市场数据分析** — 价格分布/破产率是否改善
3. **[TODO] 对照实验扩展** — 在 LP=200 新内核下重跑基线对照, 看 emergence ratio 是否变化
4. **[OPEN SPRINT] bus.rs tick_map_reduce 重构**
5. **[OPEN SPRINT] math_membrane.rs 语义断头台对齐**

## Warnings
- **4 个 researcher + 4 个 proxy 在 Mac 运行中**: `./launch_swarm.sh status` 查看, `./launch_swarm.sh stop` 停止
- **Mac proxy 需要 .env 里的 API key**: 如果 key 过期/欠费, proxy 返回 500
- **α 阿里云频繁 429 限流**: proxy retry 在处理但可能影响实验速度. 考虑换 qwen3.5-flash (RPM 50x)
- **sweep.py × 4 份**: 改一份必须 cp 到另外三份 + 重启. 参考 memory `feedback_researcher_sync.md`
- **未提交的变更**: LP=200, skill.txt, role prompt, zeta-d/, evaluator.rs — 需要 commit + push
