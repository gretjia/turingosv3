# 经济制度模型规模筛查 — Research Spec

**日期**: 2026-04-05
**状态**: DRAFT — 待架构师讨论

## 核心洞察

> 9B 模型不足以在 TuringOS 经济制度下生存。60 轮实验证明：无论怎么调参数，agent 无法有效参与市场 — 要么只做交易不写证明，要么需要 DEPTH_WEIGHT=50 暴力强制才能纵深。

## 研究问题

**TuringOS 的哈耶克经济制度在什么模型规模下才能有效运作？**

具体而言：
1. 模型需要多大才能理解"append 免费，invest 有风险"的经济激励？
2. 模型需要多大才能根据市场价格信号做出理性决策（投资高价节点、做空低价节点）？
3. 模型需要多大才能同时做数学推理 + 经济决策？
4. 是否存在一个"临界规模"，低于它经济制度失灵，高于它突然有效？

## 实验设计

### 自变量：模型规模

模型阶梯（统一用 Qwen 系列保证 prompt 兼容性 + 补充关键对照模型）：

| 层级 | 模型 | 参数 | Provider | API Model ID |
|------|------|------|----------|-------------|
| L0 | Qwen3-0.6B | 0.6B | DashScope | qwen3-0.6b |
| L1 | Qwen3-1.7B | 1.7B | DashScope | qwen3-1.7b |
| L2 | Qwen3-4B | 4B | DashScope | qwen3-4b |
| L3 | Qwen3-8B | 8B | DashScope/SiliconFlow | qwen3-8b / Qwen/Qwen3-8B |
| L4 | Qwen3.5-9B | 9B | **本地 linux1** | local (已测, depth≤2) |
| L5 | Qwen3-14B | 14B | DashScope/SiliconFlow | qwen3-14b / Qwen/Qwen3-14B |
| L6 | Qwen3.5-27B | 27B | SiliconFlow / **本地 Win1** | Qwen/Qwen3.5-27B |
| L7 | Qwen3-32B | 32B | DashScope/SiliconFlow | qwen3-32b / Qwen/Qwen3-32B |
| L8 | Qwen2.5-72B | 72B | DashScope/SiliconFlow | qwen2.5-72b-instruct |
| L9 | Qwen3.5-122B-A10B | 122B MoE | SiliconFlow | Qwen/Qwen3.5-122B-A10B |
| L10 | Qwen3.5-397B-A17B | 397B MoE | SiliconFlow | Qwen/Qwen3.5-397B-A17B |
| — | DeepSeek V3.2 | 670B MoE | DeepSeek API | deepseek-chat |
| — | DeepSeek R1 | 670B MoE | DeepSeek API | deepseek-reasoner |

对照组（非 Qwen，测泛化性）：
| 模型 | 参数 | Provider | 用途 |
|------|------|----------|------|
| Gemma-3-4B | 4B | NVIDIA NIM | L2 对照 |
| Llama-3.3-70B | 70B | NVIDIA NIM | L8 对照 |
| DeepSeek-R1-Distill-Qwen-7B | 7B | SiliconFlow | R1 蒸馏小模型 |
| DeepSeek-R1-Distill-Qwen-32B | 32B | SiliconFlow | R1 蒸馏中模型 |

### 因变量

每轮实验收集：

1. **real_nodes_per_min** — APMM 创建的真实 DAG 节点数 / 时间（分钟）
2. **append_invest_ratio** — append 操作数 / invest 操作数（>1 说明 agent 偏向建树）
3. **price_variance** — 节点价格的方差（>0 说明市场在分化定价）
4. **max_depth** — 最深链
5. **market_quality** — 是否有节点 price > 80%（市场形成了强共识）

### 控制变量（固定，大宪章默认）

| 参数 | 值 | 原因 |
|------|-----|------|
| DEPTH_WEIGHT | **0** | 纯市场定价，无人工偏好 |
| FRONTIER_CAP | **0** | 信息平权 (Law 1) |
| PRICE_GATE_ALPHA | **0** | 纯价格比较 |
| GLOBAL_DEDUP | true | 防重复 |
| THINKING_MODE | **off (强制)** | 所有模型必须关闭 thinking。测的是模型本身的经济智慧，不是 thinking chain 的补偿。Qwen3 系列用 /no_think prefix，DeepSeek Reasoner 不参与此实验（天然 thinking）。 |
| SWARM_SIZE | 15 | 够市场动力学，不至于太慢 |
| MATH_COUNT | 9 | 60% Math |
| BULL_COUNT | 3 | 20% Bull |
| BEAR_COUNT | 3 | 20% Bear |
| problem.txt | 锁定 | 同一道题 |
| skill.txt | 锁定原版 | 同一套工具说明 |

