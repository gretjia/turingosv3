#!/usr/bin/env bash
# TuringOS v3 Rule Engine — Living Harness
# PreToolUse hook for Edit/Write/Bash: evaluates rules from rules/active/*.yaml
# Exit 0 = allow (or warn), Exit 2 = block
#
# Interface: reads JSON from stdin (Claude hook protocol)
#   Edit: { tool_input: { file_path: "...", new_string: "..." } }
#   Write: { tool_input: { file_path: "...", content: "..." } }
#   Bash: { tool_input: { command: "..." } }
#
# Dependencies: jq, python3, grep -P

set -uo pipefail
# Note: NOT -e because grep returns 1 on no-match (expected in rule checks)

# --- Paths ---
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
RULES_DIR="$PROJECT_ROOT/rules/active"
LOG_FILE="$PROJECT_ROOT/rules/enforcement.log"
US=$'\x1f'  # Unit separator for field delimiting

# --- Read hook input via jq ---
INPUT=$(cat)
TOOL_NAME=$(echo "$INPUT" | jq -r '.tool_name // empty' 2>/dev/null)
FILE_PATH=$(echo "$INPUT" | jq -r '.tool_input.file_path // empty' 2>/dev/null)
COMMAND=$(echo "$INPUT" | jq -r '.tool_input.command // empty' 2>/dev/null)

# Determine the target and trigger type
TARGET=""
TRIGGER=""
if [ -n "$FILE_PATH" ]; then
    TARGET="$FILE_PATH"
    TRIGGER="pre_edit"
elif [ -n "$COMMAND" ]; then
    TARGET="$COMMAND"
    TRIGGER="pre_bash"
else
    exit 0
fi

# --- Exempt paths (relative to project root) ---
if [ -n "$FILE_PATH" ]; then
    REL_PATH="${FILE_PATH#"$PROJECT_ROOT"/}"
    case "$REL_PATH" in
        *.md|incidents/*|rules/*|handover/*|tests/*|audit/*)
            exit 0
            ;;
    esac
fi

# --- Ensure rules directory exists ---
if [ ! -d "$RULES_DIR" ]; then
    exit 0
fi

# --- Python YAML parser (no pyyaml dependency) ---
# Outputs US-delimited (0x1f) fields per line:
#   id<US>name<US>trigger<US>check_type<US>pattern<US>file_glob<US>enforcement<US>message<US>rule_file
# For compound rules, pattern = grep_pattern<TAB>grep_inverse_pattern
parse_rules() {
    export RULES_DIR_ENV="$RULES_DIR"
    python3 << 'PYEOF'
import os, glob

rules_dir = os.environ["RULES_DIR_ENV"]
US = "\x1f"

def parse_yaml_value(line):
    """Extract value after first colon, strip surrounding quotes."""
    idx = line.index(":")
    val = line[idx+1:].strip()
    if (val.startswith('"') and val.endswith('"')) or \
       (val.startswith("'") and val.endswith("'")):
        val = val[1:-1]
    return val

def parse_rule_file(fpath):
    rule = {}
    check = {}
    in_check = False
    in_stats = False

    with open(fpath) as f:
        for line in f:
            stripped = line.rstrip()
            if not stripped or stripped.startswith("#"):
                continue

            indent = len(line) - len(line.lstrip())

            if indent >= 2 and in_check and ":" in stripped:
                key = stripped.strip().split(":")[0].strip()
                val = parse_yaml_value(stripped.strip())
                check[key] = val
                continue
            if indent >= 2 and in_stats:
                continue

            if stripped.startswith("check:"):
                in_check = True
                in_stats = False
                continue
            elif stripped.startswith("stats:"):
                in_stats = True
                in_check = False
                continue
            elif ":" in stripped and indent == 0:
                in_check = False
                in_stats = False
                key = stripped.split(":")[0].strip()
                val = parse_yaml_value(stripped)
                if key in ("id", "name", "trigger", "file_glob", "enforcement", "message"):
                    rule[key] = val

    rule["check"] = check
    rule["_file"] = fpath
    return rule

for fpath in sorted(glob.glob(os.path.join(rules_dir, "*.yaml"))):
    try:
        r = parse_rule_file(fpath)
        check = r.get("check", {})
        check_type = check.get("type", "grep")

        if check_type == "compound":
            pat = check.get("grep_pattern", "") + "\t" + check.get("grep_inverse_pattern", "")
        else:
            pat = check.get("pattern", "")

        # Unescape double backslashes for grep -P compatibility
        pat = pat.replace("\\\\", "\\")

        fields = [
            r.get("id", ""),
            r.get("name", ""),
            r.get("trigger", ""),
            check_type,
            pat,
            r.get("file_glob", ""),
            r.get("enforcement", ""),
            r.get("message", ""),
            r.get("_file", ""),
        ]
        print(US.join(fields))
    except Exception as e:
        import sys
        print(f"WARN: Failed to parse {fpath}: {e}", file=sys.stderr)
PYEOF
}

# --- Glob matching ---
matches_glob() {
    local filepath="$1"
    local globs="$2"
    local rel="${filepath#"$PROJECT_ROOT"/}"
    local bname
    bname="$(basename "$filepath")"

    IFS='|' read -ra GLOB_ARRAY <<< "$globs"
    for g in "${GLOB_ARRAY[@]}"; do
        g="$(echo "$g" | xargs)"
        # shellcheck disable=SC2254
        case "$rel" in
            $g) return 0 ;;
        esac
        # shellcheck disable=SC2254
        case "$bname" in
            $g) return 0 ;;
        esac
    done
    return 1
}

# --- Log enforcement ---
log_enforcement() {
    local rule_id="$1"
    local enforcement="$2"
    local target="$3"
    local message="$4"
    local ts
    ts="$(date -u '+%Y-%m-%dT%H:%M:%SZ')"
    mkdir -p "$(dirname "$LOG_FILE")"
    echo "$ts | $enforcement | $rule_id | $target | $message" >> "$LOG_FILE"
}

# --- Update stats in rule YAML ---
update_stats() {
    local rule_file="$1"
    local ts
    ts="$(date -u '+%Y-%m-%dT%H:%M:%SZ')"

    if grep -q "times_triggered:" "$rule_file"; then
        local current
        current=$(grep "times_triggered:" "$rule_file" | head -1 | grep -oP '\d+')
        local new_count=$((current + 1))
        sed -i "s/times_triggered: $current/times_triggered: $new_count/" "$rule_file"
    fi
    sed -i "s/last_triggered: .*/last_triggered: \"$ts\"/" "$rule_file"
}

