/// Lean 4 Oracle — Popperian Guillotine for MiniF2F v2
///
/// Magna Carta Engine 3 alignment:
/// - "No goals to be solved" → OMEGA (proof complete)
/// - Compiler rejection → Veto (wrong tactic)
/// - Zero reward minting (Polymarket: only invest creates value)
/// - Identity theft protection + sorry firewall + configurable forbidden tactics

use turingosv3::sdk::tool::{TuringTool, ToolSignal};
use turingosv3::sdk::sandbox::SandboxEngine;
use std::time::Duration;
use log::{info, debug, warn};

pub struct Lean4Oracle {
    pub problem_statement: String,
    pub theorem_name: String,
    pub sandbox: Box<dyn SandboxEngine>,
    pub forbidden_tactics: Vec<String>,
}

impl Lean4Oracle {
    pub fn new(problem_statement: String, theorem_name: String, sandbox: Box<dyn SandboxEngine>) -> Self {
        let forbidden_tactics = vec![
            "#eval", "#check", "#reduce", "#exec", "#print",
            "native_decide", "IO.Process", "IO.FS",
            "System.FilePath", "run_tac", "unsafe",
            "dbg_trace", "IO.println", "IO.print",
        ].into_iter().map(String::from).collect();
        Self { problem_statement, theorem_name, sandbox, forbidden_tactics }
    }

    fn check_identity_theft(&self, payload: &str) -> bool {
        // Pure tactic payloads (no "theorem" keyword) are NOT identity theft.
        // Identity theft = agent submits a full declaration proving a DIFFERENT theorem.
        let has_declaration = payload.contains("theorem ") || payload.contains("lemma ");
        if !has_declaration {
            return false; // Pure tactic — no theft possible
        }

        // Full declaration must contain our theorem name
        if !payload.contains(&self.theorem_name) {
            return true; // Declares something but not our theorem
        }

        // Check the increment (after problem statement) for rogue declarations
        let increment = if !self.problem_statement.is_empty() {
            if let Some(idx) = payload.find(&self.problem_statement) {
                &payload[idx + self.problem_statement.len()..]
            } else {
                payload
            }
        } else {
            payload
        };
        let hijack_keywords = ["theorem ", "lemma ", "def ", "example ", "instance ",
                               "abbrev ", "axiom ", "constant ", "class ", "structure ",
                               "macro ", "syntax ", "elab "];
        for keyword in &hijack_keywords {
            let mut start = 0;
            while let Some(idx) = increment[start..].find(keyword) {
                let actual_idx = start + idx;
                let after_keyword = &increment[actual_idx + keyword.len()..];
                let found_name = after_keyword.split_whitespace().next().unwrap_or("");
                if !found_name.is_empty() && found_name != self.theorem_name {
                    return true;
                }
                start = actual_idx + keyword.len();
            }
        }
        false
    }

    fn check_sorry(&self, payload: &str) -> Option<String> {
        let tactic_region = if let Some(idx) = payload.find(":= by") {
            &payload[idx..]
        } else {
            payload
        };
        for token in ["sorry", "sorryAx"] {
            let mut search_start = 0;
            while let Some(pos) = tactic_region[search_start..].find(token) {
                let abs_pos = search_start + pos;
                let before_ok = abs_pos == 0 ||
                    (!tactic_region.as_bytes()[abs_pos - 1].is_ascii_alphanumeric() &&
                     tactic_region.as_bytes()[abs_pos - 1] != b'_');
                let after_pos = abs_pos + token.len();
                let after_ok = after_pos >= tactic_region.len() ||
                    (!tactic_region.as_bytes()[after_pos].is_ascii_alphanumeric() &&
                     tactic_region.as_bytes()[after_pos] != b'_');
                if before_ok && after_ok {
                    return Some(format!("Forbidden: '{}' detected", token));
                }
                search_start = abs_pos + token.len();
            }
        }
        None
    }

    fn is_omega(output: &str) -> bool {
        output.contains("No goals to be solved")
    }
}

