use std::fs;
use std::path::{Path, PathBuf};
use log::{info, error, warn};
use turingosv3::kernel::{File, Head, Input, Kernel, MachineState, SensorContext};
use turingosv3::sdk::tools::wallet::WalletTool;
use turingosv3::sdk::tool::{AntiZombiePruningTool, OverwhelmingGapArbitratorTool};
use turingosv3::bus::{TuringBus, ThermodynamicHeartbeatTool};
use turingosv3::sdk::sandbox::LocalProcessSandbox;
use minif2f_swarm::lean4_membrane_tool::Lean4MembraneTool;
use minif2f_swarm::swarm::SpeculativeSwarmAgent;

/// Simulates `run_turing_os_v3` but returns (proved, final_balances).
/// If `initial_balances` is provided, wallet inherits those balances instead of fresh 10000.
fn evaluate_theorem(
    problem_name: &str,
    problem_content: &str,
    mut agent: SpeculativeSwarmAgent,
    max_kernel_steps: u64,
    _swarm_size: usize,
    initial_balances: Option<&std::collections::HashMap<String, f64>>,
) -> (bool, std::collections::HashMap<String, f64>) {
    let kernel = Kernel::new();
    let mut bus = TuringBus::new(kernel);

    let sandbox = Box::new(LocalProcessSandbox::new(
        "sh",
        vec![
            "-c".to_string(),
            "cd /Users/zephryj/projects/turingosv3/experiments/minif2f_data_lean4 && source ~/.elan/env && lake env lean /dev/stdin".to_string()
        ]
    ));

    bus.mount_tool(Box::new(ThermodynamicHeartbeatTool::new(1)));
    bus.mount_tool(Box::new(AntiZombiePruningTool::new(3)));
    bus.mount_tool(Box::new(OverwhelmingGapArbitratorTool::new(1.5)));

    // [WALLET] — initialized with persistent balances if available
    let mut wallet = WalletTool::new();
    let agent_ids: Vec<String> = (0..100).map(|i| format!("Agent_{}", i)).collect();

    if let Some(balances) = initial_balances {
        // Cross-theorem balance persistence: inherit balances from previous theorem
        use turingosv3::sdk::tool::TuringTool;
        wallet.on_init(&agent_ids);
        for (agent_id, &balance) in balances {
            wallet.balances.insert(agent_id.clone(), balance);
        }
        info!(">>> [ECONOMY] Inherited balances from previous theorem. Min: {:.2}, Max: {:.2}",
              wallet.balances.values().cloned().fold(f64::INFINITY, f64::min),
              wallet.balances.values().cloned().fold(f64::NEG_INFINITY, f64::max));
    }
    bus.mount_tool(Box::new(wallet));

    bus.mount_tool(Box::new(Lean4MembraneTool::new(
        problem_content.to_string(),
        problem_name.to_string(),
        sandbox
    )));

    if initial_balances.is_none() {
        bus.init_problem(&agent_ids);
    }

    let mut q_state = MachineState::Running;
    let mut current_head = Head { paths: std::collections::HashSet::new() };
    let mut kernel_steps = 0;

    loop {
        if q_state == MachineState::Halt || kernel_steps >= max_kernel_steps {
            let mut proved = false;
            for (_, file) in &bus.kernel.tape.files {
                if file.payload.contains("[OMEGA]") {
                    proved = true;
                    break;
                }
            }
            // Redistribute pool among survivors before extracting balances
            bus.redistribute_pool();
            let mut final_balances = bus.extract_wallet_balances();
            // Rebirth: dead agents get fresh 10000 capital (new consciousness, same body)
            for i in 0..100 {
                let agent_id = format!("Agent_{}", i);
                let balance = final_balances.entry(agent_id.clone()).or_insert(0.0);
                if *balance < 1.0 {
                    info!(">>> [REBIRTH] Agent {} died (balance: {:.2}). New agent awakened with 10000.", agent_id, balance);
                    *balance = 10000.0;
                }
            }
            return (proved, final_balances);
        }

        kernel_steps += 1;

        let mut balances = std::collections::HashMap::new();
        for i in 0..100 { // Assuming max 100 agents for now, or just query for the ones we know
            let agent_id = format!("Agent_{}", i);
            balances.insert(agent_id.clone(), bus.get_agent_balance(&agent_id));
        }

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

        // This blocks and waits for the swarm
        let output = turingosv3::kernel::AIBlackBox::delta(&mut agent, &input);

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
                // Only update Head if node was actually appended (InvestOnly doesn't create nodes)
                if bus.kernel.tape.files.contains_key(&action.file_id) {
                    current_head.paths.insert(action.file_id.clone());
                    for cit in &action.citations {
                        current_head.paths.remove(cit);
                    }
                }
                q_state = output.q_o;
                bus.tick_map_reduce();
                
                // Early exit if this append was the OMEGA node!
                if action.payload.contains("[OMEGA]") {
                    bus.halt_and_settle(&action.file_id);
                    // Pool already settled by halt_and_settle, just extract
                    let final_balances = bus.extract_wallet_balances();
                    return (true, final_balances);
                }
            }
            Err(e) => {
                // Rejected by membrane
                warn!("[Batch] Tactic rejected: {}", e);
            }
        }
    }
}

