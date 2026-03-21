#!/usr/bin/env bash
# Stop hook: Remind about uncommitted changes to critical files
# Always exit 0 (advisory only)

set -euo pipefail

cd /home/zephryj/projects/turingosv3

CORE_FILES="src/kernel.rs src/bus.rs src/sdk/tools/wallet.rs experiments/minif2f_swarm/src/swarm.rs experiments/zeta_regularization/src/swarm.rs experiments/zeta_regularization/src/bin/zeta_evaluator.rs handover/ALIGNMENT.md"

DIRTY_FILES=""
for f in $CORE_FILES; do
    if [ -f "$f" ] && ! git diff --quiet -- "$f" 2>/dev/null; then
        DIRTY_FILES="$DIRTY_FILES $f"
    fi
    if [ -f "$f" ] && ! git diff --cached --quiet -- "$f" 2>/dev/null; then
        DIRTY_FILES="$DIRTY_FILES $f (staged)"
    fi
done

if [ -n "$DIRTY_FILES" ]; then
    echo "⚠ WARNING: Uncommitted changes in critical files:"
    for f in $DIRTY_FILES; do
        echo "  - $f"
    done
    echo ""
    echo "Consider committing before ending session."
    echo "Run /handover-update to save session state."
fi

exit 0
