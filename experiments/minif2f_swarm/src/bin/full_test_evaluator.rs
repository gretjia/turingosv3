use std::fs;
use std::path::{Path, PathBuf};
use log::{info, error, warn};
use turingosv3::kernel::{File, Head, Input, Kernel, MachineState, SensorContext};
use turingosv3::sdk::skill::{AntiZombiePruningSkill, OverwhelmingGapArbitrator};
use turingosv3::bus::{TuringBus, ThermodynamicHeartbeatSkill};
use turingosv3::sdk::sandbox::LocalProcessSandbox;
use minif2f_swarm::lean4_membrane::Lean4MembraneSkill;
use minif2f_swarm::swarm::SpeculativeSwarmAgent;

/// Simulates `run_turing_os_v3` but returns a boolean indicating whether OMEGA was reached,
/// or false if it failed/timeout.
fn evaluate_theorem(problem_name: &str, problem_content: &str, mut agent: SpeculativeSwarmAgent, max_kernel_steps: u64, _swarm_size: usize) -> bool {
    let kernel = Kernel::new(format!("{}_target", problem_name));
    let mut bus = TuringBus::new(kernel);

    // 1. Instantiate the Air-Gapped Sandbox
    let sandbox = Box::new(LocalProcessSandbox::new(
        "sh", 
        vec![
            "-c".to_string(), 
            "cd /Users/zephryj/projects/turingosv3/experiments/minif2f_data_lean4 && source ~/.elan/env && lake env lean /dev/stdin".to_string()
        ]
    ));

    // 2. Mount Skills
    // 🌟 THE ULTIMATE SOTA BUS CONFIGURATION 🌟

    // [HEARTBEAT] Trigger Reduce frequently but controlled by arbitrator
    bus.mount_skill(Box::new(ThermodynamicHeartbeatSkill::new(1)));

    // [PRUNING] Prevent LLM from getting stuck in repetitive tactic loops (max 3 repeats)
    bus.mount_skill(Box::new(AntiZombiePruningSkill::new(3)));

    // [ARBITRATOR] Only unleash expensive Reduce if price jumps by 50%+
    bus.mount_skill(Box::new(OverwhelmingGapArbitrator::new(1.5)));

    // [MEMBRANE] formal verification with Identity Anchor
    bus.mount_skill(Box::new(Lean4MembraneSkill::new(
        problem_content.to_string(), 
        problem_name.to_string(),
        sandbox
    )));


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
            intrinsic_reward: 0.0,
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
    let swarm_size: usize = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(5);
    
    info!("Starting Batch Evaluator with N={} LLMs per theorem", swarm_size);

    let api_url = std::env::var("LLAMA_API_URL").unwrap_or_else(|_| "https://api.siliconflow.cn/v1/chat/completions".to_string());
    let model_name = std::env::var("LLAMA_MODEL").unwrap_or_else(|_| "deepseek-ai/DeepSeek-V3".to_string());
    let timeout_secs = 600;
    
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

    for (i, path) in files.iter().enumerate() {
        let file_name = path.file_stem().unwrap().to_string_lossy().to_string();
        info!("--- Evaluating [{}/{}]: {} ---", i + 1, total_files, file_name);
        
        let mut content = fs::read_to_string(path).expect("Unable to read lean file");
        // MiniF2F theorems often end with `by sorry`. We must strip this out so our swarm can append its own tactics.
        let trimmed_content = content.trim_end();
        if trimmed_content.ends_with("by sorry") {
            content = format!("{} by", &trimmed_content[..trimmed_content.len() - 8].trim_end());
        } else if trimmed_content.ends_with("sorry") {
            content = format!("{}", &trimmed_content[..trimmed_content.len() - 5].trim_end());
        }
        
        let rt = tokio::runtime::Runtime::new().unwrap();
        let _guard = rt.enter();

        let sentinel = minif2f_swarm::wal::WalSentinel::new(format!("/tmp/{}_N{}.wal", file_name, swarm_size));
        let agent = SpeculativeSwarmAgent::new(&api_url, &model_name, max_steps_per_theorem, swarm_size, timeout_secs, sentinel, vec![], content.clone());
        
        let proved = evaluate_theorem(&file_name, &content, agent, max_steps_per_theorem, swarm_size);
        
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