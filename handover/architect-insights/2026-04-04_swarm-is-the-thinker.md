---
date: 2026-04-04
status: proposed
related_commits: []
---

## 原话
> 1、Phase 2 (thinking mode) 放弃 — TuringOS 本身就是 thinking machine，不需要开启个体 agent 的 thinking 模式。
> 2、Phase 3 经济参数大部分不需要扫：a) 前沿节点不需要限制。b) 没有理由对深链有偏好。c) 只需要价格机制就可以做到。d) 全局去重必须全开。
> 3、Swarm 数量测试需要保留，推到硬件上限。
> 5、子节点退位机制目前不完善，需要和 AutoResearch 一起研究如何预防过早退位问题。

## 浓缩
群体即思考机器，个体只需快；价格是唯一裁判

## 架构含义

### 哲学核心
TuringOS 的 "thinking" 发生在 **swarm 拓扑层** (多 agent 协作 + 市场定价)，而非个体 agent 内部。让个体 agent 开启 thinking mode 是对系统层级的混淆 — 等于让神经元自己"思考"，而大脑的思考是神经元集体行为的涌现。

### Layer 1 影响: NO IMPACT
- 不触及 kernel.rs 零领域知识
- 不触及 Append-Only DAG
- 不触及 CTF 守恒

### Layer 2 影响: UPDATE REQUIRED
1. **THINKING_MODE** — 永久固定为 "off"，从可调参数中移除
2. **FRONTIER_CAP** — 设为 0 (无限)，从可调参数中移除
3. **DEPTH_WEIGHT** — 设为 0，从可调参数中移除。市场价格自然选择深链，人工偏好是干预
4. **PRICE_GATE_ALPHA** — 保留为唯一经济调参杠杆
5. **GLOBAL_DEDUP** — 永久 true，从可调参数中移除
6. **SWARM_SIZE** — 提升为 Phase 3 唯一主轴，推到三节点硬件上限 (30+ slots)

### 新研究方向: 子节点过早退位问题
**问题**: 市场可能过早抛弃看似无望但实际有价值的证明分支。数学证明的中间步骤往往看起来不直观（"丑陋的变换"），但最终导向正确结论。如果市场在第 2-3 步就因低价格而让 agent 放弃该分支，则永远无法发现第 7 步的突破。

**核心矛盾**: 市场效率 vs 探索耐心。高效市场快速淘汰低价节点（好），但数学证明需要耐心走完反直觉的路径（坏）。

**研究方向**:
- 退位延迟: 节点需要存活 N 轮后才能被判"死亡"
- 最低流动性保障: 系统做市商持续为所有节点提供最低买单
- 随机复活: Boltzmann 采样偶尔随机选择低价节点延伸
- 批量评估: Librarian 定期重新评估被弃节点的潜力

## 行动项
- [ ] AutoResearch Plan: 删除 Phase 2, 简化 Phase 3 为纯 swarm scale + 退位机制研究
- [ ] sweep_v4.py: THINKING_MODE="off" 固定, FRONTIER_CAP=0, DEPTH_WEIGHT=0, GLOBAL_DEDUP="true" 固定
- [ ] 新研究问题: 设计子节点退位机制实验 (AutoResearch 可测指标: 被弃节点中有多少本可延伸到 depth+2)
- [ ] evaluator.rs: 审计当前退位逻辑，找到"过早退位"的具体代码路径
