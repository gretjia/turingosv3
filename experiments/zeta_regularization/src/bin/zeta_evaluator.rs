use log::{info, warn};
use std::sync::Arc;
use turingosv3::kernel::{AIBlackBox, File, Head, Input, Kernel, MachineState, SensorContext};
use turingosv3::drivers::llm_http::ResilientLLMClient;
use turingosv3::sdk::tools::wallet::WalletTool;
use turingosv3::sdk::tool::{AntiZombiePruningTool, OverwhelmingGapArbitratorTool};
use turingosv3::bus::{TuringBus, ThermodynamicHeartbeatTool};
use turingosv3::sdk::sandbox::LocalProcessSandbox;
use zeta_regularization::lean4_membrane_tool::Lean4MembraneTool;
use zeta_regularization::swarm::SpeculativeSwarmAgent;

const SWARM_SIZE: usize = 15;
const MAX_KERNEL_STEPS: u64 = 100;
const THEOREM_NAME: &str = "zeta_neg_one";

/// Plan A: ζ(-1) = -1/12 via Mathlib's riemannZeta
const LEAN_PROBLEM: &str = r#"import Mathlib

set_option maxHeartbeats 400000

open Complex in
theorem zeta_neg_one : riemannZeta (-1) = -1/12 := by"#;

