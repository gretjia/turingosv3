---
name: handover-update
description: Update handover/ai-direct/LATEST.md with current session state before ending
user_invocable: true
---

# /handover-update — Session Handover

Mandatory before ending a session. Ensures continuity for the next conversation.

## Procedure

### 1. Gather State
- Read current `handover/ai-direct/LATEST.md`
- Run `git log --oneline -10` for recent commits
- Run `git diff --stat` for uncommitted changes
- Review key decisions made in this session

### 2. Draft Update
Update `handover/ai-direct/LATEST.md` with:

```markdown
# TuringOS v3 — Handover State
**Updated**: YYYY-MM-DD
**Session Summary**: [one-line description]

## Current State
- [What works now]
- [What's broken/incomplete]
- [Active experiment status]

## Changes This Session
- [Change 1 with commit ref if applicable]
- [Change 2]
- ...

## Key Decisions
- [Decision 1]: [Rationale]
- ...

## Next Steps
1. [Immediate priority]
2. [Secondary tasks]
3. ...

## Warnings
- [Anything the next session MUST be aware of]
```

## Architect Insights (本次会话) — 必填 Section

Before presenting the draft, check `handover/architect-insights/` for files created during this session. Include in the handover:

```markdown
## Architect Insights (本次会话)
- [洞察主题]: 一句话浓缩 → 已归档到 handover/architect-insights/YYYY-MM-DD_xxx.md
- ...
（如无新洞察，标注「本次会话无新架构洞察」）
```

### 3. Present for Review
Show the draft to the user. Do NOT write without confirmation.

### 4. Commit Prompt
After user approves and the file is written:
- Suggest: `git add handover/ai-direct/LATEST.md && git commit -m "docs: update handover state"`
- Wait for user confirmation before committing
