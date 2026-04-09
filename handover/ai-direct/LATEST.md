# TuringOS v3 — Handover State
**Updated**: 2026-04-09
**Session Summary**: OMEGA 机制重设计 + 原子化步骤调优实验 × 8 + 苦涩的教训实证

## Current State
- **OMEGA 可复现**: V3.2 N=3 3M/0B+/0B- 50K，PPUT ~2×10⁻⁴，2/2 成功率
- **最佳配置确定**: 零人工规则 + [COMPLETE]协议 + 目标值预过滤 = 稳定 OMEGA
- **代码已回滚到大宪章对齐状态**: 投资 prompt 简洁版，Oracle YES/NO，无 SUBSTANCE/审计
- **未提交的变更**: evaluator.rs (OMEGA 机制 + Oracle/Librarian 独立 key 支持)
- **所有节点已清理**: omega-vm / Mac / Linux1 无残留进程

## Changes This Session

### 修复
1. **Reactor-Agent 死锁修复** (`4be6dec`) — timeout 分支加心跳广播
2. **context.txt 漏题修复** (`4be6dec`) — 删除 "divergent series and regularization"
3. **problem.txt [COMPLETE] 规则** (`4be6dec`) — 添加完成协议
4. **GENESIS_COINS 可配置** (`4be6dec`) — wallet.rs 支持环境变量
5. **Oracle/Librarian 独立 API key** (未提交) — ORACLE_URL/KEY/MODEL + LIBRARIAN_URL/KEY/MODEL

### OMEGA 机制演进
6. **Gate A/B 设计** (`8320c98`) — 去掉 [COMPLETE] 作为必要条件，纯市场触发
7. **目标值预过滤** (`8320c98`) — OMEGA_TARGET_VALUE 环境变量，防 Oracle 脑补
8. **回滚 Gate A/B** (未提交) — 恢复 [COMPLETE]+P≥0.90，保留目标值预过滤
9. **回滚结构化 Oracle 审计** (未提交) — Oracle 回归 YES/NO，不做 [C]/[P] 分类
10. **回滚 SUBSTANCE 投资维度** (未提交) — 投资 prompt 回归简洁版

### 审计
11. **Phase 2 统一审计** (`b713876`) — 651 行，4 实验，DAG 分析，PPUT，泄露审计
12. **14B false OMEGA 发现** — 数值验证证明 14B 的闭合形式公式错误（差 16,000 倍）
13. **Mac 残留进程挖掘** — 74 次 scaling law 实验数据抢救，3 个长跑（125K tx）消耗分析

### 宪法违规发现与修复
14. **Over-Alignment (Rule 20)**: SUBSTANCE 投资维度 + Oracle 结构化审计 = Engine 间职责越界 → 全部回滚
15. **Oracle 做 Engine 2 的事**: [C]/[P] 分类是定价行为不是验证行为 → 回滚到 YES/NO

## Key Decisions

### 1. 苦涩的教训在原子化步骤上的实证
**每一个"改进"都让结果变差：**
- + 步骤质量指令 → 代数错误（-1/2≠-1/12）
- + SUBSTANCE 投资维度 → 推导变慢（30min 不够）
- + Bear (2M/1B-) → 做空正确步骤 / 逼走复杂路线
- + Oracle 结构化审计 → Over-Alignment (Rule 20 违反)
**结论**: 零人工规则 + 市场涌现 = 最优。人工规则扼杀涌现。

### 2. Oracle 是 LLM 而非形式化验证器
- DeepSeek Reasoner 接受了 14B 的错误证明（闭合形式差 16,000 倍）
- DeepSeek Reasoner 接受了 3 步不完整链（用训练数据脑补）
- 目标值预过滤是唯一可靠的防伪层（字符串匹配，零 LLM 依赖）
- 大宪章的终极 Oracle 应是 Lean 4 编译器，LLM Oracle 是临时替代

