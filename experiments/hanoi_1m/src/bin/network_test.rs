use turingosv3::kernel::run_turing_os;
use env_logger;

use hanoi_1m::swarm::SpeculativeSwarmAgent;

fn main() {
    // Initialize logging
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    println!("Starting V3 MAKER Hanoi Test on Windows1-W1 GPU Backend...");

    // Default connection to the SSH tunnel setup for the windows node running llama.cpp
    // Assuming the tunnel maps localhost:8080 to the Windows host running Qwen 27B
    let api_url = std::env::var("LLAMA_API_URL").unwrap_or_else(|_| "http://127.0.0.1:8080/v1/chat/completions".to_string());
    let model_name = std::env::var("LLAMA_MODEL").unwrap_or_else(|_| "qwen3.5-27b-instruct-q4_k_m".to_string());
    
    // Testing small scale first to ensure network integration works.
    let target_steps = 10; 
    let final_omega_id = format!("hanoi_step_{}", target_steps);

    let llm_agent = SpeculativeSwarmAgent::new(&api_url, &model_name, target_steps, 4);

    run_turing_os(
        "Hanoi Tower 20 Disks MAKER Logic (Networked)".to_string(),
        llm_agent,
        final_omega_id
    );

    println!("V3 MAKER Hanoi Network Test Complete.");
}
