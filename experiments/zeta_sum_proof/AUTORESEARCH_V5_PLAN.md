# AutoResearch v5 — 从第一性原理重新设计

**日期**: 2026-04-04
**状态**: 待架构师讨论

## 教训与正确归因

### 参数 0 是宪法正确的
- **FRONTIER_CAP=0**: Law 1 信息平权 — 前沿上限 = 限制 agent 访问节点 = 信息垄断。所有节点平等可达，价格自然引导。
- **DEPTH_WEIGHT=0**: 深度偏好 = 人为告诉 agent 选深节点，违反"价格是唯一裁判"。深节点有价值则市场定高价。
- **PRICE_GATE_ALPHA=0**: α>0 让高价父节点永不退位 = 阻断市场流动性。纯价格比较才是最干净的市场。

### 真正的失败原因: 时间和吞吐不够
- Run 6 (OMEGA, depth=18): **50 分钟, 90 agents, SiliconFlow 云 API (无限并行), 6000 tx**
- 我的 Phase 3: **10 分钟, 10-30 agents, 10 parallel slots, ~50 tx**
- 参数 0 是对的。Run 6 成功不是因为参数=默认值，是因为足够的时间和交易量让市场有机会运作。
- **因果归因错误**: 我把 depth=0-1 归因于参数=0，实际上是 50 tx 的市场根本来不及形成价格信号。

## 大宪章重读理解

### 不可变
- Law 1: append 免费 (黑盒使用白盒工具零成本)
- Law 2: 只有投资消耗货币 (资本 = 拓扑确信度)
- Law 3: 独立 Skill 路径 (物种演化)
- topology.md: Turing Machine 结构 (Q_t → δ → Q_{t+1} → predicates → write/rollback)

### 可探索 (Layer 2)
- FRONTIER_CAP (默认 30)
- DEPTH_WEIGHT (默认 1.0)  
- PRICE_GATE_ALPHA (默认 0.05)
- SWARM_SIZE (默认 15)
- 角色比例 (默认 M/B+/B- 等分)
- Boltzmann 温度 T (默认 0.5)
- LIBRARIAN_INTERVAL
- Prompt (skill.txt, context.txt — problem.txt 题目锁定)
- WALL_CLOCK (当前 600s，但 Run 6 用了 50 分钟才达到 depth=18)

## 关键发现: Run 6 为什么成功

| 因素 | Run 5 (失败) | Run 6 (OMEGA) | 当前 Phase 3 |
|------|-------------|--------------|-------------|
| Agents | 90 | 90 | 10-30 |
| 交易量 | 300 | **6000** | ~50 |
| 时间 | ~1 min | **~50 min** | 10 min |
| 市场参数 | 默认 | 默认 | 全部=0 |
| Depth | 5 | **18** | 0-1 |

**结论**: depth 的关键不是 agent 数量，是**交易量（时间 × 吞吐）**。Run 5 和 Run 6 同样 90 agents，但 Run 6 给了 20x 的交易预算 → depth 从 5 跳到 18。

## v5 方案

### 核心改变: 交易预算是第一维度

不再固定 WALL_CLOCK=600s。让 Reasoner 自由探索运行时间:
- 短跑 (600s): 快速迭代，验证参数组合
- 中跑 (1800s): 验证有前途的配置
- 长跑 (3600s+): 推向 OMEGA

### Reasoner 可探索的完整参数空间

| 参数 | 默认值 | 探索范围 | 宪法约束 |
|------|--------|---------|---------|
| WALL_CLOCK_SECS | 600 | 300-7200 | 无 |
| SWARM_SIZE | 10 | 5-90 | 受 parallel slots 限制 |
| MATH/BULL/BEAR_COUNT | 等比 | 任意组合 | 总和 = SWARM_SIZE |
| FRONTIER_CAP | 0 | Reasoner 自由 | 0 = 宪法默认 (信息平权) |
| DEPTH_WEIGHT | 0 | Reasoner 自由 | 0 = 宪法默认 (价格是唯一裁判) |
| PRICE_GATE_ALPHA | 0 | Reasoner 自由 | 0 = 宪法默认 (纯价格比较) |
| LIBRARIAN_INTERVAL | 8 | 4-30 | 须在预算内触发 |
| skill.txt / context.txt | 当前 | Reasoner 自由编辑 | 不违宪 (Rust+Gemini 审查) |
| problem.txt 第一行 | 锁定 | **不可改** | 不透题 |

### 宪法守卫 (双重)

