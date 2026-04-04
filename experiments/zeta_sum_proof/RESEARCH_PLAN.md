# ζ-Sum AutoResearch — Research Plan v4

Date: 2026-04-03
Status: Active (sweep_v4.py running)

## 目标

**用 TuringOS 多 agent 群体完成 1+2+3+...=-1/12 的严格正则化证明。**

成功标准: 一条 ≥15 步的证明链，每步包含具体代数推导，最终 [COMPLETE] 声明被 DeepSeek Reasoner Oracle 验证通过。

## 度量: ERS (Effective Reasoning Score)

```
ERS = (depth/20)² × novelty × focus
```

- **depth²**: 证明深度是最重要的。从 5 步到 10 步是 4x 改进。
- **novelty**: (appends - dedup) / appends. 新内容比例。
- **focus**: min(30/frontier, 1.0). 前沿不超过 30。

depth 平方的原因: 当前系统产出大量宽度 (平行重复结论) 而非深度 (递进推导)。ERS 直接惩罚宽度、奖励深度。

## 研究问题

AutoResearch 需要回答 5 个问题。按预期影响力排序:

### Q1: Prompt 如何引导推理深度？ (最高影响)

| 假设 | 预期 |
|------|------|
| 给出完整 math toolkit → agents 使用具体工具 | depth ↑, 空断言 ↓ |
| 要求 "SHOW algebra, NOT conclusions" → agents 写等式 | novelty ↑, dedup ↓ |
| 分步骤引导 (step 1=定义, step 2=展开, step 3=代入...) → 线性推进 | depth ↑↑ |
| 去掉 toolkit 让 agents 自己发现 → 多样性 | 可能 depth ↓ 但 breadth ↑ |

**sweep_v4 如何研究**: DeepSeek 读取 tape 样本，看到 agents 在写什么，直接编辑 problem.txt / skill.txt。这是最高杠杆的操作。

### Q2: Thinking Mode vs 速度? (高影响)

| 模式 | 速度 | 质量 | 10 min 预估 |
|------|------|------|------------|
| off (/no_think) | ~33 tok/s | 口号化描述 | ~50 appends |
| budget:800 | ~15 tok/s (估) | 部分推导 | ~25 appends |
| budget:1500 | ~10 tok/s (估) | 详细推导 | ~15 appends |
| on (full thinking) | ~3 tok/s | 完整代数 | ~5 appends |

**关键权衡**: 5 个高质量步骤 (thinking ON) vs 50 个低质量步骤 (OFF)。
**预期**: budget:800 可能是 sweet spot — 足够推导但不会耗尽时间。
**sweep_v4 如何研究**: DeepSeek 可以设置 THINKING_MODE 参数。

### Q3: 经济机制如何影响深度? (中等影响)

| 参数 | 当前值 | 研究范围 | 预期 |
|------|--------|---------|------|
| FRONTIER_CAP | 30 | 10-50, 0(无限) | 太大→稀释，太小→过度集中 |
| DEPTH_WEIGHT | 1.0 | 0-2.0 | 越高→越偏向深链 |
| PRICE_GATE_ALPHA | 0.05 | 0-0.10 | 越高→父节点越粘 |
| GLOBAL_DEDUP | true | true/false | 关闭→更多重复但可能更多探索路径 |

**预期**: 这些是微调。prompt 变更的影响远大于这些参数 (10x 级别差异 vs 2x)。

### Q4: 角色比例是否重要? (低影响)

| 配置 | 预期 |
|------|------|
| 全 Math (10M/0B+/0B-) | depth 最高但无市场压力 |
| 偏 Math (8M/1B+/1B-) | 兼顾 |
| 均分 (5M/3B+/2B-) | 市场活跃但 depth 可能受限 |

**预期**: 之前 sweep_v2 的结果显示 math-heavy 配置赢。全 Math 可能更好因为 10 min 太短，市场来不及形成有效价格。

### Q5: 模型大小 vs 数量? (探索性)

Win1 有 128GB 可以跑 27B 模型。Mac 只能跑 9B/4B。

| 方案 | 配置 | 预期 |
|------|------|------|
| 多小模型 | 10 × 9B (Mac+Win1) | 更多 appends，但每步浅 |
| 少大模型 | 3 × 27B (Win1 only) | 更少 appends，但每步深 |
| 混合 | Math=27B (Win1), Bull/Bear=9B (Mac) | 质量+速度 |

**预期**: 这需要重启 Win1 server 换模型，是更大的变更。先把 Q1-Q4 答完再考虑。

## 实验计划

### Phase 1: Prompt Search ✅ 完成

Best prompt (exp059), depth=7 天花板。Prompt 已锁定。

### ~~Phase 2: Thinking Mode~~ ❌ 废弃

> **架构师指令 (2026-04-04)**: TuringOS 本身就是 thinking machine。个体 agent 不需要 thinking mode — 群体协作 + 市场定价就是"思考"。THINKING_MODE 永久固定为 "off"。

### Phase 3: Swarm Scale + 退位机制研究 (当前)

> **架构师指令 (2026-04-04)**: 价格是唯一裁判。不需要人工偏好、不需要前沿限制。唯一要研究的是 swarm 规模和子节点退位机制。

