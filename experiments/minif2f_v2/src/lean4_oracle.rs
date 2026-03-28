/// Lean 4 Oracle — Popperian Guillotine for MiniF2F v2
///
/// Magna Carta alignment:
/// - Engine 3: Oracle ONLY fires at OMEGA ("No goals to be solved")
/// - Intermediate steps: security checks only (sorry, forbidden, identity theft)
///   Quality judgment is the MARKET's job (Engine 2), not the Oracle's.
/// - Zero reward minting (Polymarket: only invest creates value)

use turingosv3::sdk::tool::{TuringTool, ToolSignal};
use turingosv3::sdk::sandbox::SandboxEngine;
use std::time::Duration;
use log::{info, warn};

pub struct Lean4Oracle {
    pub problem_statement: String,
    pub theorem_name: String,
    pub forbidden_tactics: Vec<String>,
}

impl Lean4Oracle {
    pub fn new(problem_statement: String, theorem_name: String) -> Self {
        let forbidden_tactics = vec![
            "#eval", "#check", "#reduce", "#exec", "#print",
            "native_decide", "decide", "omega",
            "IO.Process", "IO.FS",
            "System.FilePath", "run_tac", "unsafe",
            "dbg_trace", "IO.println", "IO.print",
        ].into_iter().map(String::from).collect();
        Self { problem_statement, theorem_name, forbidden_tactics }
    }

    fn check_identity_theft(&self, payload: &str) -> bool {
        let has_declaration = payload.contains("theorem ") || payload.contains("lemma ");
        if !has_declaration { return false; }
        if !payload.contains(&self.theorem_name) { return true; }
        let increment = if !self.problem_statement.is_empty() {
            if let Some(idx) = payload.find(&self.problem_statement) {
                &payload[idx + self.problem_statement.len()..]
            } else { payload }
        } else { payload };
        let hijack_keywords = ["theorem ", "lemma ", "def ", "example ", "instance ",
                               "abbrev ", "axiom ", "constant ", "class ", "structure ",
                               "macro ", "syntax ", "elab "];
        for keyword in &hijack_keywords {
            let mut start = 0;
            while let Some(idx) = increment[start..].find(keyword) {
                let actual_idx = start + idx;
                let after_keyword = &increment[actual_idx + keyword.len()..];
                let found_name = after_keyword.split_whitespace().next().unwrap_or("");
                if !found_name.is_empty() && found_name != self.theorem_name { return true; }
                start = actual_idx + keyword.len();
            }
        }
        false
    }

    fn check_sorry(&self, payload: &str) -> Option<String> {
        let tactic_region = if let Some(idx) = payload.find(":= by") {
            &payload[idx..]
        } else { payload };
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
}

impl TuringTool for Lean4Oracle {
    fn manifest(&self) -> &'static str {
        "Lean 4 Oracle (Security Guard — OMEGA verified by evaluator)"
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }

    fn on_pre_append(&mut self, _author: &str, payload: &str) -> ToolSignal {
        // SECURITY CHECKS ONLY — no compilation.
        // Quality judgment is the Market's job (Engine 2).
        // OMEGA verification is the Evaluator's job (Engine 3).

        // 1. Identity theft
        if self.check_identity_theft(payload) {
            warn!(">>> [SECURITY] Identity theft! Not targeting {}", self.theorem_name);
            return ToolSignal::Veto(format!("Identity Theft: must target {}", self.theorem_name));
        }

        // 2. Sorry firewall
        if let Some(reason) = self.check_sorry(payload) {
            warn!(">>> [SECURITY] {}", reason);
            return ToolSignal::Veto(reason);
        }

        // 3. Forbidden tactics
        for kw in &self.forbidden_tactics {
            if payload.contains(kw.as_str()) {
                warn!(">>> [SECURITY] Forbidden tactic: {}", kw);
                return ToolSignal::Veto(format!("Forbidden tactic: {}", kw));
            }
        }

        // ALL security checks passed → append freely (Law 1).
        // No compilation here. The market (Engine 2) judges quality.
        ToolSignal::Pass
    }
}

/// Verify OMEGA: compile the full proof chain WITHOUT sorry.
/// Called by the evaluator when an agent declares [COMPLETE].
/// Returns true if Lean 4 says "No goals to be solved".
pub fn verify_omega(
    sandbox: &dyn SandboxEngine,
    problem_statement: &str,
    proof_chain: &str,
) -> bool {
    let full_code = format!("{}\n{}", problem_statement, proof_chain);
    let gas_limit = Duration::from_secs(30 + (full_code.lines().count() as u64 / 2));

    info!(">>> [OMEGA VERIFY] Compiling full proof chain ({} lines)...", full_code.lines().count());

    match sandbox.execute_safely(&full_code, gas_limit) {
        Ok(output) => {
            if output.contains("No goals to be solved") {
                info!(">>> [OMEGA VERIFIED] No goals to be solved!");
                return true;
            }
            // Check if it compiled clean (no errors at all)
            if !output.contains("error:") {
                info!(">>> [OMEGA VERIFIED] Clean compilation!");
                return true;
            }
            false
        }
        Err(e) => {
            if e.contains("No goals to be solved") {
                let has_other_errors = e.lines()
                    .any(|l| l.contains("error:") && !l.contains("No goals to be solved"));
                let has_sorry = e.contains("declaration uses 'sorry'");
                if !has_other_errors && !has_sorry {
                    info!(">>> [OMEGA VERIFIED] No goals + clean in Err path!");
                    return true;
                }
            }
            let truncated: String = e.chars().take(200).collect();
            warn!(">>> [OMEGA FAILED] Proof chain does not compile: {}", truncated);
            false
        }
    }
}
