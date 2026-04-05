#!/bin/sh
# ══════════════════════════════════════════════════════════════
# AutoResearch Swarm Health Monitor
# POSIX sh compatible (macOS Bash 3.2 safe)
# Run via cron: */10 * * * * /path/to/monitor_swarm.sh
# ══════════════════════════════════════════════════════════════
set -u
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT="$(cd "$SCRIPT_DIR/.." && pwd)"
LOG="/tmp/autoresearch_swarm/monitor.log"
mkdir -p /tmp/autoresearch_swarm

ts() { date '+%Y-%m-%d %H:%M:%S'; }

echo "$(ts) === Monitor check ===" >> "$LOG"

# ── Load .env ──
if [ -f "$PROJECT/.env" ]; then
    while IFS='=' read -r key val; do
        case "$key" in \#*|"") continue ;; esac
        export "$key=$val"
    done < "$PROJECT/.env"
fi

PROXY_SCRIPT="$PROJECT/src/drivers/llm_proxy.py"
PID_DIR="$SCRIPT_DIR/shared/.pids"
mkdir -p "$PID_DIR"

# ── Config (POSIX arrays) ──
RESEARCHERS="
alpha:8088:dashscope:DASHSCOPE_API_KEY:zeta
beta:8089:siliconflow:SILICONFLOW_API_KEY:zeta-b
gamma:8090:siliconflow:SILICONFLOW_API_KEY_SECONDARY:zeta-c
delta:8091:volcengine:VOLCENGINE_API_KEY:zeta-d
"

get_field() { echo "$1" | cut -d: -f"$2"; }

get_api_key_value() { eval echo "\$$1"; }

is_pid_alive() {
    [ -f "$1" ] && kill -0 "$(cat "$1")" 2>/dev/null
}

# ── Check & restart proxies ──
for entry in $RESEARCHERS; do
    [ -z "$entry" ] && continue
    id=$(get_field "$entry" 1)
    port=$(get_field "$entry" 2)
    provider=$(get_field "$entry" 3)
    key_env=$(get_field "$entry" 4)

    if ! curl -s "http://127.0.0.1:$port/health" > /dev/null 2>&1; then
        echo "$(ts) Proxy $id (port $port) DOWN — restarting" >> "$LOG"

        # Kill stale process
        pidfile="$PID_DIR/proxy_${id}.pid"
        if [ -f "$pidfile" ]; then
            kill "$(cat "$pidfile")" 2>/dev/null || true
            rm -f "$pidfile"
        fi
        sleep 1

        key_val=$(get_api_key_value "$key_env")
        case "$key_env" in
            SILICONFLOW_API_KEY_SECONDARY) export_env="SILICONFLOW_API_KEY=$key_val" ;;
            *) export_env="${key_env}=$key_val" ;;
        esac

        env $export_env nohup python3 "$PROXY_SCRIPT" --port "$port" --provider "$provider" \
            > "/tmp/autoresearch_swarm/proxy_${id}.log" 2>&1 &
        echo "$!" > "$PID_DIR/proxy_${id}.pid"
        echo "$(ts) Proxy $id restarted (PID $!)" >> "$LOG"
    fi
done

# ── Check & restart researchers ──
for entry in $RESEARCHERS; do
    [ -z "$entry" ] && continue
    id=$(get_field "$entry" 1)
    dir=$(get_field "$entry" 5)

    pidfile="$PID_DIR/researcher_${id}.pid"
    if ! is_pid_alive "$pidfile"; then
        echo "$(ts) Researcher $id ($dir) DOWN — restarting" >> "$LOG"
        rm -f "$pidfile"

        abs_sweep="$SCRIPT_DIR/$dir/sweep.py"
        nohup python3 "$abs_sweep" > "/tmp/autoresearch_swarm/researcher_${id}.log" 2>&1 &
        echo "$!" > "$pidfile"
        echo "$(ts) Researcher $id restarted (PID $!)" >> "$LOG"
    fi
done

# ── Clean old evaluator logs (>7 days) ──
for dir in zeta zeta-b zeta-c; do
    target="$SCRIPT_DIR/$dir/logs"
    if [ -d "$target" ]; then
        old_count=$(find "$target" -name "eval_*.log" -mtime +7 2>/dev/null | wc -l)
        if [ "$old_count" -gt 0 ]; then
            find "$target" -name "eval_*.log" -mtime +7 -delete 2>/dev/null
            echo "$(ts) Cleaned $old_count old eval logs from $dir/logs/" >> "$LOG"
        fi
        # Also clean old run_* directories
        old_runs=$(find "$target" -maxdepth 1 -name "run_*" -type d -mtime +7 2>/dev/null | wc -l)
        if [ "$old_runs" -gt 0 ]; then
            find "$target" -maxdepth 1 -name "run_*" -type d -mtime +7 -exec rm -rf {} + 2>/dev/null
            echo "$(ts) Cleaned $old_runs old run dirs from $dir/logs/" >> "$LOG"
        fi
    fi