fn main() {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    info!("=== ζ(-1) = -1/12 Regularization Theorem Test ===");
    info!("Swarm N={}, Max Steps={}", SWARM_SIZE, MAX_KERNEL_STEPS);

    let api_url = "https://api.siliconflow.cn/v1/chat/completions";

    // Dual API keys for separate rate limits
    let key_primary = std::env::var("SILICONFLOW_API_KEY").expect("SILICONFLOW_API_KEY required");
    let key_secondary = std::env::var("SILICONFLOW_API_KEY_SECONDARY").unwrap_or_else(|_| key_primary.clone());

    // Heterogeneous model pool: R1 (deep reasoning) + V3.2 (fast iteration)
    let client_r1 = Arc::new(ResilientLLMClient::with_key(api_url, "Pro/deepseek-ai/DeepSeek-R1", &key_primary));
    let client_v3 = Arc::new(ResilientLLMClient::with_key(api_url, "Pro/deepseek-ai/DeepSeek-V3.2", &key_secondary));

    info!("Heterogeneous Swarm: R1 (key_primary) + V3.2 (key_secondary)");
    info!("API: {}", api_url);

    // Independent WAL
    let wal_path = format!("/tmp/{}_N{}.wal", THEOREM_NAME, SWARM_SIZE);
    let rt = tokio::runtime::Runtime::new().unwrap();
    let _guard = rt.enter();
    let sentinel = zeta_regularization::wal::WalSentinel::new(wal_path.clone());
    let recovered_files = rt.block_on(zeta_regularization::wal::recover_tape(&wal_path));
    info!("WAL recovered {} files from {}", recovered_files.len(), wal_path);

    // Build heterogeneous swarm: round-robin R1, V3.2
    let mut agent = SpeculativeSwarmAgent::new_multi(
        vec![client_r1, client_v3],
        MAX_KERNEL_STEPS,
        SWARM_SIZE,
        sentinel,
        recovered_files,
        LEAN_PROBLEM.to_string(),
    );

    // Build kernel + bus
    let kernel = Kernel::new();
    let mut bus = TuringBus::new(kernel);

    // Air-gapped Lean 4 sandbox (Mac path)
    let sandbox = Box::new(LocalProcessSandbox::new(
        "sh",
        vec![
            "-c".to_string(),
            "cd /Users/zephryj/projects/turingosv3/experiments/minif2f_data_lean4 && source ~/.elan/env && lake env lean /dev/stdin".to_string(),
        ],
    ));

    // SOTA Tool Stack
    bus.mount_tool(Box::new(ThermodynamicHeartbeatTool::new(1)));
    bus.mount_tool(Box::new(AntiZombiePruningTool::new(3)));
    bus.mount_tool(Box::new(OverwhelmingGapArbitratorTool::new(1.5)));
    bus.mount_tool(Box::new(WalletTool::new()));
    bus.mount_tool(Box::new(Lean4MembraneTool::new(
        LEAN_PROBLEM.to_string(),
        THEOREM_NAME.to_string(),
        sandbox,
    )));

    // Initialize 100 agent wallets in economy (match minif2f SOTA config)
    // Swarm only uses N=SWARM_SIZE concurrently, but liquidation guard may probe up to 100
    let agent_ids: Vec<String> = (0..100).map(|i| format!("Agent_{}", i)).collect();
    bus.init_problem(&agent_ids);

    info!(">>> TuringOS v3 Booted for ζ(-1) theorem. N={}, Max Steps={}. <<<", SWARM_SIZE, MAX_KERNEL_STEPS);

    let mut q_state = MachineState::Running;
    let mut current_head = Head { paths: std::collections::HashSet::new() };
    let mut kernel_steps: u64 = 0;

    loop {
        if q_state == MachineState::Halt || kernel_steps >= MAX_KERNEL_STEPS {
            info!("==== EVALUATION COMPLETE (steps={}) ====", kernel_steps);

            // Final map-reduce
            bus.kernel.hayekian_map_reduce();

            // Check for OMEGA
            let mut proved = false;
            for (_, file) in &bus.kernel.tape.files {
                if file.payload.contains("[OMEGA]") {
                    proved = true;
                    break;
                }
            }

            // Tape dump for analysis
            info!("--- TAPE AUDIT DUMP ---");
            for (id, file) in &bus.kernel.tape.files {
                info!(
                    "ID: {} | Parent: {:?} | Price: {:.2} | Reward: {:.2} | Payload: {}",
                    id,
                    file.citations,
                    file.price,
                    file.intrinsic_reward,
                    file.payload.chars().take(200).collect::<String>().replace('\n', " ")
                );
            }
            info!("-----------------------");

            if proved {
                info!("OMEGA: zeta(-1) = -1/12 PROVED by swarm!");
            } else {
                info!("zeta(-1) = -1/12 NOT proved within {} steps. Tape data saved for analysis.", kernel_steps);
            }
            break;
        }

        kernel_steps += 1;

        // Build balances map (query all 100 funded wallets)
        let mut balances = std::collections::HashMap::new();
        for i in 0..100 {
            let agent_id = format!("Agent_{}", i);
            balances.insert(agent_id.clone(), bus.get_agent_balance(&agent_id));
        }

        // Build tombstones
        let mut tombstones = std::collections::HashMap::new();
        for id in bus.kernel.tape.files.keys() {
            let graves = bus.get_tombstones(id);
            if !graves.is_empty() {
                tombstones.insert(id.clone(), graves);
            }
        }

        let input = Input {
            q_i: q_state.clone(),
            s_i: SensorContext {
                visible_tape: bus.kernel.tape.clone(),
                current_head: current_head.clone(),
                agent_balances: balances,
                market_ticker: bus.kernel.get_market_ticker(3),
                tombstones,
            },
        };

        let output = AIBlackBox::delta(&mut agent, &input);

        let action = output.a_o;
        let file = File {
            id: action.file_id.clone(),
            author: action.author,
            payload: action.payload.clone(),
            citations: action.citations.clone(),
            stake: action.stake,
            intrinsic_reward: 0.0,
            price: 0.0,
        };

        match bus.append(file) {
            Ok(_) => {
                info!("[Step {}] File Appended: {}", kernel_steps, action.file_id);
                current_head.paths.insert(action.file_id.clone());
                for cit in &action.citations {
                    current_head.paths.remove(cit);
                }
                q_state = output.q_o;
                bus.tick_map_reduce();

                if let Some(f) = bus.kernel.tape.files.get(&action.file_id) {
                    if f.price > 0.0 {
                        info!("    => Price: {:.2}", f.price);
                    }
                }

                // Early OMEGA exit
                if action.payload.contains("[OMEGA]") {
                    info!("OMEGA detected at step {}!", kernel_steps);
                    bus.halt_and_settle(&action.file_id);
                    q_state = MachineState::Halt;
                }
            }
            Err(e) => {
                let payload_preview: String = action.payload.chars().take(200).collect();
                warn!("[Step {}] REJECTED: {} | Payload: {}", kernel_steps, e, payload_preview.replace('\n', " "));
            }
        }
    }
}
