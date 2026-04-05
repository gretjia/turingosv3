#!/bin/sh
# ══════════════════════════════════════════════════════════════
# TuringOS AutoResearch Swarm Launcher
# POSIX sh compatible (macOS Bash 3.2 safe — no declare -A)
#
# Usage:
#   ./launch_swarm.sh          # launch all
#   ./launch_swarm.sh stop     # stop all
#   ./launch_swarm.sh status   # show status
# ══════════════════════════════════════════════════════════════
set -eu
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT="$(cd "$SCRIPT_DIR/.." && pwd)"

# ── Load .env ──
if [ -f "$PROJECT/.env" ]; then
    while IFS='=' read -r key val; do
        case "$key" in \#*|"") continue ;; esac
        export "$key=$val"
    done < "$PROJECT/.env"
fi

PROXY_SCRIPT="$PROJECT/src/drivers/llm_proxy.py"
LOG_DIR="/tmp/autoresearch_swarm"
PID_DIR="$SCRIPT_DIR/shared/.pids"
mkdir -p "$LOG_DIR" "$PID_DIR"

# ── Researcher configs (POSIX-compatible, no associative arrays) ──
# Format: id:port:provider_flag:env_override:dir
RESEARCHERS="
alpha:8088:dashscope:DASHSCOPE_API_KEY:zeta
beta:8089:siliconflow:SILICONFLOW_API_KEY:zeta-b
gamma:8090:siliconflow:SILICONFLOW_API_KEY_SECONDARY:zeta-c
"

get_field() { echo "$1" | cut -d: -f"$2"; }

get_api_key_value() {
    # Resolve env var name to its value
    eval echo "\$$1"
}

is_proxy_alive() {
    curl -s "http://127.0.0.1:$1/health" > /dev/null 2>&1
}

is_pid_alive() {
    [ -f "$1" ] && kill -0 "$(cat "$1")" 2>/dev/null
}

# ══════════════════════════════════════════════════════════════
# Stop
# ══════════════════════════════════════════════════════════════
stop_all() {
    echo "=== Stopping AutoResearch Swarm ==="
    for entry in $RESEARCHERS; do
        [ -z "$entry" ] && continue
        id=$(get_field "$entry" 1)
        port=$(get_field "$entry" 2)

        # Stop researcher via PID file
        pidfile="$PID_DIR/researcher_${id}.pid"
        if is_pid_alive "$pidfile"; then
            kill "$(cat "$pidfile")" 2>/dev/null && echo "  Stopped researcher $id" || true
        fi
        rm -f "$pidfile"

        # Stop proxy via PID file
        pidfile="$PID_DIR/proxy_${id}.pid"
        if is_pid_alive "$pidfile"; then
            kill "$(cat "$pidfile")" 2>/dev/null && echo "  Stopped proxy $id (port $port)" || true
        fi
        rm -f "$pidfile"
    done
    echo "=== All stopped ==="
}

# ══════════════════════════════════════════════════════════════
# Status
# ══════════════════════════════════════════════════════════════
show_status() {
    echo "=== AutoResearch Swarm Status ==="
    for entry in $RESEARCHERS; do
        [ -z "$entry" ] && continue
        id=$(get_field "$entry" 1)
        port=$(get_field "$entry" 2)
        dir=$(get_field "$entry" 5)

        # Proxy status
        if is_proxy_alive "$port"; then
            proxy_status="UP"
        else
            proxy_status="DOWN"
        fi

        # Researcher status
        pidfile="$PID_DIR/researcher_${id}.pid"
        if is_pid_alive "$pidfile"; then
            rpid=$(cat "$pidfile")
            rss_kb=$(ps -o rss= -p "$rpid" 2>/dev/null || echo 0)
            rss_mb=$((rss_kb / 1024))
            res_status="UP (PID $rpid, ${rss_mb}MB)"
        else
            res_status="DOWN"
        fi

        # Experiment count
        results_file="$SCRIPT_DIR/$dir/results.tsv"
        if [ -f "$results_file" ]; then
            exp_count=$(($(wc -l < "$results_file") - 1))
        else
            exp_count=0
        fi

        echo "  [$id] proxy=$proxy_status | researcher=$res_status | experiments=$exp_count"
    done

    # Bulletin stats
    bul_file="$SCRIPT_DIR/shared/bulletin.jsonl"
    if [ -f "$bul_file" ]; then
        bul_lines=$(wc -l < "$bul_file")
        bul_size=$(du -h "$bul_file" | cut -f1)
        echo "  Bulletin: $bul_lines entries ($bul_size)"
    fi

    # Memory (macOS-aware: use pagesize from sysctl)
    if command -v vm_stat >/dev/null 2>&1; then
        pagesize=$(sysctl -n hw.pagesize 2>/dev/null || echo 4096)
        free_pages=$(vm_stat | grep "Pages free" | awk '{print $3}' | tr -d '.')
        free_mb=$((free_pages * pagesize / 1024 / 1024))
        echo "  Memory free: ~${free_mb}MB (pagesize=${pagesize})"
    fi
    echo "=== End Status ==="
}

