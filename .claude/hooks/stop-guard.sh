#!/usr/bin/env bash
# Stop hook: Remind about uncommitted changes to critical files
# Always exit 0 (advisory only)

set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

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
fi

# Living Harness reminders
if [ -f "rules/enforcement.log" ]; then
    TRIGGERS=$(wc -l < rules/enforcement.log 2>/dev/null || echo 0)
    if [ "$TRIGGERS" -gt 0 ]; then
        echo "📊 Rule engine triggered $TRIGGERS times this session."
    fi
fi
echo "→ /handover-update to save session state"
echo "→ /harness-reflect if new violations were logged (Living Harness)"

exit 0
