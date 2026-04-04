---
date: 2026-04-04
status: proposed
related_commits: []
---

## 原话
> reasoner不能改宪法，大宪章，re-init前必须对齐，由rust硬性检查

## 浓缩
re-init 前宪法对齐，Rust 硬性守门

## 架构含义

### 三条规则
1. **Reasoner 不可改宪法** — Reasoner 编辑 skill.txt/context.txt 时，不可引入违反大宪章 (Law 1-3) 或 Engine 分离原则的内容
2. **re-init 前必须对齐** — Reasoner 发起 re-init 时，必须先通过宪法合规检查，确认当前状态不违宪
3. **由 Rust 硬性检查** — 不是 Python prompt 层面的"请求"，是 evaluator.rs 编译时硬编码的守门逻辑

### 实现层级
- Python (sweep_v4.py): Reasoner prompt 告知规则 → 软约束（可被绕过）
- **Rust (evaluator.rs): 硬性拦截 → 不可绕过**（架构师要求的层级）

### 需要 Rust 检查的点
1. **prompt 文件加载时**: evaluator 启动时读 prompt/，检查内容不含违宪模式
   - 不含 Lean 4 语法 (Rule 22 已有 bus.rs Phase 0 拦截)
   - 不含绕过市场机制的指令 (如 "ignore price", "always invest")
   - 不含修改 kernel 行为的指令
2. **re-init 信号**: sweep_v4.py 发起 re-init 前，调用 evaluator 的宪法检查模式
   - `evaluator --constitutional-check` 返回 PASS/FAIL
   - FAIL 则阻止 re-init，Reasoner 必须先修复违规

## 行动项
- [ ] evaluator.rs: 添加 `--constitutional-check` 模式，检查 prompt/ 目录内容合宪
- [ ] sweep_v4.py: re-init action 处理中，先调用宪法检查，PASS 才执行
- [ ] 定义"违宪模式"清单（可复用 scripts/constitutional_check.sh 的逻辑，移植到 Rust）