### 3. 价格信号的两个维度（架构师指令）
高价格应传递：(1) Ground Truth 合规 (2) 目标推进。
当前市场自然地编码了这两个维度（在零人工规则下）。人工添加第三维度（如 SUBSTANCE）反而破坏信号。

### 4. [COMPLETE] 是行为指令而非质量规则
skill.txt 的 PROOF COMPLETION PROTOCOL 不是质量控制，是必要的行为引导。
没有它 → 70+ 次运行零 [COMPLETE]。有它 → agent 自然写出 [COMPLETE]。

### 5. Bear 在 N=3 下效果为负
1 Bear / 3 agents = 1/3 做空权重。Bear 做空正确步骤的概率和做空错误步骤的概率一样高。
Bear 可能在 N≥7 (Bear 权重 ≤ 1/7) 时才有正面效果。

## Experiment Results (PPUT 为唯一评估标准)

| Experiment | Config | OMEGA | PPUT | Quality | Key Finding |
|------------|--------|-------|------|---------|-------------|
| Baseline oneshot | bare V3.2 | N/A | N/A | SHORTCUT 4/5 | 无构造性推导 |
| **OMEGA #1 ★** | **3M 50K** | **YES** | **2.19e-4** | **8.5/10** | **最佳** |
| Exp 1 (14B) | 3M 50K 14B | YES(INVALID) | N/A | WRONG MATH | Oracle 检不出错 |
| Exp 2/2b (质量) | 3M 50K | NO | N/A | -1/2 错误 | 细粒度→错误累积 |
| **Exp 3 (复现)** | **3M 50K** | **YES** | **2.00e-4** | **6/10** | **OMEGA 可复现** |
| Exp 4 (SUBSTANCE) | 3M 50K | NO | N/A | 100% substantive | 推导变慢 |
| Exp 5b (Bear+规则) | 2M/1B- 50K | NO | N/A | Bear 做空 [COMPLETE] | C∩P90=0 |
| Exp 6 (Bear 干净) | 2M/1B- 50K | NO | N/A | sinh² 弯路 | Bear 逼走复杂路线 |

## Next Steps
1. **Commit 当前回滚** — evaluator.rs 的大宪章对齐版本
2. **N=5 实验** — 3M/1B+/1B-，测试 Bear 在更大 swarm 中的效果
3. **PPUT 跨 N 曲线** — 用最佳配置 (3M 50K) 测 N=3,5,7 的 PPUT
4. **换题测试** — 用不同的数学问题验证 PPUT 是否问题无关
5. **Lean 4 Oracle** — 替代 LLM Oracle 实现真正的形式化验证
6. **[OPEN SPRINT] bus.rs tick_map_reduce 重构**
7. **[OPEN SPRINT] math_membrane.rs 语义断头台对齐**

## Warnings
- **Oracle (DeepSeek Reasoner) 不可靠**: 接受了 14B 错误证明 + 3 步不完整链。OMEGA_TARGET_VALUE 预过滤是必须的
- **Mac 上不要遗留进程**: 本次发现 12 个 evaluator + 4 个 proxy 遗留 40 小时消耗 ~3 亿 tokens
- **实验间不要 pkill proxy**: 只 reset stats，不杀进程。否则下一个实验静默失败
- **14B 的 OMEGA 是 false positive**: 数学审计证明闭合形式公式差 16,000 倍
- **未提交变更**: evaluator.rs 有大量回滚改动待 commit

## Architect Insights (本次会话)
- **苦涩的教训 × 原子步骤**: 人工规则（质量指令、SUBSTANCE、结构化审计）全部让 OMEGA 成功率从 100% 降到 0%。零规则 + 市场涌现 = 最优。已通过 8 次对照实验实证。
- **价格信号两维度**: 高价格 = Ground Truth 合规 + 目标推进。市场自然编码这两个维度。人工添加第三维度（SUBSTANCE、方法论纯粹性）破坏信号。
- （未归档到 architect-insights/ — 建议下次会话归档）
