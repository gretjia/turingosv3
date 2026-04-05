# TuringOS v3 — Handover State
**Updated**: 2026-04-06 (01:00)
**Session Summary**: 4-researcher AI swarm 上线运行 + 市场修复 (LP=200) + 对照实验 (4.6x emergence) + 宪法审计 + 研究方向纠偏

## Current State
- **4 个 AI 研究员在 Mac Studio 自主运行中** (`./launch_swarm.sh status`)
  - α 阿里云 (qwen3-8b) — 46 exp, Life #8, 8B 地板守护者 ✅ 对齐
  - β 硅基α (Qwen2.5-32B) — 25 exp, Life #7, 需降到 7B-14B ⚠️ 待纠偏
  - γ 硅基β (Qwen2.5-32B) — 31 exp, Life #8, 需测不同架构 ⚠️ 待纠偏
  - δ 火山引擎 (doubao-seed/deepseek-v3-2) — 13 exp, Life #1, 需回到 seed 小模型 ⚠️ 待纠偏
- **共享白板**: `autoresearch/shared/bulletin.jsonl` (30 条, 已从 88 条压缩)
- **LP=200 市场修复生效**: 价格 5%-95% (旧: 41-59%), α 出现首次破产
- **研究方向**: 公告栏已发架构师指令 — 找地板不找天花板, 分工明确
- **火山引擎 qwen3-8b/32b 不可用**: 需在控制台手动开通推理接入点

## Changes This Session

### Multi-Researcher Swarm (`bbb5b63`, `e1f17a7`)
1. sweep.py v7 — identity, bulletin, OOM guards, semaphore, PID lock
2. 4 个 researcher workspace + 4 个 proxy (dashscope/siliconflow×2/volcengine)
3. launch_swarm.sh + monitor_swarm.sh (POSIX sh, macOS safe)
4. llm_proxy.py: `--provider` 强制标志 + 429 exponential backoff retry
5. Codex exec 审计 ×2: 首次 FAIL → 修复 → 二次 PASS

### 市场修复 (`e1f17a7`)
6. bus.rs: SYSTEM_LP_AMOUNT 1000→200 (Codex 审计 APPROVED)
7. actor.rs: 兄弟节点可见性 (Agent 看到同父兄弟可选投资)
8. Price Gate 保留 (Codex REJECT 了禁用方案)

### 宪法违规发现与修复
9. **答案泄露**: context.txt "divergent series and regularization techniques" → 已删除
10. **DEFAULT_CONTEXT**: evaluator.rs 硬编码同样泄露文本 → 清理为 "reasoning agent collaborating on a mathematical proof"
11. **Librarian prompt**: "1+2+3+...=-1/12 (regularization)" → "a mathematical proof"
12. **γ API key**: 西里尔字母 `а` (U+0430) 混入 → 修复为 ASCII `a`

### Bull/Bear 投资指南 (`e1f17a7`)
13. skill.txt 重写: 市场机制 + 投注规模 + 决策框架
14. evaluator.rs role/invest prompt 升级: "NEVER pass when you see good/bad math"

### 对照实验
15. control_group.py: 单 qwen3-8b 一次性 18 步(假) / 迭代 5 步(循环) vs swarm 23 步
16. run027_dag_audit.md: 完整 DAG 审计 (340 节点, 114 traded, 价格 5-95%)

### 公告栏管理
17. 88 条→30 条压缩 (归档 bulletin_archive_20260406_010245.jsonl)
18. 架构师指令: 研究方向纠偏 + 分工建议 + 协作提醒

## Key Decisions
- **涌现只在个体弱时可见**: 大模型个体能力接近集体 → emergence ratio 低 → 必须用小模型证明
- **LP=200 是平衡点**: LP=100 曾崩盘, LP=1000 杀死信号, LP=200 Codex 审计通过
- **Rust binary 必须在 Mac 编译**: x86→arm64 Exec format error 教训
- **不购买火山引擎 Distill 算力**: 按量计费模型足够, 预购算力不适合探索阶段
- **研究方向=找地板**: 8B 已证明 4.6x emergence, 应向下探 7B/3B/1B

## Architect Insights (本次会话)
- **苦涩的教训推论: 推理速度 > 参数量**: 72B 不如 32B, 因为时间有限 → 步骤少 → depth 受限
- **市场流动性是涌现的瓶颈**: LP=1000 杀死价格信号, Agent 的数学判断无法通过市场表达
- **涌现只在弱个体时可见**: 模型越强 → 个体能力越接近集体 → emergence ratio 趋近 1
- **协作不需要编排**: 4 AI 通过 append-only 白板自组织 (信息瀑布模式)
- **透题比想象中微妙**: context.txt 一个词 "regularization" 就引导了整条证明路径
- （本次会话未创建新 architect-insights 归档文件）

## Next Steps
1. **[RUNNING] 4 researcher 继续运行** — 等待纠偏指令生效, 观察是否转向小模型
2. **[TODO] β 降到 7B**: SiliconFlow 有 Qwen2.5-7B-Instruct, 需验证 7B 的 emergence ratio
3. **[TODO] δ 系统测 doubao-seed 地板**: 1.6-flash→1.6→2.0-lite→2.0-mini 逐步升级
4. **[TODO] 火山引擎开通 qwen3-8b**: 控制台→在线推理→预置推理接入点→开通
5. **[TODO] 每个新模型跑对照组**: control_group.py 计算 emergence ratio
6. **[OPEN SPRINT] bus.rs tick_map_reduce 重构**
7. **[OPEN SPRINT] math_membrane.rs 语义断头台对齐**

## Warnings
- **4 researcher + 4 proxy 在 Mac 运行中**: `./launch_swarm.sh status/stop`
- **α 阿里云 429 限流频繁**: proxy retry 处理中 (463 次), 考虑换 qwen3.5-flash (RPM 50x)
- **β/γ/δ 方向需要纠偏**: 公告栏指令已发, 但研究员是自主的 — 可能需要多轮指令才能纠正
- **sweep.py ×4 份同步**: 改一份必须 cp 到其余三份 + 重启 (`feedback_researcher_sync.md`)
- **内存 2.6GB 空闲**: 四人同时运行时偏低, monitor_swarm.sh 有 gamma-pause 机制
- **未提交**: rules/ 变更, zeta/config.json — 非关键, 下次 session 可一并提交