### WALL_CLOCK 设计

**核心矛盾**：小模型快（50 tok/s）大模型慢（5 tok/s），固定 wall clock 不公平。

**方案 A — 固定 wall clock（推荐）**：
- 所有模型跑 **600s (10 min)**
- 理由：市场有固定的"交易日"长度。快模型在同样时间内产出更多 → 这本身就是信息（更快的市场更有效率？还是质量更重要？）
- 优点：简单、可比
- 缺点：大模型可能只完成 5 个 agent round

**方案 B — 固定 agent rounds**：
- 等到每个 agent 完成至少 10 轮决策后停止
- 理由：公平比较每个模型的"决策质量"而非"决策速度"
- 缺点：大模型可能需要 60 分钟

**方案 C — 两阶段**：
- 第一阶段：固定 600s，所有模型。快速筛选。
- 第二阶段：对有潜力的模型（depth > 2），延长到 1800s 验证。

**推荐方案 C** — 先快筛再验证。

### 执行架构

```
model_scaling_sweep.py (新脚本，不是 sweep_v4.py)
  ├── 模型列表 (从小到大)
  ├── 对每个模型:
  │     ├── 配置 evaluator 连接到对应 API
  │     ├── 跑 600s 实验
  │     ├── 收集 5 个指标
  │     └── 记录到 model_scaling_results.tsv
  ├── 完成后生成对比报告
  └── 识别"临界规模"拐点
```

### evaluator 如何切换模型

evaluator.rs 已支持多 provider (`LLM_PROVIDER=local|aliyun|siliconflow|deepseek`)。
对每个模型，设置:
- `LLM_PROVIDER` = siliconflow / aliyun / deepseek / local
- `LLM_MODEL` = 模型 ID
- `LLM_URL` = API endpoint

### 预期结果

| 规模 | 预期行为 |
|------|---------|
| 1-3B | 经济制度完全失灵。agent 输出垃圾，无法理解市场。depth=0 |
| 7-9B | 部分理解。能写简单数学但不理解经济激励。depth=1-2（已验证） |
| 14-32B | **假设的临界区间**。开始理解"高价节点值得继续推进" |
| 70B+ | 经济制度有效。agent 能同时做数学和经济决策。depth >> 5 |
| DeepSeek V3 | Run 6 用 7B 达到 depth=18 — 但那是透题+云端无限并行 |

### 关键对照

- **Run 6 对照**：Qwen2.5-7B + 透题 prompt + 默认经济参数 (30/1.0/0.05) → depth=18
- **本次实验**：各规模模型 + 不透题 prompt + 纯市场经济 (0/0/0) → depth=?

如果 70B 在纯市场下也只能 depth=2，说明问题不在模型规模，而在经济制度本身（需要非零的 DEPTH_WEIGHT/FRONTIER_CAP 来补偿）。
如果 70B 在纯市场下达到 depth=10+，说明 9B 确实不够聪明参与市场。

## 预算估算

| 模型 | 每轮成本 | 轮数 | 总计 |
|------|---------|------|------|
| SiliconFlow 小模型 (1-7B) | ~$0.01 | 各 1-2 轮 | ~$0.05 |
| SiliconFlow 大模型 (14-72B) | ~$0.10 | 各 1-2 轮 | ~$0.50 |
| DashScope Qwen (各规模) | ~$0.05-0.50 | 各 1-2 轮 | ~$2.00 |
| DeepSeek V3 | ~$0.20 | 2-3 轮 | ~$0.60 |
| 本地 (9B, 27B) | 免费 | 各 2 轮 | $0 |
| **总计** | | ~20-30 轮 | **~$3-5** |

## 待讨论

1. 方案 C（两阶段）是否合适？
2. SWARM_SIZE=15 是否太多（对慢模型可能排队严重）？
3. 是否需要对每个模型跑 3 次取中位数（减少随机性）？
4. 本地 27B (Win1) 是否加入对比？
5. 是否同时测试 thinking mode（对 Qwen3.5/DeepSeek R1 有效）？