fn main() {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    let args: Vec<String> = std::env::args().collect();
    let swarm_size: usize = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(5);
    
    info!("Starting Batch Evaluator with N={} LLMs per theorem", swarm_size);

    let api_url = std::env::var("LLAMA_API_URL").unwrap_or_else(|_| "https://api.siliconflow.cn/v1/chat/completions".to_string());
    let primary_model = std::env::var("LLAMA_MODEL").unwrap_or_else(|_| "doubao-1-5-pro-32k-250115".to_string());
    let smart_model = std::env::var("SMART_MODEL").unwrap_or_else(|_| primary_model.clone());
    let smart_model_2 = std::env::var("SMART_MODEL_2").unwrap_or_else(|_| smart_model.clone());
    // Smart model 2 can use a different API provider (e.g., DeepSeek official API)
    let smart_api_url_2 = std::env::var("SMART_API_URL_2").unwrap_or_else(|_| api_url.clone());
    let timeout_secs = 600;

    // Heterogeneous model configuration:
    // Agent 0-2 → primary (fast miners), Agent 3 → smart_1, Agent 4 → smart_2
    let is_heterogeneous = smart_model != primary_model || smart_model_2 != smart_model;
    let models: Vec<(&str, &str)> = if is_heterogeneous {
        info!("Heterogeneous swarm: miners={}, smart1={}, smart2={} (via {})", primary_model, smart_model, smart_model_2, smart_api_url_2);
        vec![
            (&api_url, primary_model.as_str()),
            (&api_url, primary_model.as_str()),
            (&api_url, primary_model.as_str()),
            (&api_url, smart_model.as_str()),
            (&smart_api_url_2, smart_model_2.as_str()),
        ]
    } else {
        info!("Homogeneous swarm: model={}", primary_model);
        vec![(&api_url, primary_model.as_str())]
    };

    // Max steps the swarm is allowed to take to prove ONE theorem
    let max_steps_per_theorem = 100;

    // Target the MacStudio local path for Lean4 dataset
    let test_dir = Path::new("/Users/zephryj/projects/turingosv3/experiments/minif2f_data_lean4/MiniF2F/Test");
    
    // Read the fixed 244 test theorems
    let target_list_path = "/Users/zephryj/projects/turingosv3/experiments/minif2f_swarm/target_244_test_theorems.txt";
    let target_list_content = fs::read_to_string(target_list_path).expect("Could not read target_244_test_theorems.txt");
    let target_files: Vec<String> = target_list_content.lines().map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();

    let mut files: Vec<PathBuf> = Vec::new();
    for f in target_files {
        files.push(test_dir.join(f));
    }

    if files.is_empty() {
        error!("No lean files found");
        return;
    }

    let mut success_count = 0;
    let total_files = files.len();
    let mut persistent_balances: Option<std::collections::HashMap<String, f64>> = None;

    for (i, path) in files.iter().enumerate() {
        let file_name = path.file_stem().unwrap().to_string_lossy().to_string();
        info!("--- Evaluating [{}/{}]: {} ---", i + 1, total_files, file_name);

        let mut content = fs::read_to_string(path).expect("Unable to read lean file");
        let trimmed_content = content.trim_end();
        if trimmed_content.ends_with("by sorry") {
            content = format!("{} by", &trimmed_content[..trimmed_content.len() - 8].trim_end());
        } else if trimmed_content.ends_with("sorry") {
            content = format!("{}", &trimmed_content[..trimmed_content.len() - 5].trim_end());
        }

        let rt = tokio::runtime::Runtime::new().unwrap();
        let _guard = rt.enter();

        let sentinel = minif2f_swarm::wal::WalSentinel::new(format!("/tmp/{}_N{}.wal", file_name, swarm_size));
        let agent = SpeculativeSwarmAgent::new_heterogeneous(models.clone(), max_steps_per_theorem, swarm_size, timeout_secs, sentinel, vec![], content.clone());

        let (proved, final_balances) = evaluate_theorem(
            &file_name, &content, agent, max_steps_per_theorem, swarm_size,
            persistent_balances.as_ref(),
        );

        // Cross-theorem balance persistence: carry balances forward
        persistent_balances = Some(final_balances);

        if proved {
            info!("✅ Theorem {} PROVED!", file_name);
            success_count += 1;
        } else {
            info!("❌ Theorem {} FAILED or TIMEOUT.", file_name);
        }
    }

    info!("==========================================");
    info!("FULL TEST EVALUATION COMPLETE (N={})", swarm_size);
    info!("Score: {}/{} ({:.1}%)", success_count, total_files, (success_count as f64 / total_files as f64) * 100.0);
    info!("==========================================");
}