# --- Evaluate rules ---
BLOCKED=0
BLOCK_MSG=""
WARN_MSGS=""

while IFS= read -r line; do
    [ -z "$line" ] && continue

    # Parse US-delimited fields
    IFS="$US" read -ra F <<< "$line"
    rule_id="${F[0]:-}"
    rule_name="${F[1]:-}"
    rule_trigger="${F[2]:-}"
    check_type="${F[3]:-}"
    pattern="${F[4]:-}"
    file_glob="${F[5]:-}"
    enforcement="${F[6]:-}"
    message="${F[7]:-}"
    rule_file="${F[8]:-}"

    # Filter by trigger type
    if [ "$rule_trigger" != "$TRIGGER" ]; then
        continue
    fi

    # --- For pre_edit: match file glob ---
    if [ "$TRIGGER" = "pre_edit" ] && [ -n "$FILE_PATH" ]; then
        if ! matches_glob "$FILE_PATH" "$file_glob"; then
            continue
        fi
    fi

    # --- For pre_bash: check command against pattern ---
    if [ "$TRIGGER" = "pre_bash" ] && [ -n "$COMMAND" ]; then
        violated=0
        case "$check_type" in
            grep)
                if echo "$COMMAND" | grep -qP "$pattern" 2>/dev/null; then
                    violated=1
                fi
                ;;
            grep_inverse)
                if ! echo "$COMMAND" | grep -qP "$pattern" 2>/dev/null; then
                    violated=1
                fi
                ;;
            compound)
                grep_pat="${pattern%%	*}"
                inv_pat="${pattern#*	}"
                v1=0; v2=0
                if [ -n "$grep_pat" ] && echo "$COMMAND" | grep -qP "$grep_pat" 2>/dev/null; then
                    v1=1
                fi
                if [ -n "$inv_pat" ] && ! echo "$COMMAND" | grep -qP "$inv_pat" 2>/dev/null; then
                    v2=1
                fi
                if [ $v1 -eq 1 ] || [ $v2 -eq 1 ]; then violated=1; fi
                ;;
        esac

        if [ $violated -eq 1 ]; then
            update_stats "$rule_file"
            log_enforcement "$rule_id" "$enforcement" "$COMMAND" "$message"
            if [ "$enforcement" = "block" ]; then
                BLOCKED=1
                BLOCK_MSG="$message"
            else
                WARN_MSGS="${WARN_MSGS}${message}\n"
            fi
        fi
        continue
    fi

    # --- For pre_edit: check new content against pattern ---
    new_content=$(echo "$INPUT" | jq -r '.tool_input.new_string // .tool_input.content // empty' 2>/dev/null)

    if [ -z "$new_content" ]; then
        # Catch-all warn rules fire on any edit to matched file (e.g. R-006)
        if [ "$enforcement" = "warn" ] && [ "$pattern" = ".*" ]; then
            update_stats "$rule_file"
            log_enforcement "$rule_id" "$enforcement" "$FILE_PATH" "$message"
            WARN_MSGS="${WARN_MSGS}${message}\n"
        fi
        continue
    fi

    violated=0
    case "$check_type" in
        grep)
            if [ "$pattern" = ".*" ]; then
                violated=1
            elif echo "$new_content" | grep -qP "$pattern" 2>/dev/null; then
                violated=1
            fi
            ;;
        grep_inverse)
            if ! echo "$new_content" | grep -qP "$pattern" 2>/dev/null; then
                violated=1
            fi
            ;;
        compound)
            grep_pat="${pattern%%	*}"
            inv_pat="${pattern#*	}"
            v1=0; v2=0
            if [ -n "$grep_pat" ] && echo "$new_content" | grep -qP "$grep_pat" 2>/dev/null; then
                v1=1
            fi
            if [ -n "$inv_pat" ] && ! echo "$new_content" | grep -qP "$inv_pat" 2>/dev/null; then
                v2=1
            fi
            if [ $v1 -eq 1 ] || [ $v2 -eq 1 ]; then violated=1; fi
            ;;
    esac

    if [ $violated -eq 1 ]; then
        update_stats "$rule_file"
        log_enforcement "$rule_id" "$enforcement" "$FILE_PATH" "$message"
        if [ "$enforcement" = "block" ]; then
            BLOCKED=1
            BLOCK_MSG="$message"
            break
        else
            WARN_MSGS="${WARN_MSGS}${message}\n"
        fi
    fi

done < <(parse_rules)

# --- Emit warnings to stderr ---
if [ -n "$WARN_MSGS" ]; then
    echo -e "$WARN_MSGS" >&2
fi

# --- Block or allow ---
if [ $BLOCKED -eq 1 ]; then
    echo "$BLOCK_MSG"
    exit 2
fi

exit 0
