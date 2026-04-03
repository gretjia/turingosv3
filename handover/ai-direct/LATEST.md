# TuringOS v3 — Handover State
**Updated**: 2026-04-03
**Session Summary**: 6 大架构升级 + Karpathy AutoResearch v4 框架 + 本地双节点推理 + DeepSeek 数学/经济双审计

## Current State
- **AutoResearch v4: RUNNING** — sweep_v4.py (PID 668993) 在后台运行 Phase 1 (Prompt Search)
  - Baseline 完成: ERS=0, depth=0, 30 appends/10min
  - Exp 1 运行中: DeepSeek 自主编辑了 problem.txt (去掉 toolkit, 要求 depth over restatement)
  - TSV 记录: `experiments/zeta_sum_proof/audit/autoresearch_v4.tsv`
- **本地推理双节点: ONLINE**
  - Mac (18080): Qwen3.5-9B, llama.cpp --parallel 2, ~33 tok/s
  - Win1 (18081): Qwen3.5-9B, llama.cpp --parallel 2 via SSH 持久会话, ~28 tok/s
  - 无 rate limit, 无 API 费用
- **zeta_sum_proof: 未证明** — 但 agents 首次产出真正的代数推导 (Bernoulli 展开, 虚部消去)

## Changes This Session (未提交)

### 1. Frontier Price Gate (actor.rs)
- 子不如父则父不退位: `child.price > parent.price × (1 + α/depth)` 才 mask
- 前沿从 2 节点爆到 190 (α=0 时)，加 depth-boost 后可控
- Source: architect directive 2026-04-02

### 2. Global Dedup (bus.rs)
- 跨分支内容去重: 40-char 前缀全 DAG 唯一
- 754 branch dedup + 跨分支结论复制 → 全局拦截
- Env: GLOBAL_DEDUP=true/false

### 3. Lineage Score + Depth Weight (actor.rs)
- Boltzmann 选择按血统链加权: `score × log(depth+1)^weight`
- BoltzmannParams 全可配 (FRONTIER_CAP, DEPTH_WEIGHT, PRICE_GATE_ALPHA)

### 4. Ground Truth 日志 (librarian.rs + evaluator.rs)
- success.jsonl / failure.jsonl — append-only JSONL, 不可删
- Librarian 从文件读取完整 rejection 分类 (不再被 top 5 截断)
- Source: architect "log = Ground Truth 不可篡改"

### 5. Librarian → DeepSeek V3 管理层 (librarian.rs)
- 压缩引擎从本地规则 → DeepSeek V3 (chat) API 调用
- Oracle (验证) = DeepSeek Reasoner, Librarian (压缩) = DeepSeek Chat
- 每 50 appends 触发, 15/15 agents 更新 learned.md
- `</think>` 标签自动剥离

### 6. Prompt 可变文件 + AutoResearch v4 (evaluator.rs + sweep_v4.py)
- problem.txt / skill.txt / context.txt 从编译时 const → 运行时文件加载
- DeepSeek V3 作为搜索代理: 读实验结果 → 编辑 prompt 或改参数
- 固定 600s wall clock, ERS 单一指标, greedy keep/discard
- Karpathy 核心对齐: "LLM IS the search algorithm"

### 7. Thinking Mode 控制 (llm_http.rs)
- THINKING_MODE env var: on / off / budget:N
- Qwen3.5 的 /no_think 通过 system prompt 注入 (llama.cpp 不支持 API 参数)
- 根因: thinking ON 生成 1000+ 隐藏 tokens → 12 tok/s; OFF → 33 tok/s

### 8. Local LLM Provider (evaluator.rs + llm_http.rs)
- LLM_PROVIDER=local, LLM_URLS 支持多端点 round-robin
- Mac + Win1 双节点并发, agents 自动分配

## Key Decisions
- **Global Dedup 启用**: DeepSeek 经济审计发现 cross-branch 结论复制是主要浪费源
- **Append Cost = 0 不可变**: DeepSeek 建议 dynamic append cost → 否决 (违反 Law 1)
- **Depth Rewards/Bounty 否决**: 违反 Rule 19 (零印钞)
- **Price Decay 否决**: 违反 Law 2 精神 (价格 = 贝叶斯概率)
- **Hint 公式正确**: DeepSeek Reasoner 审计声称 limit=1/2, 手动验证证明是 -1/12 (审计员出错)
- **Thinking OFF 为默认**: 本地推理下 thinking 太慢, 但纳入 AutoResearch 参数空间待研究

## Architect Insights (本次会话)
- **Librarian = 管理层 Agent**: 用最强模型(DeepSeek)压缩 → `2026-04-02_librarian-management-layer.md`
- **Log = Ground Truth**: 不可篡改, 反幻觉锚点 → `2026-04-02_log-as-ground-truth.md`
- **子不如父则父不退位**: Frontier Price Gate → `2026-04-02_frontier-price-gate.md`
- **Log 未履行 Ground Truth 职责**: compression 截断导致信息丢失 → `2026-04-02_log-ground-truth-violation.md`
- **根因分析强制令**: 禁止"可能是", 必须找到根因 → feedback memory

## DeepSeek 双审计结果 (2026-04-03)
- **数学审计**: Grade F — agents 只空断言不推导 (但 hint 公式经手动验证正确)
- **经济审计**: 10/15 破产 = Bulls/Bears 无信息优势; 前沿 190 = Price Gate 太宽松; 资本集中 = 泡沫不是发现; 宽度替代深度因为 append 免费 + 投资回报与深度无关

## Research Plan (Active)
见 `experiments/zeta_sum_proof/RESEARCH_PLAN.md`
- **Phase 1 (当前)**: Prompt Search — DeepSeek 编辑 prompt, ~50 实验过夜
- Phase 2: Thinking Mode sweep
- Phase 3: Economic mechanism tuning
- Phase 4: 27B model scaling

## Next Steps
1. **监控 AutoResearch v4 过夜运行** — 检查 ERS 趋势, prompt 编辑历史
2. **Win1 SSH 持久化** — nohup ssh -t 方式需要改进 (进程可能在隧道断开时死亡)
3. **[OPEN SPRINT] 提交本次会话代码** — 670 行变更待 commit
4. **Phase 2 启动条件**: Phase 1 找到 depth > 10 的 prompt 后切换到 thinking sweep

## Warnings
- **Win1 llama-server 不稳定**: SSH Session 0 无 Vulkan GPU 上下文, 必须用 `nohup ssh -t` 前台方式启动. 隧道断开后进程可能死亡. 详见 `handover/windows1_llama_server_rootcause.md`
- **Qwen3.5 默认 thinking**: 不加 /no_think 则每请求 60s+. THINKING_MODE env var 控制
- **670 行未提交代码**: actor.rs, bus.rs, librarian.rs, evaluator.rs, llm_http.rs 均有重大变更
- **ERS baseline = 0**: depth 从 log 中 grep "deepest chain" 得到, 如果 Librarian 禁用 (LIBRARIAN_INTERVAL=999) 则无 depth 数据 → ERS 恒为 0. 可能需要修复 ERS 计算
