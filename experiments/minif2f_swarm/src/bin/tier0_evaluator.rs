use std::fs;
use std::path::{Path, PathBuf};
use log::{info, error, warn};
use turingosv3::drivers::llm_http::ResilientLLMClient;
use std::sync::Arc;
use std::process::Command;

/// This script evaluates Tier 0: Direct Zero-Shot completion without TuringOS feedback loop.
/// It asks the LLM to output the entire Lean 4 proof in one go, and tests if it compiles.
fn main() {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    let api_url = std::env::var("LLAMA_API_URL").unwrap_or_else(|_| "https://api.siliconflow.cn/v1/chat/completions".to_string());
    let model_name = std::env::var("LLAMA_MODEL").unwrap_or_else(|_| "deepseek-ai/DeepSeek-R1-Distill-Qwen-32B".to_string());
    
    info!("Starting Tier 0 Evaluator (Pure Zero-Shot without TuringOS)");

    let target_list_path = "/Users/zephryj/projects/turingosv3/experiments/minif2f_swarm/target_20_theorems.txt";
    let target_list_content = fs::read_to_string(target_list_path).expect("Could not read target_20_theorems.txt");
    let target_files: Vec<String> = target_list_content.lines().map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();

    let valid_dir = Path::new("/Users/zephryj/projects/turingosv3/experiments/minif2f_data_lean4/MiniF2F/Valid");
    
    let rt = tokio::runtime::Runtime::new().unwrap();
    let client = Arc::new(ResilientLLMClient::new(&api_url, &model_name, 600));

    let mut success_count = 0;

    for (i, file_name) in target_files.iter().enumerate() {
        let path = valid_dir.join(file_name);
        let base_name = path.file_stem().unwrap().to_string_lossy().to_string();
        info!("--- Evaluating Tier 0 [{}/20]: {} ---", i + 1, base_name);
        
        let theorem_statement = fs::read_to_string(&path).expect("Unable to read lean file");
        
        let prompt = format!(
            "You are a Lean 4 expert. Provide the COMPLETE Lean 4 proof for the following theorem in one go.\n\n{}\n\nOutput your ENTIRE proof starting with the tactic block. DO NOT use 'sorry'. Enclose your final proof block inside [Proof: ... ] tags.",
            theorem_statement
        );

        let response = rt.block_on(async {
            client.resilient_generate(&prompt, 0, 0.0).await
        });

        match response {
            Ok(raw_text) => {
                // Extract the proof block
                let mut proof_code = "".to_string();
                if let Some(start) = raw_text.find("[Proof:") {
                    let slice = &raw_text[start+7..];
                    if let Some(end) = slice.rfind(']') {
                        proof_code = slice[..end].trim().to_string();
                    } else {
                        proof_code = slice.trim().to_string();
                    }
                } else {
                    // Fallback to just using the raw text and hoping it compiles
                    proof_code = raw_text;
                }

                let full_code = format!("{}\n  {}", theorem_statement, proof_code);
                
                // Write and test
                let temp_file = Path::new("/Users/zephryj/projects/turingosv3/experiments/minif2f_data_lean4/").join(format!("tier0_test_{}.lean", base_name));
                fs::write(&temp_file, &full_code).unwrap();
                
                let output = Command::new("sh")
                    .arg("-c")
                    .arg(format!("cd ~/projects/turingosv3/experiments/minif2f_data_lean4 && source ~/.elan/env && lake env lean {}", temp_file.display()))
                    .output();
                    
                fs::remove_file(&temp_file).unwrap();
                
                match output {
                    Ok(out) => {
                        let combined = format!("{}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr));
                        if !combined.contains("error:") && !combined.contains("sorry") {
                            info!("✅ Theorem {} PROVED! (Tier 0)", base_name);
                            success_count += 1;
                        } else {
                            warn!("❌ Theorem {} FAILED! (Compiler Error/Sorry)", base_name);
                        }
                    }
                    Err(e) => {
                        warn!("❌ Theorem {} FAILED to run compiler: {}", base_name, e);
                    }
                }
            }
            Err(e) => {
                error!("❌ API Call Failed: {:?}", e);
            }
        }
    }
    info!("==========================================");
    info!("TIER 0 EVALUATION COMPLETE");
    info!("Score: {}/20 ({:.1}%)", success_count, (success_count as f64 / 20.0) * 100.0);
    info!("==========================================");
}