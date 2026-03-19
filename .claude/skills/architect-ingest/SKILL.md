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
