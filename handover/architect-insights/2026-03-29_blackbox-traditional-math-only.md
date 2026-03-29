---
date: 2026-03-29
status: proposed
related_commits: []
---

## 原话
> 禁止输出任何lean语法的回答，必须用传统数学。

## 浓缩
黑盒推理用传统数学，Lean仅在OMEGA翻译

## 架构含义

### 核心原则
LLM Agent (黑盒) 的输出必须是传统数学推理 (自然语言/LaTeX)，绝不允许输出 Lean 4 语法。
Lean 4 编译仅在 Engine 3 (OMEGA Guillotine) 层发生，由独立的翻译器负责 math → Lean。

### 对齐《苦涩的教训》
- LLM 擅长: 数学直觉、策略选择、结构洞察
- LLM 不擅长: 形式语法、类型匹配、tactic 链组装
- 让黑盒做它擅长的事，机械翻译交给机器

### 对齐四大引擎分离 (CLAUDE.md #20)
- Engine 1 (认识论): Agent 用传统数学自由思考 → append 自然语言数学步骤
- Engine 2 (资本): 市场对"数学质量"定价，而非"语法正确性"定价
- Engine 3 (断头台): OMEGA 时翻译 math → Lean → 编译验证
- Engine 4 (物种): Agent 按数学策略分化，而非按 Lean 语法能力分化

### Layer 1 影响: NO IMPACT
- kernel.rs: 不关心 payload 格式 (零领域知识 ✓)
- DAG: 仍然 append-only ✓
- CTF 守恒: 不受影响 ✓

### Layer 2 影响: MAJOR UPDATE REQUIRED
- 评估器 prompt 模板: 从"write Lean tactic"改为"write math reasoning step"
- Lean4Oracle on_pre_append: sorry/forbidden 检查需适配 (自然语言无 Lean 关键词)
- OMEGA 流程: 需要新增 math → Lean 翻译层 (可用 LLM 或规则引擎)
- Tape 格式: 从 Lean tactic 变为传统数学步骤
- 市场定价语义: P_yes 反映数学正确性概率而非编译成功概率

### Run 4 教训佐证
Gemini 数学审计发现: Agent 尝试的 3 条路径全部在 Lean 语法层面失败 (norm_num 超时、simp_rw 类型不匹配)，
但数学方向有 2 条是 PROMISING。如果 Agent 用传统数学表达，市场可以对数学质量而非语法质量定价。

## 行动项
- [ ] 重写 evaluator.rs 的 SYSTEM_PROMPT: 要求传统数学输出
- [ ] 新增 math → Lean 翻译层 (OMEGA 前调用)
- [ ] 适配 Lean4Oracle 安全检查 (pre-append 不再检查 Lean 关键词)
- [ ] 更新 CLAUDE.md 加入 "黑盒禁 Lean" 规则
- [ ] 设计 Tape 格式规范 (传统数学步骤的结构化模板)
