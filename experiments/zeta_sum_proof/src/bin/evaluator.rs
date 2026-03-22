use log::{info, warn};
use std::sync::Arc;
use turingosv3::kernel::{AIBlackBox, File, Head, Input, Kernel, MachineState, SensorContext};
use turingosv3::drivers::llm_http::ResilientLLMClient;
use turingosv3::sdk::tools::wallet::WalletTool;
use turingosv3::sdk::tool::{AntiZombiePruningTool, OverwhelmingGapArbitratorTool};
use turingosv3::bus::{TuringBus, ThermodynamicHeartbeatTool};
use zeta_sum_proof::math_membrane::MathStepMembrane;
use zeta_sum_proof::swarm::SpeculativeSwarmAgent;

const SWARM_SIZE: usize = 15;
const MAX_KERNEL_STEPS: u64 = 100;
const THEOREM_NAME: &str = "zeta_sum_regularization";

/// Problem statement + hint formula
/// Each agent sees this at every step, plus the current best proof chain
const PROBLEM: &str = r#"PROVE: 1 + 2 + 3 + 4 + ... = -1/12 (in the sense of regularization)

HINT FORMULA: M(m,N) = m * exp(-m/N) * cos(m/N)

KEY IDEA: For each fixed N, the series S(N) = Σ_{m=0}^∞ M(m,N) converges.
The limit as N→∞ of S(N) equals -1/12, even though the ordinary sum Σm diverges.

RULES:
- Write exactly ONE mathematical reasoning step
- Use only university-level calculus (series, limits, complex exponentials)
- Your step must logically follow from the previous steps shown above
- When the proof reaches -1/12, declare [COMPLETE]"#;

fn main() {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    info!("=== ζ-Sum Regularization Proof: 1+2+3+... = -1/12 ===");
    info!("Pure market validation — no Lean 4 intermediate checks");
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
    let sentinel = zeta_sum_proof::wal::WalSentinel::new(wal_path.clone());
    let recovered_files = rt.block_on(zeta_sum_proof::wal::recover_tape(&wal_path));
    info!("WAL recovered {} files from {}", recovered_files.len(), wal_path);

    let mut agent = SpeculativeSwarmAgent::new_multi(
        vec![client_miner, client_scholar, client_explorer],
        MAX_KERNEL_STEPS, SWARM_SIZE, sentinel, recovered_files, PROBLEM.to_string(),
    );

    let kernel = Kernel::new();
    let mut bus = TuringBus::new(kernel);

    // Tool stack: NO Lean 4 sandbox — MathStepMembrane instead
    bus.mount_tool(Box::new(ThermodynamicHeartbeatTool::new(1)));
    bus.mount_tool(Box::new(AntiZombiePruningTool::new(3)));
    bus.mount_tool(Box::new(OverwhelmingGapArbitratorTool::new(1.5)));
    bus.mount_tool(Box::new(WalletTool::new()));
    bus.mount_tool(Box::new(MathStepMembrane::new()));

    let agent_ids: Vec<String> = (0..100).map(|i| format!("Agent_{}", i)).collect();
    bus.init_problem(&agent_ids);

    info!(">>> TuringOS v3 Booted (Market-Only Mode). N={}, Max Steps={}. <<<", SWARM_SIZE, MAX_KERNEL_STEPS);

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

            // Trace golden path for proof chain
            info!("--- PROOF CHAIN (Golden Path) ---");
            if let Some(omega_node) = bus.kernel.tape.files.values().find(|f| f.payload.contains("[OMEGA]")) {
                let path = bus.kernel.trace_golden_path(&omega_node.id);
                for (i, node_id) in path.iter().rev().enumerate() {
                    if let Some(node) = bus.kernel.tape.files.get(node_id) {
                        let step = node.payload.lines().last().unwrap_or(&node.payload).trim();
                        info!("Step {}: [{}] Price:{:.0} | {}", i + 1, node_id, node.price, step);
                    }
                }
            }

            info!("--- FULL TAPE DUMP ---");
            for (id, file) in &bus.kernel.tape.files {
                info!("ID: {} | Parent: {:?} | Price: {:.2} | Payload: {}",
                    id, file.citations, file.price,
                    file.payload.chars().take(150).collect::<String>().replace('\n', " "));
            }
            info!("-----------------------");

            if proved {
                info!("OMEGA: Proof chain COMPLETE! Submit to terminal oracle (Lean 4) for final verification.");
            } else {
                info!("NOT proved within {} steps.", kernel_steps);
            }
            break;
        }

        kernel_steps += 1;

        let mut balances = std::collections::HashMap::new();
        for i in 0..100 {
            let aid = format!("Agent_{}", i);
            balances.insert(aid.clone(), bus.get_agent_balance(&aid));
        }

        let mut tombstones = std::collections::HashMap::new();
        for id in bus.kernel.tape.files.keys() {
            let g = bus.get_tombstones(id);
            if !g.is_empty() { tombstones.insert(id.clone(), g); }
        }
        let rg = bus.get_tombstones("root");
        if !rg.is_empty() { tombstones.insert("root".to_string(), rg); }

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
            id: action.file_id.clone(), author: action.author,
            payload: action.payload.clone(), citations: action.citations.clone(),
            stake: action.stake, intrinsic_reward: 0.0, price: 0.0,
        };

        match bus.append(file) {
            Ok(_) => {
                info!("[Step {}] File Appended: {}", kernel_steps, action.file_id);
                current_head.paths.insert(action.file_id.clone());
                for cit in &action.citations { current_head.paths.remove(cit); }
                q_state = output.q_o;
                bus.tick_map_reduce();
                if let Some(f) = bus.kernel.tape.files.get(&action.file_id) {
                    if f.price > 0.0 { info!("    => Price: {:.2}", f.price); }
                }
                if action.payload.contains("[OMEGA]") {
                    info!("OMEGA detected at step {}!", kernel_steps);
                    bus.halt_and_settle(&action.file_id);
                    q_state = MachineState::Halt;
                }
            }
            Err(e) => {
                let p: String = action.payload.chars().take(150).collect();
                warn!("[Step {}] REJECTED: {} | Payload: {}", kernel_steps, e, p.replace('\n', " "));
            }
        }
    }
}
