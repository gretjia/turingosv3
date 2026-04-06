use env_logger;

use hanoi_1m::swarm::SpeculativeSwarmAgent;
use std::collections::HashSet;
use turingosv3::kernel::{AIBlackBox, File, Head, Input, Kernel, MachineState, Q, SensorContext};
use turingosv3::bus::{TuringBus, MembraneGuardTool, ThermodynamicHeartbeatTool, WalSnapshotTool};

pub fn run_turing_os_v3(human_spec: String, mut ai: impl AIBlackBox) {
    let kernel = Kernel::new();
    let mut bus = TuringBus::new(kernel);

    bus.mount_tool(Box::new(ThermodynamicHeartbeatTool::new(10)));
    bus.mount_tool(Box::new(MembraneGuardTool));
    bus.mount_tool(Box::new(WalSnapshotTool));

    println!(">>> TuringOS v3 Booted. Awaiting HALT. [{}] <<<", human_spec);

    let mut q_state = MachineState::Running;
    let mut current_head = Head { paths: HashSet::new() };

    loop {
        if q_state == MachineState::Halt {
            println!("==== [HALT] DOUBLE-CIRCLE REACHED. UNIVERSE FROZEN. ====");
            println!(">>> Forcing Final MapReduce Settlement (Judgment Day)... <<<");
            bus.kernel.hayekian_map_reduce();
            
            println!("--- TAPE AUDIT DUMP ---");
            for (id, file) in &bus.kernel.tape.files {
                println!("ID: {} | Parent: {:?} | Price: {:.2} | Payload: {}", id, file.citations, file.price, file.payload.replace('\n', " "));
            }
            println!("-----------------------");
            break;
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
                agent_balances: std::collections::HashMap::new(),
                market_ticker: "".to_string(),
                tombstones,
            },
        };

        let output = ai.delta(&input);

        let action = output.a_o;
        let file = File {
            id: action.file_id.clone(),
            author: action.author,
            payload: action.payload.clone(),
            citations: action.citations.clone(),
            stake: action.stake,
            intrinsic_reward: 0.0,
            price: 0.0,
            created_at: 0,
            completion_tokens: 0,
        };

        match bus.append(file) {
            Ok(_) => {
                println!("[+] File Appended: {}", action.file_id);
                // Update Head
                current_head.paths.insert(action.file_id.clone());
                for cit in &action.citations {
                    current_head.paths.remove(cit);
                }
                q_state = output.q_o;
                bus.tick_map_reduce();
                // Optional debug to see price change (only if reduced)
                if bus.kernel.tape.files.get(&action.file_id).unwrap().price > 0.0 {
                     println!("    => Imputed Price: {:.2}", bus.kernel.tape.files.get(&action.file_id).unwrap().price);
                }
            }
            Err(e) => {
                println!("[-] REJECTED by SKILL: {}", e);
            }
        }
    }
}

fn main() {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    println!("Starting V3 MAKER Hanoi Test on Windows1-W1 GPU Backend...");

    let api_url = std::env::var("LLAMA_API_URL").unwrap_or_else(|_| "https://api.siliconflow.cn/v1/chat/completions".to_string());
    let model_name = std::env::var("LLAMA_MODEL").unwrap_or_else(|_| "Qwen/Qwen2.5-7B-Instruct".to_string());
    let timeout_secs: u64 = std::env::var("LLAMA_TIMEOUT")
        .unwrap_or_else(|_| "600".to_string())
        .parse()
        .unwrap_or(600);
    
    let target_steps = 100_000;

    let wal_path = "hanoi_1m_recovery.wal".to_string();
    let rt = tokio::runtime::Runtime::new().unwrap();
    let _guard = rt.enter();
    let sentinel = hanoi_1m::wal::WalSentinel::new(wal_path.clone());
    let recovered_files = rt.block_on(hanoi_1m::wal::recover_tape(&wal_path));
    
    println!("Bootloader resurrected {} files from WAL.", recovered_files.len());

    let llm_agent = SpeculativeSwarmAgent::new(&api_url, &model_name, target_steps, 100, timeout_secs, sentinel, recovered_files);

    run_turing_os_v3(
        "Hanoi Tower 20 Disks MAKER Logic (Networked)".to_string(),
        llm_agent
    );

    println!("V3 MAKER Hanoi Network Test Complete.");
}
