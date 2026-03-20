---
name: architect-ingest
description: Ingest architect directives — archive fully, detect axiom impact, wait for authorization before executing
user_invocable: true
---

# /architect-ingest — Architect Directive Ingestion

When receiving architect directives (bible.md updates, new planning_directive, philosophical architecture decisions).

**Core Principle: 接收指令 ≠ 授权执行。Archive and analyze only — do NOT execute code changes.**

## Procedure

### 1. Full Archive (不可压缩)
Save the complete directive to `handover/directives/YYYY-MM-DD_<topic>.md`:
- Source document identifier
- Complete directive content (every item, no omissions)
- Design philosophy and rationale

### 2. Axiom Impact Detection (自动)

#### Layer 1 Detection (永恒不变量)
Check if directive affects:
- kernel.rs 零领域知识
- Append-Only DAG
- SKILL-only reward minting
- Stake >= 1.0

If any Layer 1 invariant would be violated → **VIOLATION**
Archive the directive but DO NOT execute.

#### Layer 2 Detection (可演进参数)
Check if directive affects:
- 并发度 N
- Boltzmann 温度
- Anti-Zombie 阈值
- 模型选择

If Layer 2 parameters change → **UPDATE REQUIRED**
Requires user confirmation before applying.

### 3. Output Axiom Impact Rating

```
=== ARCHITECT DIRECTIVE IMPACT ===
Directive: [title]
Archived:  handover/directives/YYYY-MM-DD_<topic>.md

Layer 1 Impact: NO IMPACT / VIOLATION
Layer 2 Impact: NO IMPACT / UPDATE REQUIRED

Affected Files:
- [file1]: [what changes]
- [file2]: [what changes]

Rating: NO IMPACT / UPDATE REQUIRED / VIOLATION

=== AWAITING USER AUTHORIZATION ===
```

### 4. Wait for User Confirmation
- If NO IMPACT: inform user, no action needed
- If UPDATE REQUIRED: list specific changes, wait for explicit "proceed"
- If VIOLATION: explain which invariant is violated, recommend discussion with architect

### 5. Execute (only after confirmation)
Apply changes only after receiving explicit user authorization.

---

## Branch B: 口头洞察捕获模式

When the input is a verbal architect insight (not a full directive document) — a philosophical principle, design analogy, or non-obvious architectural constraint:

### 1. 提取原话
Extract the architect's exact words as a quote.

### 2. 压缩本质
Condense to one sentence (≤50 characters) capturing the core principle.

### 3. 写入归档
Create `handover/architect-insights/YYYY-MM-DD_<topic-slug>.md` using this template:

```markdown
---
date: YYYY-MM-DD
status: proposed | implemented | deferred
related_commits: []
---

## 原话
> [exact architect quote]

## 浓缩
[one-sentence essence, ≤50 chars]

## 架构含义
- Layer 1/2 impact analysis
- Constraints or extensions to existing mechanisms

## 行动项
- [ ] Concrete implementation steps (if any)
```

### 4. 更新 Memory 引用
If the insight introduces a new category not yet referenced in memory, update the memory index.

### 5. 呈现确认
Show the condensed insight to the user for confirmation before finalizing.

**识别信号**: 架构师使用类比、哲学引用、或对现有机制提出本质性重新解读时，触发此分支。
