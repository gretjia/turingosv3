use std::fs;
use std::path::{Path, PathBuf};
use log::{info, error, warn};
use rand::seq::SliceRandom;
use rand::thread_rng;
use turingosv3::kernel::{File, Head, Input, Kernel, MachineState, SensorContext, Output, Action};
use turingosv3::bus::{TuringBus, ThermodynamicHeartbeatTool};
use minif2f_swarm::lean4_membrane_tool::Lean4MembraneTool;
use minif2f_swarm::swarm::SpeculativeSwarmAgent;

/// Simulates `run_turing_os_v3` but returns a boolean indicating whether OMEGA was reached,
/// or false if it failed/timeout.
fn evaluate_theorem(problem_name: &str, problem_content: &str, mut agent: SpeculativeSwarmAgent, max_kernel_steps: u64, swarm_size: usize) -> bool {
    let kernel = Kernel::new(format!("{}_target", problem_name));
    let mut bus = TuringBus::new(kernel);

    // Mount Skills
    bus.mount_tool(Box::new(ThermodynamicHeartbeatTool::new(10)));
    bus.mount_tool(Box::new(Lean4MembraneTool::new(problem_content.to_string(), "/Users/zephryj/projects/turingosv3/experiments/minif2f_data_lean4")));

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
            return proved;
        }

        kernel_steps += 1;

        let input = Input {
            q_i: q_state.clone(),
            s_i: SensorContext {
                visible_tape: bus.kernel.tape.clone(),
                current_head: current_head.clone(),
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
            price: 0.0,
        };

        match bus.append(file) {
            Ok(_) => {
                // Update Head
                current_head.paths.insert(action.file_id.clone());
                for cit in &action.citations {
                    current_head.paths.remove(cit);
                }
                q_state = output.q_o;
                bus.tick_map_reduce();
                
                // Early exit if this append was the OMEGA node!
                if action.payload.contains("[OMEGA]") {
                    return true;
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
    let swarm_size: usize = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(50);
    
    info!("Starting Batch Evaluator with N={} LLMs per theorem", swarm_size);

    let api_url = std::env::var("LLAMA_API_URL").unwrap_or_else(|_| "https://api.siliconflow.cn/v1/chat/completions".to_string());
    let model_name = std::env::var("LLAMA_MODEL").unwrap_or_else(|_| "deepseek-ai/DeepSeek-R1-Distill-Qwen-32B".to_string());
    let timeout_secs = 600;
    
    // Max steps the swarm is allowed to take to prove ONE theorem
    let max_steps_per_theorem = 50; 

    // Target the MacStudio local path for Lean4 dataset
    let valid_dir = Path::new("/Users/zephryj/projects/turingosv3/experiments/minif2f_data_lean4/MiniF2F/Valid");
    
    // Read the fixed 20 theorems
    let target_list_path = "/Users/zephryj/projects/turingosv3/experiments/minif2f_swarm/target_20_theorems.txt";
    let target_list_content = fs::read_to_string(target_list_path).expect("Could not read target_20_theorems.txt");
    let target_files: Vec<String> = target_list_content.lines().map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();

    let mut files: Vec<PathBuf> = Vec::new();
    for f in target_files {
        files.push(valid_dir.join(f));
    }

    if files.is_empty() {
        error!("No lean files found");
        return;
    }

    let mut success_count = 0;

    for (i, path) in files.iter().enumerate() {
        let file_name = path.file_stem().unwrap().to_string_lossy().to_string();
        info!("--- Evaluating [{}/20]: {} ---", i + 1, file_name);
        
        let content = fs::read_to_string(path).expect("Unable to read lean file");
        
        let rt = tokio::runtime::Runtime::new().unwrap();
        let _guard = rt.enter();

        let sentinel = minif2f_swarm::wal::WalSentinel::new(format!("/tmp/{}_N{}.wal", file_name, swarm_size));
        let agent = SpeculativeSwarmAgent::new(&api_url, &model_name, max_steps_per_theorem, swarm_size, timeout_secs, sentinel, vec![]);
        
        let proved = evaluate_theorem(&file_name, &content, agent, max_steps_per_theorem, swarm_size);
        
        if proved {
            info!("✅ Theorem {} PROVED!", file_name);
            success_count += 1;
        } else {
            info!("❌ Theorem {} FAILED or TIMEOUT.", file_name);
        }
    }

    info!("==========================================");
    info!("BATCH EVALUATION COMPLETE (N={})", swarm_size);
    info!("Score: {}/20 ({:.1}%)", success_count, (success_count as f64 / 20.0) * 100.0);
    info!("==========================================");
}