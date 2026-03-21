#!/usr/bin/env bash
# TuringOS v3 — Experiment Boot Script
#
# Automates steps 3-10 of experiment creation. Only step 2 (Lean 4
# formalization) requires LLM intelligence — everything else is template.
#
# Usage:
#   ./scripts/boot-experiment.sh <project_name> <theorem_name> <lean_problem_file>
#
# Example:
#   echo 'import Mathlib
#   theorem my_thm : 1 + 1 = 2 := by' > /tmp/problem.lean
#   ./scripts/boot-experiment.sh my_experiment my_thm /tmp/problem.lean
#
# Prerequisites:
#   - SILICONFLOW_API_KEY, SILICONFLOW_API_KEY_SECONDARY, DEEPSEEK_API_KEY in env
#   - Lean 4 + Mathlib installed on Mac (for compilation)
#   - SSH access to zephrymac-studio
#
# What this script does:
#   1. Creates experiment directory structure
#   2. Copies shared modules (swarm, harness, wal, membrane)
#   3. Generates Cargo.toml
#   4. Generates evaluator.rs from template
#   5. Registers in workspace Cargo.toml
#   6. Runs cargo check
#   7. Syncs to Mac
#   8. Launches tmux session
#
# What this script does NOT do:
#   - Formalize the theorem in Lean 4 (that's the LLM's job)
#   - Choose model configuration (uses default 3-species)
#   - Decide swarm parameters (uses default N=15, steps=100)

set -euo pipefail

TURINGOS_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
MAC_HOST="zephrymac-studio"
MAC_ROOT="/Users/zephryj/projects/turingosv3"

