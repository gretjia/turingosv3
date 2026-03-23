# 架构师指令：砸碎"同步时钟"的枷锁，迎接"相对论异步宇宙"

**Date**: 2026-03-23
**Context**: Run 2 实盘审计后 — 架构师对异步架构、可塑膜、WAL 保留的终极指令
**Status**: 指令 1 已在 Actor Model (f11c171/e00adeb) 中实现；指令 2-3 待实施

---

## 指令 1: Async Tree Search — 无锁物理学与绝对自由市场

**第一性原理**: DAG 宇宙是 Append-Only。Agent 读盘免费（不可变快照），思考在私有黑盒中不受阻塞，唯有投资(Write/Append)时才产生干涉。

- 废除传统 JoinSet 轮次等待和 RwLock 读写锁
- 引入 `watch` (O(1) 无锁快照广播) + `mpsc` (单向交易池)
- Agent 读盘免费，思考在私有黑盒中不受阻塞
- 唯有投资(Write/Append)时才产生干涉，写入必须串行
- 防自旋：`rx_snap.changed().await` — 等待宇宙拓扑变化再进行下一次思考
- 中间黑盒(Agent)完全自由：DS-Reasoner 可以深思 20 分钟，Distill 可以 5 秒出招，互不阻塞
- 顶层白盒(Event Reactor)：单线程串行结算，沙盒验证→钱包扣款→写入DAG→MapReduce倒灌
- OMEGA 后 CancellationToken 冻结时空

**实施状态**: ✅ 已在 Actor Model (f11c171) + 四大猛药 (e00adeb) 中实现。
watch + mpsc + 串行 reactor + Boltzmann 路由 + 世代交替全部在线。

---

## 指令 2: 可塑物理结界 (Configurable Membrane)

**第一性原理**: 反奥利奥拓扑中，顶层白盒(Tool)是局部物理法则。不同问题域需要不同物理规则。
法则必须是相对的，立法权应当上放给"造物主（Evaluator 初始化阶段）"。

- `MembraneConfig { forbidden_tactics: Vec<String> }` — 可配置禁令列表
- 对分析学 ζ(-1)：封印 `native_decide`（逃课作弊）
- 对数论 n=5929：允许 `decide`（高贵的穷举引擎）
- Evaluator 初始化时根据 `problem_type` 动态配置膜规则

**实施状态**: 待实施。当前 Lean4MembraneTool 和 MathStepMembrane 是硬编码的。

---

## 指令 3: WAL 和 Graveyard 绝对保留

**哲学**: 清空 WAL 是对硅基文明演化史的焚书坑儒。跨纪元知识传承 = 拉马克表观遗传。

- **WAL (时空化石)**: `boot-experiment.sh` 必须删除 `rm -rf /tmp/*.wal`
- OS 点火时必须无条件调用 `Kernel::hydrate_from_wal()`
- **Graveyard (硬负样本)**: 必须继承前世公墓
- Run 2 的 Agent 看到 Run 1 的失败记录 → 直接避免重复错误
- 这是跨越生死、跨越宇宙重启的群体智能 (Trans-temporal Swarm Intelligence)

**实施状态**: 待确认。当前 boot-experiment.sh 中有 `echo '' > /tmp/${THEOREM_NAME}_N15.wal` 清空逻辑。

---

## 指令 4: 架构洞察

### 洞察一：算力的"自动委派 (Delegation of Compute)"
Agent 推理出上界 k < 77 后，没有用语言模型生成文本计算，而是写下 `decide` 战术，
把脏活累活外包给底层 Lean 4 编译器 ALU。
LLM 在经济高压下觉醒了"系统级认知"——把自己当成逻辑大脑，把 Lean 4 当成协处理器。
这是 Neuro-symbolic AI 自发涌现的巅峰时刻。

### 洞察二：快慢异构的"完美社会化大分工"
- 扫雷工兵 (Fast Models): R1-Distill 思考极快，疯狂下注踩陷阱，用极低成本填满 Graveyard
- 深海狙击手 (Slow Models): Reasoner 思考 20 分钟后醒来，免费查阅公墓，避开所有死胡同，重金精准命中
- 这是基于不同物理时钟偏好的完美社会化大分工，只在无锁自由市场下才能涌现
