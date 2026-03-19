# Architect Directive: 反奥利奥架构 — 粉碎行为经济学中央干预，恢复纯粹自由定价
**Date**: 2026-03-19
**Source**: 首席架构师 (0x00 — 终极审判 + 三界大一统)
**Trigger**: Coder Agent 提出 `conservative_stake = balance * 0.01` 被架构师 ABSOLUTE VETO
**Classification**: Philosophy — 三界辖区划定 + 自由定价权捍卫

---

## 0x00 核心裁决

### VETO: `conservative_stake = balance * 0.01` 的 "1% 保守建议价"

**理由（三条）**:

1. **伪造的价格发现 (Fake Price Discovery)**: 从无脑抄写 `500` 变成无脑抄写 `balance * 0.01`，大模型依然没有进行任何主观价值判断。这是"锚定效应诱导的中央计划经济"。

2. **剥夺风险定价权 (Deprivation of Risk Pricing)**: 如果大模型对某步有 100% 确信（如极简化简 `ring`），它理应梭哈 2000 币。用系统的力量把它"按"在 1% 区间，抹杀了极值天才（Blue-chips）的涌现。

3. **保姆心态 (The Nanny State)**: 如果 LLM 连数字都不会填，导致 parse 为 0.0 触发 VETO 斩杀，那它活该承受物理痛觉。市场不需要保姆，市场需要残酷的自然淘汰。

### 架构定性: 越界罪 (Boundary Violation)

`swarm.rs`（顶层白盒 Orchestrator）用 Rust 代码计算 `conservative_stake` 并塞进 `delta`（中间黑盒）的 Prompt → **顶层白盒用代码霸权侵犯中间黑盒的自由定价主权**。

## 0x01 反奥利奥架构 — 三界封神榜

### 第一界：顶层白盒 (Top-Level Whitebox) — 绝对独裁的真理法庭与央行
- 映射: Event Bus + TOOLs (MembraneGuardTool, WalletTool)
- 治理: **硬限制 (Hard Limits) + 零自由度**
- 绝对不能用 Prompt 去"求"大模型
- 钱包余额不足或编译 Error → VETO + 质押烧毁
- 度量黑洞：打分逻辑对大模型绝对保密

### 第二界：中间黑盒 (Middle Blackbox) — 充满高熵的图灵资本市场
- 映射: LLM Agents + SKILLs (Markdown 思想钢印)
- 治理: **软引导 (Soft Prompts) + 绝对自由度**
- **严禁 Rust 代码干预定价行为**
- 只能通过 SKILL(Markdown) 告知经济法则
- 大模型享有完全行动自由，但承担被顶层白盒爆仓的后果

### 第三界：底层白盒 (Bottom-Level Whitebox) — 冷酷无情的物理时空基座
- 映射: Kernel (Append-Only DAG, 时间之箭)
- 治理: **数学拓扑的硬不变量**
- 只要顶层白盒放行，无条件将 output 写入 Tape
- 拒绝环状引用和历史覆写

## 0x02 批准的修复方案

### 批准项:
1. **(1a)** 破产清算 `if balance < 1.0 { continue; }` ✅ — 顶层白盒的本职工作
2. **(3)** JoinSet 空集保护 `if set.is_empty() { break; }` ✅
3. **(4)** 修复 "paradox" fallback payload ✅
4. **Prompt 模板**: 用 `<FLOAT>` 占位符替代硬编码 500 ✅ — 中间黑盒的主权归还

### 否决项:
1. **(1b)** `let conservative_stake = (balance * 0.01).max(1.0)` ❌ — 越界罪
2. **(1c)** `Amount: {conservative_stake:.2}` 作为默认值注入 Prompt ❌ — 锚定效应
3. Prompt 中注入 `CONSERVATIVE EXPLORATION STAKE: {:.2}` ❌ — 央行施舍

### 新增要求:
- Prompt 末尾格式: `Amount: <FLOAT>` — 抽象占位符，逼迫 LLM 自主生成价格
- SKILL 中增加 WARNING: "Never output the literal text `<FLOAT>`. You must type a REAL DECIMAL NUMBER."
- SKILL 示例必须展示极端方差: 12.5 ~ 1500.0

## 0x03 正确修复的边界映射

| 修复 | 所在界 | 手段 | 合规性 |
|------|--------|------|--------|
| `if balance < 1.0 { continue; }` | 顶层白盒 | Rust 硬限制 | ✅ 正确 |
| `<FLOAT>` 占位符 | 中间黑盒边界 | Prompt 软引导 | ✅ 正确 |
| SKILL 方差示例 (12.5~1500.0) | 中间黑盒 | Markdown 思想钢印 | ✅ 正确 |
| `set.is_empty()` 保护 | 顶层白盒 | Rust 硬限制 | ✅ 正确 |
| `"paradox"` → 安全 payload | 顶层白盒 | Rust 修复 | ✅ 正确 |
| ~~`conservative_stake = balance * 0.01`~~ | ~~越界~~ | ~~Rust 干预定价~~ | ❌ VETOED |