impl TuringTool for Lean4Oracle {
    fn manifest(&self) -> &'static str {
        "Lean 4 Oracle (Popperian Guillotine)"
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }

    fn on_pre_append(&mut self, _author: &str, payload: &str) -> ToolSignal {
        // 1. Identity theft check
        if self.check_identity_theft(payload) {
            warn!(">>> [ORACLE] Identity theft! Not targeting {}", self.theorem_name);
            return ToolSignal::Veto(format!("Identity Theft: must target {}", self.theorem_name));
        }

        // 2. Sorry firewall
        if let Some(reason) = self.check_sorry(payload) {
            warn!(">>> [ORACLE] {}", reason);
            return ToolSignal::Veto(reason);
        }

        // 3. Forbidden tactics
        for kw in &self.forbidden_tactics {
            if payload.contains(kw.as_str()) {
                warn!(">>> [ORACLE] Forbidden tactic: {}", kw);
                return ToolSignal::Veto(format!("Forbidden tactic: {}", kw));
            }
        }

        // 4. Compile with sorry appended (test intermediate step validity)
        // If payload is a pure tactic (no "theorem"/"import"), prepend problem_statement
        let full_code = if payload.contains("theorem ") || payload.contains("import ") {
            payload.to_string()
        } else {
            // Pure tactic — prepend problem statement to form valid Lean 4
            format!("{}  {}", self.problem_statement, payload)
        };
        let test_code = format!("{}\n  sorry", full_code);
        let tactic_count = full_code.lines().count() as u64;
        let gas_limit = Duration::from_secs(15 + (tactic_count / 2));

        debug!("--- Lean4 Oracle Test ---\n{}\n---", test_code);

        match self.sandbox.execute_safely(&test_code, gas_limit) {
            Ok(output) => {
                // "No goals to be solved" in Ok path = OMEGA
                if Self::is_omega(&output) {
                    info!(">>> [OMEGA] No goals to be solved! Proof complete.");
                    // Magna Carta: ZERO reward minting. Only tag OMEGA.
                    return ToolSignal::Modify(format!("{}\n  -- [OMEGA:03b17cc758d1492dc24d53ba008e4ed6]", payload));
                }
                // Valid intermediate step — pass through
                // Also check without sorry for potential OMEGA
                if let Ok(omega_out) = self.sandbox.execute_safely(&full_code, gas_limit) {
                    if !omega_out.contains("error:") || Self::is_omega(&omega_out) {
                        info!(">>> [OMEGA] Proof verified without sorry!");
                        return ToolSignal::Modify(format!("{}\n  -- [OMEGA:03b17cc758d1492dc24d53ba008e4ed6]", payload));
                    }
                }
                ToolSignal::Pass
            }
            Err(e) => {
                // "No goals to be solved" in Err path (mixed with other output)
                if Self::is_omega(&e) {
                    let has_other_errors = e.lines()
                        .any(|l| l.contains("error:") && !l.contains("No goals to be solved"));
                    let has_sorry_warning = e.contains("declaration uses 'sorry'");

                    if !has_other_errors && !has_sorry_warning {
                        info!(">>> [OMEGA] Guillotine: No goals + clean = proved!");
                        return ToolSignal::Modify(format!("{}\n  -- [OMEGA:03b17cc758d1492dc24d53ba008e4ed6]", payload));
                    } else {
                        // Double-check without sorry
                        if let Ok(omega_out) = self.sandbox.execute_safely(&full_code, gas_limit) {
                            if !omega_out.contains("error:") {
                                info!(">>> [OMEGA] Double-check verified.");
                                return ToolSignal::Modify(format!("{}\n  -- [OMEGA:03b17cc758d1492dc24d53ba008e4ed6]", payload));
                            }
                        }
                    }
                }
                let truncated_200: String = e.chars().take(200).collect();
                let truncated_500: String = e.chars().take(500).collect();
                warn!(">>> [ORACLE] Compiler rejected: {}", truncated_200);
                ToolSignal::Veto(format!("Compiler Error:\n{}", truncated_500))
            }
        }
    }
}