# ══════════════════════════════════════════════════════════════
# Launch
# ══════════════════════════════════════════════════════════════
launch_all() {
    echo "=== Launching AutoResearch Swarm ==="
    echo "  Project: $PROJECT"
    echo ""

    # ── Launch proxies ──
    for entry in $RESEARCHERS; do
        [ -z "$entry" ] && continue
        id=$(get_field "$entry" 1)
        port=$(get_field "$entry" 2)
        provider=$(get_field "$entry" 3)
        key_env=$(get_field "$entry" 4)

        # Skip if already running
        if is_proxy_alive "$port"; then
            echo "  Proxy $id (port $port): already running"
            continue
        fi

        key_val=$(get_api_key_value "$key_env")
        # For secondary SiliconFlow key, env var name is SILICONFLOW_API_KEY (what proxy reads)
        case "$key_env" in
            SILICONFLOW_API_KEY_SECONDARY)
                export_env="SILICONFLOW_API_KEY=$key_val"
                ;;
            *)
                export_env="${key_env}=$key_val"
                ;;
        esac

        env $export_env nohup python3 "$PROXY_SCRIPT" --port "$port" --provider "$provider" \
            > "$LOG_DIR/proxy_${id}.log" 2>&1 &
        echo "$!" > "$PID_DIR/proxy_${id}.pid"
        echo "  Proxy $id (port $port, provider=$provider): launched (PID $!)"
    done

    # Wait for proxies
    echo "  Waiting for proxies..."
    sleep 3

    for entry in $RESEARCHERS; do
        [ -z "$entry" ] && continue
        id=$(get_field "$entry" 1)
        port=$(get_field "$entry" 2)
        if is_proxy_alive "$port"; then
            echo "  Proxy $id: OK"
        else
            echo "  WARNING: Proxy $id (port $port) not responding!"
        fi
    done

    echo ""

    # ── Launch researchers (using absolute paths for PID detection) ──
    for entry in $RESEARCHERS; do
        [ -z "$entry" ] && continue
        id=$(get_field "$entry" 1)
        dir=$(get_field "$entry" 5)

        pidfile="$PID_DIR/researcher_${id}.pid"

        # Skip if already running
        if is_pid_alive "$pidfile"; then
            echo "  Researcher $id ($dir/): already running"
            continue
        fi

        abs_sweep="$SCRIPT_DIR/$dir/sweep.py"
        nohup python3 "$abs_sweep" > "$LOG_DIR/researcher_${id}.log" 2>&1 &
        echo "$!" > "$pidfile"
        echo "  Researcher $id ($dir/): launched (PID $!)"
    done

    echo ""
    echo "=== Swarm Active ==="
    echo "  Logs:     $LOG_DIR/"
    echo "  Bulletin: $SCRIPT_DIR/shared/bulletin.jsonl"
    echo "  Monitor:  ./launch_swarm.sh status"
    echo "  Stop:     ./launch_swarm.sh stop"
}

# ══════════════════════════════════════════════════════════════
# Entry
# ══════════════════════════════════════════════════════════════
case "${1:-launch}" in
    stop)    stop_all ;;
    status)  show_status ;;
    launch)  launch_all ;;
    *)       echo "Usage: $0 [launch|stop|status]" ;;
esac