done

# ── Clean old tapes (keep latest 50 per researcher) ──
for dir in zeta zeta-b zeta-c; do
    target="$SCRIPT_DIR/$dir/tapes"
    if [ -d "$target" ]; then
        tape_count=$(find "$target" -name "tape_*.md" 2>/dev/null | wc -l)
        if [ "$tape_count" -gt 50 ]; then
            to_delete=$((tape_count - 50))
            find "$target" -name "tape_*.md" -print0 2>/dev/null | xargs -0 ls -t | tail -n "$to_delete" | xargs rm -f
            echo "$(ts) Cleaned $to_delete old tapes from $dir/tapes/" >> "$LOG"
        fi
        # Clean old WAL files too
        find "$target" -name "wal_*.json" -mtime +7 -delete 2>/dev/null
    fi
done

# ── Archive bulletin if too large (>500 entries) — with lock ──
bul_file="$SCRIPT_DIR/shared/bulletin.jsonl"
if [ -f "$bul_file" ]; then
    bul_lines=$(wc -l < "$bul_file")
    if [ "$bul_lines" -gt 500 ]; then
        # Use Python for atomic locked archive (fcntl)
        python3 -c "
import fcntl, os, sys
bul = '$bul_file'
fd = os.open(bul, os.O_RDWR)
try:
    fcntl.flock(fd, fcntl.LOCK_EX)
    with open(bul) as f:
        lines = f.readlines()
    if len(lines) > 500:
        archive = bul.replace('.jsonl', '_archive_') + '$(date +%Y%m%d_%H%M%S).jsonl'
        with open(archive, 'w') as af:
            af.writelines(lines[:-200])
        with open(bul, 'w') as bf:
            bf.writelines(lines[-200:])
        print(f'Archived {len(lines)-200} entries')
finally:
    fcntl.flock(fd, fcntl.LOCK_UN)
    os.close(fd)
" >> "$LOG" 2>&1
    fi
fi

# ── Memory pressure check (macOS-aware with correct pagesize) ──
if command -v vm_stat >/dev/null 2>&1; then
    pagesize=$(sysctl -n hw.pagesize 2>/dev/null || echo 16384)
    free_pages=$(vm_stat | grep "Pages free" | awk '{print $3}' | tr -d '.')
    free_mb=$((free_pages * pagesize / 1024 / 1024))
    if [ "$free_mb" -lt 2048 ]; then
        echo "$(ts) WARNING: Low memory (${free_mb}MB free, pagesize=${pagesize})" >> "$LOG"
        # Under severe pressure, pause gamma (lowest priority)
        if [ "$free_mb" -lt 1024 ]; then
            echo "$(ts) CRITICAL: Pausing gamma researcher (${free_mb}MB free)" >> "$LOG"
            pidfile="$PID_DIR/researcher_gamma.pid"
            if is_pid_alive "$pidfile"; then
                kill -STOP "$(cat "$pidfile")" 2>/dev/null || true
            fi
        fi
    else
        # Resume gamma if paused
        pidfile="$PID_DIR/researcher_gamma.pid"
        if is_pid_alive "$pidfile"; then
            kill -CONT "$(cat "$pidfile")" 2>/dev/null || true
        fi
    fi
fi

# ── Clean stale semaphore locks ──
for lock in "$SCRIPT_DIR"/shared/.eval_slot_*.lock; do
    [ -f "$lock" ] || continue
    lock_pid=$(awk '{print $2}' "$lock" 2>/dev/null)
    if [ -n "$lock_pid" ] && ! kill -0 "$lock_pid" 2>/dev/null; then
        rm -f "$lock"
        echo "$(ts) Cleaned stale semaphore lock: $lock (PID $lock_pid dead)" >> "$LOG"
    fi
done

# ── Rotate monitor log (keep last 1000 lines) ──
if [ -f "$LOG" ]; then
    log_lines=$(wc -l < "$LOG")
    if [ "$log_lines" -gt 1000 ]; then
        tail -500 "$LOG" > "${LOG}.tmp" && mv "${LOG}.tmp" "$LOG"
    fi
fi

echo "$(ts) Monitor check complete" >> "$LOG"
