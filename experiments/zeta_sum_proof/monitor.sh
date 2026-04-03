#!/bin/bash
# AutoResearch v4 Monitor — runs via cron on omega-vm
# Checks health, restarts dead processes, logs status
# Install: crontab -e → */30 * * * * /home/zephryj/projects/turingosv3/experiments/zeta_sum_proof/monitor.sh

set -e
PROJECT="/home/zephryj/projects/turingosv3"
LOG="/tmp/autoresearch_monitor.log"
V4_LOG="/tmp/autoresearch_v4.log"
TSV="$PROJECT/experiments/zeta_sum_proof/audit/autoresearch_v4.tsv"

echo "=== $(date -u +%Y-%m-%dT%H:%M:%SZ) Monitor ===" >> "$LOG"

# 1. Check sweep_v4 process
SWEEP_PID=$(pgrep -f 'sweep_v4.py' || true)
if [ -z "$SWEEP_PID" ]; then
    echo "  [RESTART] sweep_v4.py was dead, restarting..." >> "$LOG"
    cd "$PROJECT"
    source "$PROJECT/.env" 2>/dev/null
    python3 experiments/zeta_sum_proof/sweep_v4.py >> "$V4_LOG" 2>&1 &
    echo "  [RESTART] New PID: $!" >> "$LOG"
else
    echo "  [OK] sweep_v4.py alive (PID $SWEEP_PID)" >> "$LOG"
fi

# 2. Check Mac endpoint
MAC_HEALTH=$(timeout 5 curl -s http://127.0.0.1:18080/health 2>/dev/null || echo "DEAD")
if echo "$MAC_HEALTH" | grep -q "ok"; then
    echo "  [OK] Mac endpoint healthy" >> "$LOG"
else
    echo "  [RESTART] Mac endpoint dead, restarting..." >> "$LOG"
    ssh zephrymac-studio "pkill -f llama-server; sleep 2; nohup /opt/homebrew/bin/llama-server -m /Users/zephryj/work/models/Qwen3.5-9B-Q4_K_M.gguf --host 0.0.0.0 --port 8080 -ngl 99 -c 8192 --parallel 2 --threads 8 > /tmp/llama_server.log 2>&1 &" 2>/dev/null
    # Rebuild tunnel if needed
    pkill -f 'ssh.*18080.*zephrymac' 2>/dev/null || true
    sleep 5
    ssh -f -N -L 18080:127.0.0.1:8080 zephrymac-studio 2>/dev/null || true
    echo "  [RESTART] Mac restarted" >> "$LOG"
fi

# 3. Check Win1 endpoint — NSSM Windows service (persistent, survives SSH disconnect)
# Root cause: SSH Session 0 has no Vulkan GPU. Fix: NSSM SERVICE_INTERACTIVE_PROCESS.
# Service registered: nssm install llama-server + nssm set Type SERVICE_INTERACTIVE_PROCESS
WIN1_HEALTH=$(timeout 5 curl -s http://127.0.0.1:18081/health 2>/dev/null || echo "DEAD")
if echo "$WIN1_HEALTH" | grep -q "ok"; then
    echo "  [OK] Win1 endpoint healthy (NSSM service)" >> "$LOG"
else
    echo "  [RESTART] Win1 tunnel dead, rebuilding..." >> "$LOG"
    # Service should be auto-running. Just rebuild SSH tunnel.
    fuser -k 18081/tcp 2>/dev/null || true
    sleep 1
    # Restart NSSM service if needed
    ssh windows1-w1 "nssm status llama-server 2>nul" 2>/dev/null | grep -q "RUNNING" || \
        ssh windows1-w1 "nssm restart llama-server 2>nul" 2>/dev/null
    sleep 10
    ssh -f -N -L 18081:127.0.0.1:8081 windows1-w1 2>/dev/null || true
    sleep 2
    WIN1_CHECK=$(timeout 5 curl -s http://127.0.0.1:18081/health 2>/dev/null || echo "DEAD")
    if echo "$WIN1_CHECK" | grep -q "ok"; then
        echo "  [RESTART] Win1 tunnel recovered (NSSM service running)" >> "$LOG"
    else
        echo "  [FAIL] Win1 recovery failed" >> "$LOG"
    fi
fi

# 4. ERS trend check
if [ -f "$TSV" ]; then
    TOTAL=$(tail -n +2 "$TSV" | wc -l)
    ZERO_ERS=$(tail -n +2 "$TSV" | awk -F'\t' '$3 == "0.0" || $3 == "0.00000"' | wc -l)
    LAST_ERS=$(tail -1 "$TSV" | awk -F'\t' '{print $3}')
    BEST_DEPTH=$(tail -n +2 "$TSV" | awk -F'\t' '{print $4}' | sort -rn | head -1)
    echo "  [STATS] experiments=$TOTAL zero_ers=$ZERO_ERS last_ers=$LAST_ERS best_depth=$BEST_DEPTH" >> "$LOG"

    # Alert: ERS stuck at 0 for 5+ experiments
    LAST_5_ZERO=$(tail -5 "$TSV" | awk -F'\t' '$3 == "0.0" || $3 == "0.00000"' | wc -l)
    if [ "$LAST_5_ZERO" -ge 5 ] && [ "$TOTAL" -ge 5 ]; then
        echo "  [ALERT] ERS stuck at 0.0 for last 5 experiments! Check depth computation." >> "$LOG"
    fi
fi

# 5. Last activity
if [ -f "$V4_LOG" ]; then
    LAST_LINE=$(tail -1 "$V4_LOG")
    echo "  [LOG] $LAST_LINE" >> "$LOG"
fi

echo "" >> "$LOG"