**锁死参数** (不再扫描):
- THINKING_MODE = "off" (群体即 thinker)
- FRONTIER_CAP = 0 (无限，市场自然淘汰)
- DEPTH_WEIGHT = 0 (无人工深度偏好，价格说了算)
- GLOBAL_DEDUP = true (永久开启)

**唯一保留的经济杠杆**: PRICE_GATE_ALPHA

**研究维度 A — Swarm Scale (推到硬件上限)**:

三节点 LAN 拓扑:
```
linux1 (192.168.3.113:8080) — parallel=20+, Vulkan GPU, 108GB VRAM
Mac    (192.168.3.93:8080)  — parallel=5, Apple M4, 32GB
Win1   (192.168.3.112:8081) — parallel=5, Vulkan GPU, 32GB
总 slots ≈ 30
```

扫描: N=10 → 15 → 20 → 30 → 40 → 50+
- parallel 跟随 N 同步提升
- 每级 N 跑 3 次取中位数 ERS
- **停止条件**: depth 不再随 N 增长 或 elapsed > 700s
- 角色比例随 N 等比放大 (60% Math / 20% Bull / 20% Bear)

**研究维度 B — 子节点退位机制 (防止过早退位)**:

**核心问题**: 市场可能过早抛弃看似无望但实际有价值的证明分支。数学证明中间步骤往往反直觉 — 第 2 步"丑陋"但第 7 步突破。市场在第 2 步就判死刑。

**矛盾**: 市场效率 (快速淘汰低价节点) vs 探索耐心 (走完反直觉路径)。

**待测假设**:

| 假设 | 机制 | 如何测 |
|------|------|--------|
| 退位太快 | 节点存活 N 轮后才能被判"死亡" | 增加 min_survival_rounds 参数 |
| 流动性不足 | 系统做市商持续为所有节点提供最低买单 | 调整做市商初始注入量 |
| 缺乏二次机会 | Boltzmann 采样偶尔随机选择低价节点延伸 | 测试非零温度下的低价节点复活率 |
| 评估滞后 | Librarian 定期重新评估被弃节点潜力 | 降低 LIBRARIAN_INTERVAL + 评估覆盖度 |

**AutoResearch 可测指标**: 
- 被弃节点中有多少本可延伸到 depth+2 (事后分析)
- 最终 OMEGA 路径上有多少节点曾经低于退位阈值
- 平均节点存活时间 vs 贡献 depth

**执行方式**:
- 轮 1: 纯 swarm scale (N=10→50+)，锁死经济参数，找最优 N
- 轮 2: 固定 N，审计 evaluator.rs 退位逻辑，暴露退位参数，Reasoner 搜索

### Phase 4: Model Scaling (可选)

linux1 108GB VRAM，9B 仅用 ~5GB。可测:
- 全 27B (parallel=6, ~20 tok/s) — 如果 swarm scale 不够，试更强模型
- 异构: Math=27B (linux1), Bull/Bear=9B (Mac+Win1)
- **前提**: Phase 3 swarm scale 未突破 depth 天花板时才进入

## 预期时间线

| Phase | 时间 | 实验数 | 关键产出 |
|-------|------|--------|---------|
| Phase 1 | ✅ 完成 | 96 | 最佳 prompt (exp059), depth=7 |
| ~~Phase 2~~ | ❌ 废弃 | — | 群体即 thinker，个体不需 thinking |
| Phase 3 轮1 | **当前** | ~30 | Swarm scale N=10→50+，找最优 N |
| Phase 3 轮2 | 轮1后 | ~20 | 子节点退位机制研究 |
| Phase 4 | 视情况 | ~10 | 27B / 异构 swarm (仅当 Phase 3 触顶) |

## 架构

```
linux1 (192.168.3.113, 深圳局域网, AMD AI Max 395 128GB)
  ├── sweep_v4.py                         — AutoResearch loop (NEVER STOP)
  │     ├── DeepSeek Reasoner (搜索代理)  — 读结果 → 编辑 prompt/config
  │     └── evaluator binary              — 10 min 固定预算
  │           ├── prompt/ 目录             — THE MUTABLE ARTIFACT
  │           └── N agents (Qwen3.5-9B)
  │                 ├── linux1:8080 (本机) — Vulkan GPU, parallel=5+, ~38 tok/s
  │                 └── Mac:8080 (LAN)    — Apple M4, parallel=2-5, ~33 tok/s
  ├── audit/autoresearch_v4_phase2.tsv    — Ground Truth experiment log
  └── logs/                               — 每轮完整日志

omega-vm (GCP) — 代码仓库 + Git + 远程监控
Win1 (192.168.3.112) — llama-server 备用节点 (Vulkan GPU, 40 tok/s)
```

## 成功/失败判据

| 结果 | 含义 | 行动 |
|------|------|------|
| ERS > 0.1 (depth ≥ 7) | 系统在学习深度推理 | 继续优化 |
| ERS > 0.3 (depth ≥ 12) | 接近完整证明 | 启动 DeepSeek Oracle 验证 |
| ERS = 0 持续 10+ 实验 | 系统性失败 | 检查根因 (模型能力? prompt? 机制?) |
| [COMPLETE] at P≥90% | 证明完成候选 | Oracle 验证 → 如果通过 = OMEGA |
