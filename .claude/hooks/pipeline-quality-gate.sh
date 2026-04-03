#!/usr/bin/env bash
# pipeline-quality-gate.sh — Living Harness Pipeline Nervous System
# PostToolUse hook: validates architect insights completeness + triggers violation tracing
# Always exit 0 (advisory only)

set -euo pipefail

INPUT=$(cat)
FILE_PATH=$(echo "$INPUT" | jq -r '.tool_input.file_path // empty' 2>/dev/null)

if [ -z "$FILE_PATH" ]; then
    exit 0
fi

PROJECT_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || echo /home/zephryj/projects/turingosv3)"
cd "$PROJECT_ROOT" 2>/dev/null || exit 0

# ════════════════════════════════════════════════════════════
# Gate 1: Architect Insight completeness check
# ════════════════════════════════════════════════════════════
if echo "$FILE_PATH" | grep -qE "handover/architect-insights/.*\.md$"; then
    CONTENT=$(echo "$INPUT" | jq -r '.tool_input.content // empty' 2>/dev/null)
    NEW_STR=$(echo "$INPUT" | jq -r '.tool_input.new_string // empty' 2>/dev/null)
    INS_CONTENT="${CONTENT}${NEW_STR}"

    if [ -z "$INS_CONTENT" ] && [ -f "$FILE_PATH" ]; then
        INS_CONTENT=$(cat "$FILE_PATH")
    fi

    if [ -n "$INS_CONTENT" ]; then
        MISSING=""
        echo "$INS_CONTENT" | grep -q "## Rationale" || MISSING="${MISSING} [Rationale]"
        echo "$INS_CONTENT" | grep -qi "## Assumption\|## 前提假设\|## Precondition" || MISSING="${MISSING} [Assumptions]"
        echo "$INS_CONTENT" | grep -qi "## Rejected\|## 被拒绝\|## Alternative" || MISSING="${MISSING} [Rejected Alternatives]"
        echo "$INS_CONTENT" | grep -qi "## Verif\|## 验证\|## Test" || MISSING="${MISSING} [Verification Protocol]"

        if [ -n "$MISSING" ]; then
            INSIGHT_NAME=$(basename "$FILE_PATH" .md | head -c 40)
            echo ""
            echo "┌──────────────────────────────────────────────────────────┐"
            echo "│  LIVING HARNESS — INSIGHT QUALITY GATE                  │"
            echo "└──────────────────────────────────────────────────────────┘"
            echo "  $INSIGHT_NAME 缺少:${MISSING}"
            echo "  V-001 教训: fund_agent 被 4 次内审放过, 因为缺少被拒方案记录"
            echo "  → 请补充缺失 section 后继续"
            echo ""
        fi
    fi
fi

# ════════════════════════════════════════════════════════════
# Gate 2: VIA_NEGATIVA modification — trigger violation tracing
# ════════════════════════════════════════════════════════════
if echo "$FILE_PATH" | grep -qE "VIA_NEGATIVA"; then
    if [ -f "incidents/INDEX.yaml" ]; then
        LATEST_V=$(grep -oP 'V-\d+' VIA_NEGATIVA.md 2>/dev/null | sort -t- -k2 -n | tail -1 || echo "")
        if [ -n "$LATEST_V" ]; then
            echo ""
            echo "┌──────────────────────────────────────────────────────────┐"
            echo "│  LIVING HARNESS — VIOLATION TRACE BACK                  │"
            echo "└──────────────────────────────────────────────────────────┘"
            echo "  新违规 $LATEST_V 已记录。"
            echo "  → /lesson-to-rule $LATEST_V (生成执法规则)"
            echo "  → 创建 incidents/$LATEST_V trace 目录"
            echo ""
        fi
    fi
fi

exit 0
