use turingosv3::sdk::skill::{TuringSkill, SkillSignal};
use std::process::Command;
use std::fs;
use std::path::Path;
use log::{info, debug, warn};

pub struct Lean4MembraneSkill {
    pub problem_statement: String,
    pub work_dir: String,
}

impl Lean4MembraneSkill {
    pub fn new(problem_statement: String, work_dir: &str) -> Self {
        Self { 
            problem_statement,
            work_dir: work_dir.to_string()
        }
    }

    fn verify_lean_code(&self, code: &str) -> Result<String, String> {
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let rand_id = rand::random::<u64>();
        let unique_filename = format!("temp_proof_{}_{}.lean", timestamp, rand_id);
        let temp_file = Path::new(&self.work_dir).join(&unique_filename);
        
        // Write the code to a temporary file
        if let Err(e) = fs::write(&temp_file, code) {
            return Err(format!("Failed to write temp file: {}", e));
        }

        // Run the lean compiler
        // Using `~/.elan/bin/lake env lean` inside the project dir to pick up Mathlib
        let output = Command::new("sh")
            .arg("-c")
            .arg(format!("cd ~/projects/turingosv3/experiments/minif2f_data_lean4 && source ~/.elan/env && lake env lean {}", temp_file.display()))
            .output();

        // Clean up the temporary file so we don't pollute the drive
        let _ = fs::remove_file(&temp_file);

        match output {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                let combined = format!("{}{}", stdout, stderr);
                
                if combined.contains("command not found") {
                    Err(combined)
                } else if combined.contains("error: No goals to be solved") {
                    // This is actually a massive SUCCESS! 
                    // It means the Tactic solved the goal, so our appended `sorry` became invalid!
                    Ok("OMEGA".to_string())
                } else if combined.contains("error:") {
                    Err(combined)
                } else {
                    Ok(combined)
                }
            }
            Err(e) => Err(format!("Failed to execute lean: {}", e))
        }
    }
}

impl TuringSkill for Lean4MembraneSkill {
    fn manifest(&self) -> &'static str {
        "Lean 4 Membrane Guillotine (Formal Verification)"
    }

    fn on_pre_append(&mut self, payload: &str) -> SkillSignal {
        // Construct the full lean proof attempt.
        // We append 'sorry' to suppress the "unsolved goals" error, so we only catch actual syntax/logic errors.
        let test_code = format!(
            "{}\n  sorry", 
            payload
        );

        debug!("--- Lean4 Membrane Testing Code ---\n{}\n-----------------------------------", test_code);

        match self.verify_lean_code(&test_code) {
            Ok(output) => {
                if output == "OMEGA" {
                    info!("🎇 OMEGA NODE REACHED! Theorem proved perfectly! 🎇");
                    return SkillSignal::Modify(format!("{}\n  -- [OMEGA]", payload));
                }

                // Wait, if it succeeds WITH sorry, it's just a valid intermediate step.
                // We should check if it compiles WITHOUT sorry to see if it's the Omega node!
                let omega_code = payload.to_string();
                
                if let Ok(_omega_output) = self.verify_lean_code(&omega_code) {
                    info!("🎇 OMEGA NODE REACHED! Theorem proved perfectly! 🎇");
                    // Modify the payload to inject a special flag for the kernel to recognize if needed,
                    // or just pass the clean tactic.
                    SkillSignal::Modify(format!("{}\n  -- [OMEGA]", payload))
                } else {
                    // Valid intermediate step
                    SkillSignal::Pass
                }
            }
            Err(e) => {
                // The compiler threw an error. VETO!
                debug!("Lean4 Membrane VETO: Compiler rejected the tactic.");
                SkillSignal::Veto(format!("Compiler Error:\n{}", e))
            }
        }
    }
}