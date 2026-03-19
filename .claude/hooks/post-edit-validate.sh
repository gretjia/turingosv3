#!/usr/bin/env bash
# PostToolUse hook: Run cargo check after editing critical files
# Exit 0 = pass, Exit 1 = fail

set -euo pipefail

INPUT=$(cat)
FILE_PATH=$(echo "$INPUT" | jq -r '.tool_input.file_path // empty' 2>/dev/null)

if [ -z "$FILE_PATH" ]; then
    exit 0
fi

# Only trigger for critical files
CRITICAL_PATTERN="src/kernel\.rs|src/bus\.rs|src/sdk/tool\.rs|src/sdk/tools/wallet\.rs|Cargo\.toml"

if echo "$FILE_PATH" | grep -qE "$CRITICAL_PATTERN"; then
    echo "Critical file modified: $FILE_PATH"
    echo "Running cargo check..."

    cd /home/zephryj/projects/turingosv3
    if cargo check 2>&1; then
        echo "✓ cargo check PASSED"
        exit 0
    else
        echo "✗ cargo check FAILED — fix compilation errors before continuing"
        exit 1
    fi
fi

exit 0
