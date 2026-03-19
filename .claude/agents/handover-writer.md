---
name: handover-writer
description: Generates handover documentation by analyzing git history and current state
model: sonnet
tools:
  - Read
  - Bash
  - Grep
  - Glob
---

# Handover Writer Agent

Generate updated `handover/ai-direct/LATEST.md` for session continuity.

## Procedure

1. Read current `handover/ai-direct/LATEST.md` for existing state
2. Run `git log --oneline -20` to see recent commits
3. Run `git diff HEAD~5..HEAD --stat` to see recent file changes
4. Read any modified critical files for context

## Output

Generate a draft LATEST.md update containing:

### Structure
```markdown
# TuringOS v3 — Handover State
**Date**: YYYY-MM-DD
**Session**: [brief description]

## Current State
- What works
- What's broken
- Active experiments

## This Session
- Changes made (with commit refs)
- Key decisions and rationale
- Problems encountered

## Next Steps
- Immediate priorities
- Blocked items
- Open questions

## Critical Notes
- Anything the next session MUST know
```

Present the draft to the user for review before writing. Do NOT write directly — output the draft and wait for confirmation.
