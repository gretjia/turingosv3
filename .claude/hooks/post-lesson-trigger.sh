#!/usr/bin/env bash
# post-lesson-trigger.sh — Living Harness: auto-suggest /lesson-to-rule
# PostToolUse hook: when VIA_NEGATIVA.md or handover violation logs are modified
# Always exit 0 (advisory)

set -euo pipefail

INPUT=$(cat)
FILE_PATH=$(echo "$INPUT" | jq -r '.tool_input.file_path // empty' 2>/dev/null)

if [ -z "$FILE_PATH" ]; then
    exit 0
fi

case "$FILE_PATH" in
    *VIA_NEGATIVA*|*VIOLATION*|*violation*)
        echo ""
        echo "┌──────────────────────────────────────────────────────────┐"
        echo "│  LIVING HARNESS — NEW VIOLATION DETECTED                │"
        echo "└──────────────────────────────────────────────────────────┘"
        echo "  → 自动运行 /lesson-to-rule 将违规转化为可执行规则"
        echo "  → 同时创建 incidents/ 完整 trace 目录"
        echo ""
        ;;
esac

exit 0