# --- Argument parsing ---
if [ $# -lt 3 ]; then
    echo "Usage: $0 <project_name> <theorem_name> <lean_problem_file>"
    echo ""
    echo "  project_name:     Directory name under experiments/ (e.g., number_theory_min)"
    echo "  theorem_name:     Lean 4 theorem name (e.g., find_smallest)"
    echo "  lean_problem_file: Path to file containing Lean 4 theorem statement"
    exit 1
fi

PROJECT_NAME="$1"
THEOREM_NAME="$2"
LEAN_PROBLEM_FILE="$3"

# P0 Security: validate project_name as valid Rust crate identifier
if [[ ! "$PROJECT_NAME" =~ ^[a-z][a-z0-9_]*$ ]]; then
    echo "ERROR: project_name must be a valid Rust crate name [a-z][a-z0-9_]*"
    echo "  Got: '$PROJECT_NAME'"
    exit 1
fi

if [ ! -f "$LEAN_PROBLEM_FILE" ]; then
    echo "ERROR: Lean problem file not found: $LEAN_PROBLEM_FILE"
    exit 1
fi

# P2: validate Lean problem doesn't contain raw string terminator
if grep -q '"#' "$LEAN_PROBLEM_FILE"; then
    echo "ERROR: Lean problem file contains '\"#' which breaks Rust raw string r#\"...\"#"
    echo "  Remove or escape this sequence."
    exit 1
fi

LEAN_PROBLEM=$(cat "$LEAN_PROBLEM_FILE")
PROJECT_DIR="$TURINGOS_ROOT/experiments/$PROJECT_NAME"

echo "=== TuringOS v3 Experiment Boot ==="
echo "Project:  $PROJECT_NAME"
echo "Theorem:  $THEOREM_NAME"
echo "Problem:  $(echo "$LEAN_PROBLEM" | head -3)..."
echo ""

# P1: Overwrite protection
if [ -d "$PROJECT_DIR" ]; then
    if [ "${FORCE:-}" != "1" ]; then
        echo "ERROR: $PROJECT_DIR already exists. Set FORCE=1 to overwrite."
        exit 1
    fi
    echo "WARNING: Overwriting existing project $PROJECT_DIR"
fi

# --- Step 3: Create directory structure ---
echo "[3/10] Creating directory structure..."
mkdir -p "$PROJECT_DIR/src/bin"

# --- Step 4: Copy shared modules ---
echo "[4/10] Copying shared modules..."
TEMPLATE_DIR="$TURINGOS_ROOT/experiments/zeta_regularization/src"
for module in swarm.rs harness.rs wal.rs lean4_membrane_tool.rs; do
    cp "$TEMPLATE_DIR/$module" "$PROJECT_DIR/src/$module"
done

cat > "$PROJECT_DIR/src/lib.rs" << 'LIBEOF'
pub mod lean4_membrane_tool;
pub mod swarm;
pub mod harness;
pub mod wal;
LIBEOF

cat > "$PROJECT_DIR/src/main.rs" << MAINEOF
fn main() {
    println!("$PROJECT_NAME experiment initialized.");
}
MAINEOF

# --- Step 5: Generate Cargo.toml ---
echo "[5/10] Generating Cargo.toml..."
cat > "$PROJECT_DIR/Cargo.toml" << CARGOEOF
[package]
name = "$PROJECT_NAME"
version = "0.1.0"
edition = "2021"

[dependencies]
turingosv3 = { path = "../../" }
tokio = { version = "1.28", features = ["full"] }
reqwest = { version = "0.11", features = ["json", "rustls-tls"], default-features = false }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
env_logger = "0.10"
log = "0.4"
rand = "0.8"
CARGOEOF

# --- Step 6: Generate evaluator.rs ---
echo "[6/10] Generating evaluator.rs..."

# Escape the Lean problem for Rust raw string literal
# We write it directly since r#"..."# handles most content
cat > "$PROJECT_DIR/src/bin/evaluator.rs" << 'EVALEOF'
use log::{info, warn};
use std::sync::Arc;
use turingosv3::kernel::{AIBlackBox, File, Head, Input, Kernel, MachineState, SensorContext};
use turingosv3::drivers::llm_http::ResilientLLMClient;
use turingosv3::sdk::tools::wallet::WalletTool;
use turingosv3::sdk::tool::{AntiZombiePruningTool, OverwhelmingGapArbitratorTool};
use turingosv3::bus::{TuringBus, ThermodynamicHeartbeatTool};
use turingosv3::sdk::sandbox::LocalProcessSandbox;
EVALEOF

# Inject crate-specific use statements
cat >> "$PROJECT_DIR/src/bin/evaluator.rs" << EVALEOF2
use ${PROJECT_NAME}::lean4_membrane_tool::Lean4MembraneTool;
use ${PROJECT_NAME}::swarm::SpeculativeSwarmAgent;

const SWARM_SIZE: usize = 15;
const MAX_KERNEL_STEPS: u64 = 100;
const THEOREM_NAME: &str = "${THEOREM_NAME}";

const LEAN_PROBLEM: &str = r#"${LEAN_PROBLEM}"#;
EVALEOF2

# Append the main function template (identical for all experiments)
cat >> "$PROJECT_DIR/src/bin/evaluator.rs" << 'EVALEOF3'

fn main() {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    info!("=== TuringOS v3 Experiment: {} ===", THEOREM_NAME);
    info!("Swarm N={}, Max Steps={}", SWARM_SIZE, MAX_KERNEL_STEPS);

    let sf_url = "https://api.siliconflow.cn/v1/chat/completions";
    let ds_url = "https://api.deepseek.com/chat/completions";

    let key_sf_primary = std::env::var("SILICONFLOW_API_KEY").expect("SILICONFLOW_API_KEY required");
    let key_sf_secondary = std::env::var("SILICONFLOW_API_KEY_SECONDARY").unwrap_or_else(|_| key_sf_primary.clone());
    let key_deepseek = std::env::var("DEEPSEEK_API_KEY").unwrap_or_else(|_| key_sf_primary.clone());

    let client_miner = Arc::new(ResilientLLMClient::with_key(sf_url, "deepseek-ai/DeepSeek-R1-Distill-Qwen-32B", &key_sf_primary));
    let client_scholar = Arc::new(ResilientLLMClient::with_key(ds_url, "deepseek-reasoner", &key_deepseek));
    let client_explorer = Arc::new(ResilientLLMClient::with_key(sf_url, "Pro/deepseek-ai/DeepSeek-R1", &key_sf_secondary));

    info!("Miner: R1-Distill-Qwen-32B | Scholar: deepseek-reasoner | Explorer: DeepSeek-R1");

    let wal_path = format!("/tmp/{}_N{}.wal", THEOREM_NAME, SWARM_SIZE);
    let rt = tokio::runtime::Runtime::new().unwrap();
    let _guard = rt.enter();
    let sentinel = CRATE_NAME::wal::WalSentinel::new(wal_path.clone());
    let recovered_files = rt.block_on(CRATE_NAME::wal::recover_tape(&wal_path));
    info!("WAL recovered {} files from {}", recovered_files.len(), wal_path);

    let mut agent = SpeculativeSwarmAgent::new_multi(
        vec![client_miner, client_scholar, client_explorer],
        MAX_KERNEL_STEPS, SWARM_SIZE, sentinel, recovered_files, LEAN_PROBLEM.to_string(),
    );

    let kernel = Kernel::new();
    let mut bus = TuringBus::new(kernel);

    let sandbox = Box::new(LocalProcessSandbox::new("sh", vec![
        "-c".to_string(),
        "cd /Users/zephryj/projects/turingosv3/experiments/minif2f_data_lean4 && source ~/.elan/env && lake env lean /dev/stdin".to_string(),
    ]));

    bus.mount_tool(Box::new(ThermodynamicHeartbeatTool::new(1)));
    bus.mount_tool(Box::new(AntiZombiePruningTool::new(3)));
    bus.mount_tool(Box::new(OverwhelmingGapArbitratorTool::new(1.5)));
    bus.mount_tool(Box::new(WalletTool::new()));
    bus.mount_tool(Box::new(Lean4MembraneTool::new(LEAN_PROBLEM.to_string(), THEOREM_NAME.to_string(), sandbox)));

    let agent_ids: Vec<String> = (0..100).map(|i| format!("Agent_{}", i)).collect();
    bus.init_problem(&agent_ids);

    info!(">>> TuringOS v3 Booted. N={}, Max Steps={}. <<<", SWARM_SIZE, MAX_KERNEL_STEPS);

    let mut q_state = MachineState::Running;
    let mut current_head = Head { paths: std::collections::HashSet::new() };
    let mut kernel_steps: u64 = 0;

    loop {
        if q_state == MachineState::Halt || kernel_steps >= MAX_KERNEL_STEPS {
            info!("==== EVALUATION COMPLETE (steps={}) ====", kernel_steps);
            bus.kernel.hayekian_map_reduce();
            let mut proved = false;
            for (_, file) in &bus.kernel.tape.files {
                if file.payload.contains("[OMEGA]") { proved = true; break; }
            }
            info!("--- TAPE AUDIT DUMP ---");
            for (id, file) in &bus.kernel.tape.files {
                info!("ID: {} | Parent: {:?} | Price: {:.2} | Reward: {:.2} | Payload: {}",
                    id, file.citations, file.price, file.intrinsic_reward,
                    file.payload.chars().take(200).collect::<String>().replace('\n', " "));
            }
            info!("-----------------------");
            if proved { info!("OMEGA: Theorem PROVED!"); } else { info!("NOT proved within {} steps.", kernel_steps); }
            break;
        }

        kernel_steps += 1;
        let mut balances = std::collections::HashMap::new();
        for i in 0..100 { let aid = format!("Agent_{}", i); balances.insert(aid.clone(), bus.get_agent_balance(&aid)); }
        let mut tombstones = std::collections::HashMap::new();
        for id in bus.kernel.tape.files.keys() { let g = bus.get_tombstones(id); if !g.is_empty() { tombstones.insert(id.clone(), g); } }
        let rg = bus.get_tombstones("root"); if !rg.is_empty() { tombstones.insert("root".to_string(), rg); }

        let input = Input {
            q_i: q_state.clone(),
            s_i: SensorContext { visible_tape: bus.kernel.tape.clone(), current_head: current_head.clone(),
                agent_balances: balances, market_ticker: bus.kernel.get_market_ticker(3), tombstones },
        };
        let output = AIBlackBox::delta(&mut agent, &input);
        let action = output.a_o;
        let file = File { id: action.file_id.clone(), author: action.author, payload: action.payload.clone(),
            citations: action.citations.clone(), stake: action.stake, intrinsic_reward: 0.0, price: 0.0 };

        match bus.append(file) {
            Ok(_) => {
                info!("[Step {}] File Appended: {}", kernel_steps, action.file_id);
                current_head.paths.insert(action.file_id.clone());
                for cit in &action.citations { current_head.paths.remove(cit); }
                q_state = output.q_o;
                bus.tick_map_reduce();
                if let Some(f) = bus.kernel.tape.files.get(&action.file_id) { if f.price > 0.0 { info!("    => Price: {:.2}", f.price); } }
                if action.payload.contains("[OMEGA]") { info!("OMEGA detected at step {}!", kernel_steps); bus.halt_and_settle(&action.file_id); q_state = MachineState::Halt; }
            }
            Err(e) => { let p: String = action.payload.chars().take(200).collect(); warn!("[Step {}] REJECTED: {} | Payload: {}", kernel_steps, e, p.replace('\n', " ")); }
        }
    }
}
EVALEOF3

# Replace CRATE_NAME placeholder with actual crate name
sed -i "s/CRATE_NAME/${PROJECT_NAME}/g" "$PROJECT_DIR/src/bin/evaluator.rs"

# --- Step 7: Register in workspace ---
echo "[7/10] Registering in workspace..."
if ! grep -q "experiments/$PROJECT_NAME" "$TURINGOS_ROOT/Cargo.toml"; then
    sed -i '/^]/i\    "experiments/'"$PROJECT_NAME"'",' "$TURINGOS_ROOT/Cargo.toml"
fi

# --- Step 8: Cargo check ---
echo "[8/10] Running cargo check..."
cd "$TURINGOS_ROOT"
if cargo check -p "$PROJECT_NAME" 2>&1; then
    echo "✓ cargo check PASSED"
else
    echo "✗ cargo check FAILED"
    exit 1
fi

# --- Step 9: Sync to Mac ---
echo "[9/10] Syncing to Mac..."
rsync -avz --exclude 'target/' "$PROJECT_DIR/" "$MAC_HOST:$MAC_ROOT/experiments/$PROJECT_NAME/"
scp "$TURINGOS_ROOT/Cargo.toml" "$MAC_HOST:$MAC_ROOT/Cargo.toml"

# --- Step 10: Launch tmux ---
echo "[10/10] Launching tmux session '$PROJECT_NAME'..."
ssh "$MAC_HOST" "tmux kill-session -t $PROJECT_NAME 2>/dev/null; \
    echo '' > /tmp/${THEOREM_NAME}_N15.wal && \
    tmux new-session -d -s $PROJECT_NAME \
    'cd $MAC_ROOT/experiments/$PROJECT_NAME && \
    export SILICONFLOW_API_KEY=\$SILICONFLOW_API_KEY && \
    export SILICONFLOW_API_KEY_SECONDARY=\$SILICONFLOW_API_KEY_SECONDARY && \
    export DEEPSEEK_API_KEY=\$DEEPSEEK_API_KEY && \
    unset VOLCENGINE_API_KEY && \
    RUST_LOG=info cargo run --release --bin evaluator 2>&1 | tee /tmp/${PROJECT_NAME}_run1.log'"

echo ""
echo "=== Boot Complete ==="
echo "tmux session: $PROJECT_NAME"
echo "Log: /tmp/${PROJECT_NAME}_run1.log"
echo "WAL: /tmp/${THEOREM_NAME}_N15.wal"
