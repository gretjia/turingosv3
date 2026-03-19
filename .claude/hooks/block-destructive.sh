#!/usr/bin/env bash
# PreToolUse hook: Block destructive Bash commands
# Exit 0 = allow, Exit 2 = block

set -euo pipefail

# Read tool input from stdin
INPUT=$(cat)
COMMAND=$(echo "$INPUT" | jq -r '.tool_input.command // empty' 2>/dev/null)

if [ -z "$COMMAND" ]; then
    exit 0
fi

# Block rm -rf on dangerous paths
if echo "$COMMAND" | grep -qE 'rm\s+(-[a-zA-Z]*r[a-zA-Z]*f|--recursive\s+--force|-[a-zA-Z]*f[a-zA-Z]*r)\s'; then
    if echo "$COMMAND" | grep -qE '(^|\s)(\/|~\/|\.\.\/|\.claude)'; then
        echo "BLOCKED: rm -rf on dangerous path detected: $COMMAND"
        exit 2
    fi
fi

# Block git push --force and git reset --hard
if echo "$COMMAND" | grep -qE 'git\s+push\s+.*--force|git\s+push\s+-f'; then
    echo "BLOCKED: git push --force is prohibited. Use regular git push."
    exit 2
fi

if echo "$COMMAND" | grep -qE 'git\s+reset\s+--hard'; then
    echo "BLOCKED: git reset --hard is prohibited. Use git stash or git checkout instead."
    exit 2
fi

# Block sed/awk modifications to kernel.rs core constants
if echo "$COMMAND" | grep -qE '(sed|awk).*kernel\.rs'; then
    if echo "$COMMAND" | grep -qE '(intrinsic_reward|hayekian_map_reduce|market_price)'; then
        echo "BLOCKED: Direct sed/awk modification of kernel.rs core constants is prohibited."
        exit 2
    fi
fi

# Block deletion of WAL files
if echo "$COMMAND" | grep -qE 'rm\s.*\.(wal|wal\.bak)'; then
    echo "BLOCKED: Deletion of WAL files requires manual confirmation."
    exit 2
fi

# Block deletion of experiment data directories
if echo "$COMMAND" | grep -qE 'rm\s.*experiments/.*/data'; then
    echo "BLOCKED: Deletion of experiment data directories requires manual confirmation."
    exit 2
fi

exit 0
