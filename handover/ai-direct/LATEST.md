# TuringOS v3 — Handover State
**Updated**: 2026-03-21
**Session Summary**: 从 ζ(-1) 到 Number Theory — 两个 OMEGA，Core SDK 建设，大宪章实施，boot 自动化

## Current State
- **两个定理已证明**:
  - ζ(-1) = -1/12 (Run 15, `apply?`, 51 min)
  - Smallest n: 7|n ∧ square ∧ digit9 ∧ digitsum25 = **5929** (Run 2, 构造性证明, 26 min)
- **Core SDK 已建成**: `protocol.rs`, `prompt.rs`, `search.rs` — 任何新项目可复用
- **boot-experiment.sh**: 3 参数自动创建项目 + 编译 + 部署 + 启动
- **所有测试已完成**: Mac 上无活跃 swarm 进程
- **Branch `auto-numtheory-loop`**: 2 commits 未合并到 main (drain timeout fix + OMEGA docs)

## Changes This Session (主要 commits)
- `176cd88` — Core SDK: protocol.rs (JSON <action> parser), prompt.rs (极简模板), search.rs (免费搜索)
- `616f98e` — minif2f swarm 迁移到 Core SDK
- `73ef88a` — search/observe routing 修复: 免费行为不再进入 bus.append()
- `1f911d4` — **OMEGA #1**: ζ(-1) = -1/12 by Agent_2 (R1, `apply?`)
- `49f55d3` — 形式化证明提交文档 (独立复现 + 对照组验证)
- `4ab6950` — 大宪章实施: ALIGNMENT.md + 四引擎 + Guillotine + Invest 术语
- `8f0dc7f` — boot-experiment.sh: 自动化实验启动脚本 + README 重写
- `dcc76aa` — number_theory_min 实验项目
- `23af73f` — drain timeout 修复 (5 min 截断慢 agent)
- `413a59a` — **OMEGA #2**: n=5929 构造性证明 by Agent_1 (deepseek-reasoner)

## Key Decisions
- **极简 prompt**: Run 13 (复杂 prompt, 12%) vs Run 15 (极简, OMEGA) → "压缩即智能"
- **撤销 Auto-Oracle**: Gemini 建议保留显式 search 工具, 不自动喂饭 (保护物种演化)
- **JSON 协议 + legacy fallback**: `<action>{...}</action>` 优先, 兼容 `[Tactic:]+[Wallet:]`
- **drain timeout**: collect-all 架构 + 5 min 截断 → 6x 加速
- **大宪章**: 四引擎实施但 prompt 不解释规则 → 系统执行体现规则

## 15 轮 ζ(-1) 演化 + 2 轮 Number Theory
| Run | 题目 | Append | OMEGA | 关键事件 |
|-----|------|--------|-------|---------|
| 1-3 | ζ(-1) | 0-13 | ❌ | sandbox stderr 修复 |
| 5-8 | ζ(-1) | 0-1 | ❌ | 异构 swarm + 退火 + 首次 append |
| 12-13 | ζ(-1) | 50/12 | ❌ | 大宪章 → prompt 过载 |
| 14-15 | ζ(-1) | 0/3 | **✅** | search routing 修复 → OMEGA |
| NT-1 | 5929 | 0 | ❌ | drain 阻塞 (30 min/step) |
| **NT-2** | **5929** | **2** | **✅** | **drain timeout → 26 min OMEGA** |

## Next Steps
1. **合并 `auto-numtheory-loop` 到 main**
2. **drain timeout 提升到 Core SDK** (目前只在 number_theory_min)
3. **membrane blocklist 参数化** (native_decide 按题目配置)
4. **更多定理测试** — 验证 SDK 架构的通用性
5. **Speciation Engine (Phase 4)** — per-agent DNA 演化

## Warnings
- Branch `auto-numtheory-loop` 有 2 commits 未合并到 main
- `native_decide` 在 membrane blocklist 中, 可能误杀合法 tactic (Number Theory Run 1 中 29% reject)
- zeta 项目的 swarm.rs 没有 drain timeout (只有 number_theory_min 有)
- Mac .env 没有 SILICONFLOW_API_KEY_SECONDARY — tmux 启动时必须手动 export

## Architect Insights (本次会话)
- **压缩即智能**: 不要把社会契约编码进 prompt, 让社会分工从价格信号中涌现 → 已归档到 ALIGNMENT.md
- **实施原则**: "暂时不追求效率, 保证原则对齐, 为泛化能力储备" → 已归档到 ALIGNMENT.md
- **构造性推理 > 信息检索**: Number Theory OMEGA 是真正的数学推理 (n=5929 独立推导 + 最小性穷举), 不是 apply? 库搜索 → 审计报告已证实
