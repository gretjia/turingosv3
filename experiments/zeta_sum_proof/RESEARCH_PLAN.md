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

### Phase 1: Baseline + Prompt Search (当前)

sweep_v4.py 运行中。DeepSeek 作为搜索代理:
1. 跑 baseline (10 min)
2. DeepSeek 读结果 → 编辑 prompt 或改参数
3. 跑实验 → ERS 提升则保留，否则回滚
4. 重复

**预期**: ~6 实验/小时。过夜跑 ~50 实验。第一晚应能找到让 depth > 10 的 prompt。

### Phase 2: Thinking Mode Sweep (baseline 稳定后)

在 Phase 1 找到最佳 prompt 后，固定 prompt，扫 THINKING_MODE:
- off → budget:800 → budget:1500 → on
- 对比同一 prompt 下的 ERS

### Phase 3: Mechanism Tuning (prompt + thinking 固定后)

固定 prompt + thinking，扫经济参数:
- FRONTIER_CAP, DEPTH_WEIGHT, PRICE_GATE_ALPHA

### Phase 4: Model Scaling (所有小模型参数固定后)

在 Win1 上启动 27B，测试 heterogeneous swarm。

## 预期时间线

| Phase | 时间 | 实验数 | 关键产出 |
|-------|------|--------|---------|
| Phase 1 | 今夜-明天 | ~50 | 最佳 prompt，depth > 10 |
| Phase 2 | 明天 | ~10 | 最佳 thinking mode |
| Phase 3 | 后天 | ~20 | 最佳经济参数组合 |
| Phase 4 | 视情况 | ~10 | 27B vs 9B 对比 |

## 架构

```
omega-vm (GCP, 编排)
  ├── sweep_v4.py                    — AutoResearch loop (NEVER STOP)
  │     ├── DeepSeek V3 (搜索代理)   — 读结果 → 编辑 prompt/config
  │     └── evaluator binary         — 10 min 固定预算
  │           ├── prompt/ 目录        — THE MUTABLE ARTIFACT
  │           │     ├── problem.txt
  │           │     ├── skill.txt
  │           │     └── context.txt
  │           └── 10 agents (Qwen3.5-9B)
  │                 ├── Mac:18080     — llama.cpp, --parallel 2, ~33 tok/s
  │                 └── Win1:18081    — llama.cpp, --parallel 2, ~28 tok/s
  ├── audit/autoresearch_v4.tsv      — Ground Truth experiment log
  └── logs/v4/                       — 每轮完整日志
```

## 成功/失败判据

| 结果 | 含义 | 行动 |
|------|------|------|
| ERS > 0.1 (depth ≥ 7) | 系统在学习深度推理 | 继续优化 |
| ERS > 0.3 (depth ≥ 12) | 接近完整证明 | 启动 DeepSeek Oracle 验证 |
| ERS = 0 持续 10+ 实验 | 系统性失败 | 检查根因 (模型能力? prompt? 机制?) |
| [COMPLETE] at P≥90% | 证明完成候选 | Oracle 验证 → 如果通过 = OMEGA |
