use turingosv3::sdk::skill::{TuringSkill, SkillSignal};
use turingosv3::sdk::sandbox::SandboxEngine;
use std::time::Duration;
use log::{info, debug, warn};

pub struct Lean4MembraneSkill {
    pub problem_statement: String,
    pub theorem_name: String,
    pub sandbox: Box<dyn SandboxEngine>,
}

impl Lean4MembraneSkill {
    pub fn new(problem_statement: String, theorem_name: String, sandbox: Box<dyn SandboxEngine>) -> Self {
        Self { 
            problem_statement,
            theorem_name,
            sandbox,
        }
    }

    /// Extract theorem name from a Lean 4 statement
    /// Example: "theorem amc12a_2020_p7 ..." -> "amc12a_2020_p7"
    fn check_identity_theft(&self, payload: &str) -> bool {
        // 1. Ensure our theorem name is present as a definition
        if !payload.contains(&format!("theorem {}", self.theorem_name)) {
            return true;
        }
        
        // 2. Ensure no OTHER theorem is being defined (hijacking defense)
        let theorem_keyword = "theorem ";
        let mut start = 0;
        while let Some(idx) = payload[start..].find(theorem_keyword) {
            let actual_idx = start + idx;
            let after_theorem = &payload[actual_idx + theorem_keyword.len()..];
            
            // Check if what follows "theorem " is NOT our name
            // We look at the first word after the keyword
            let found_name = after_theorem.split_whitespace().next().unwrap_or("");
            if found_name != self.theorem_name && !found_name.is_empty() {
                return true; // Found a definition for a different theorem!
            }
            start = actual_idx + theorem_keyword.len();
        }

        false
    }
}

impl TuringSkill for Lean4MembraneSkill {
    fn manifest(&self) -> &'static str {
        "Lean 4 Sandboxed Membrane (Anti-Identity-Theft)"
    }

    fn on_pre_append(&mut self, payload: &str) -> SkillSignal {
        // 1. Cognitive Defense: Check for Identity Theft / Hijacking
        if self.check_identity_theft(payload) {
            warn!(">>> [SHIELD] Identity Theft Detected! LLM tried to prove a different theorem.");
            return SkillSignal::Veto(format!("Identity Theft: Payload does not target theorem {}", self.theorem_name));
        }

        // 2. Construct the full verification code
        // MiniF2F theorems often end with `by sorry`. 
        // The payload passed here should already have been processed to be ready for tactics.
        let test_code = format!(
            "{}\n  sorry", 
            payload
        );

        debug!("--- Lean4 Membrane Testing Code ---\n{}\n-----------------------------------", test_code);

        // 🌟 Thermodynamics: Max 10 seconds for Lean to compute
        let gas_limit = Duration::from_secs(10);

        match self.sandbox.execute_safely(&test_code, gas_limit) {
            Ok(output) => {
                if output.contains("error: No goals to be solved") {
                    info!("🎇 OMEGA NODE REACHED! Theorem proved perfectly! 🎇");
                    return SkillSignal::Modify(format!("{}\n  -- [OMEGA]", payload));
                }

                // If it succeeds WITH sorry, it's a valid intermediate step.
                // Now check if it compiles WITHOUT sorry to see if it's the Omega node!
                if let Ok(omega_output) = self.sandbox.execute_safely(payload, gas_limit) {
                    if !omega_output.contains("error:") || omega_output.contains("error: No goals to be solved") {
                        info!("🎇 OMEGA NODE REACHED! Theorem proved perfectly! 🎇");
                        return SkillSignal::Modify(format!("{}\n  -- [OMEGA]", payload));
                    }
                }
                
                SkillSignal::Pass
            }
            Err(e) => {
                // The compiler threw an error or timed out. VETO!
                debug!("Lean4 Membrane VETO: Compiler rejected the tactic or timed out.");
                SkillSignal::Veto(format!("Compiler/Sandbox Error:\n{}", e))
            }
        }
    }
}