1. **Rust 硬检查** (evaluator --constitutional-check): 模式匹配 (Lean 语法、绕过市场关键字、Engine 越界等)
2. **Gemini 语义审计** (gemini-2.5-flash via Mac proxy): 语义级违宪检测

无参数硬下限 — 所有 Layer 2 参数由 Reasoner 自由探索。

### Re-init 机制 (Markov)

```
Reasoner 判断当前 Life 卡住
  ↓
Rust --constitutional-check → PASS?
  ↓
Gemini 语义审计 → PASS?
  ↓
压缩当前 Life 记忆 (best ERS, what worked, what failed)
  ↓
覆盖写入 prev_life_memory.json (Markov: N-1 only)
  ↓
重置为 topology.md 初始状态
  ↓
新 Life baseline，Reasoner 读取前世记忆
```

Reasoner 自主判断何时 re-init，无硬编码阈值。

### 硬件拓扑

```
linux1 (192.168.3.113) — 编排 + 主力推理
  ├── llama-server :8080 — Vulkan, 108GB VRAM, parallel=5→20
  └── ~38 tok/s

Mac (192.168.3.93:8080) — 辅助推理
  ├── llama-server :8080 — Apple M4, parallel=5
  └── ~33 tok/s

Win1 (192.168.3.112:8081) — 备用推理 (optional)
  ├── llama-server :8081 — Vulkan, parallel=5
  └── ~40 tok/s
```

linux1 parallel 需要提升:
- 当前 5 → 目标 20 (108GB VRAM，9B 模型只用 5GB)
- 需要重启 llama-server with --parallel 20

### 执行计划

**Step 1 (立即)**: 提升 linux1 parallel + 延长运行时间
- linux1 llama-server --parallel 20 (108GB VRAM, 9B 模型只用 5GB)
- WALL_CLOCK 从 600s 提升到 1800s (30 min)，给市场足够时间形成价格信号
- 保持 FRONTIER_CAP=0, DEPTH_WEIGHT=0, PRICE_GATE_ALPHA=0 (宪法默认)

**Step 2**: 让 Reasoner 自由搜索
- 所有 Layer 2 参数可调
- WALL_CLOCK 可调 (600-3600)
- Prompt (skill.txt/context.txt) 可编辑
- problem.txt 锁定

**Step 3**: 如果 Reasoner 卡住 → re-init (带 Markov 记忆)
- 三重宪法守卫
- Reasoner 自主判断时机

**Step 4**: 推向长跑
- 一旦找到有前途的配置 (depth > 5)，延长 WALL_CLOCK 到 1800-3600s
- 参考 Run 6: 50 分钟达到 OMEGA

### 成功标准

| 里程碑 | 含义 |
|--------|------|
| depth > 5 | 市场机制恢复正常 |
| depth > 10 | 接近完整证明 |
| depth > 15 | 启动 Oracle 验证 |
| OMEGA | 证明完成 |

## Meta-Harness 论文关键发现 (2603.28052v1)

1. **Raw traces >> summaries**: Reasoner 看 ERS 分数 (压缩摘要) vs 看原始实验日志 (raw traces) 有 15+ 准确率差距。当前 Reasoner 只看 `[ERS=0.002] depth=1 appends=51` — 损失了关键诊断信号（为什么 agent 卡在 depth=1? 是 prompt 问题还是市场问题?）。
   - **改进**: 在 Reasoner prompt 中注入原始日志片段（最深链的 append 内容、market 价格变化、被弃节点等），而非仅 ERS 数字。参考 Meta-Harness: 每轮 ~82 文件的 selective reading。

2. **LLM 就是搜索算法**: 不需要人工设计探索/利用 trade-off。让 Reasoner 访问完整历史，它自己会判断何时转向。

3. **Additive > Aggressive rewrite**: 一次改一件事。6 次连续回退说明"同时改多个东西"是错误策略。

## 待讨论

1. **WALL_CLOCK 放宽**: 长跑 (3600s) 意味着每轮实验 1 小时，搜索效率下降。建议 Reasoner 先用短跑确定方向，再用长跑验证。
2. **linux1 parallel=20**: 需要重启 llama-server，当前 sweep 会中断。
3. **Win1 是否加入**: 之前 WSL2 有网络问题。可以用 Win1 的 llama-server 作为第三端点（如果防火墙已开放）。
4. **Gemini API 配额**: 每次 re-init 调一次 Gemini。如果 re-init 频繁（比如每 5 轮），是否会超免费配额？
