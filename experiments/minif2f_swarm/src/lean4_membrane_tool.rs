use turingosv3::sdk::tool::{TuringTool, ToolSignal};
use turingosv3::sdk::sandbox::SandboxEngine;
use std::time::Duration;
use log::{info, debug, warn};

pub struct Lean4MembraneTool {
    pub problem_statement: String,
    pub theorem_name: String,
    pub sandbox: Box<dyn SandboxEngine>,
    /// Configurable forbidden tactics — different problem domains need different physics rules.
    /// "sorry" and "sorryAx" are always forbidden (hardcoded, non-negotiable).
    /// For real analysis: add "native_decide" (no shortcut proofs).
    /// For number theory: allow "decide" (legitimate exhaustive search).
    pub forbidden_tactics: Vec<String>,
}

impl Lean4MembraneTool {
    pub fn new(problem_statement: String, theorem_name: String, sandbox: Box<dyn SandboxEngine>) -> Self {
        // Default forbidden list: RCE defense + native_decide
        let forbidden_tactics = vec![
            "#eval", "#check", "#reduce", "#exec",
            "native_decide", "IO.Process", "IO.FS",
            "System.FilePath", "run_tac", "unsafe",
        ].into_iter().map(String::from).collect();
        Self { problem_statement, theorem_name, sandbox, forbidden_tactics }
    }

    /// Create with custom forbidden tactics list (architect directive: configurable membrane)
    pub fn with_config(
        problem_statement: String,
        theorem_name: String,
        sandbox: Box<dyn SandboxEngine>,
        forbidden_tactics: Vec<String>,
    ) -> Self {
        Self { problem_statement, theorem_name, sandbox, forbidden_tactics }
    }

    /// Extract theorem name from a Lean 4 statement
    /// Example: "theorem amc12a_2020_p7 ..." -> "amc12a_2020_p7"
    fn check_identity_theft(&self, payload: &str) -> bool {
        // 1. Ensure our theorem name is present somewhere in the payload
        if !payload.contains(&self.theorem_name) {
            return true;
        }

        // 2. Only scan the LLM-generated INCREMENT (after the problem statement)
        //    for hijacking attempts. The problem statement itself naturally contains
        //    `def`, `instance`, etc. from MiniF2F preambles — those are legitimate.
        let increment = if !self.problem_statement.is_empty() {
            if let Some(idx) = payload.find(&self.problem_statement) {
                &payload[idx + self.problem_statement.len()..]
            } else {
                payload
            }
        } else {
            payload
        };

        // Dangerous definition keywords that could introduce fake theorems
        let hijack_keywords = ["theorem ", "lemma ", "def ", "example ", "instance ",
                               "abbrev ", "axiom ", "constant ", "class ", "structure ",
                               "macro ", "syntax ", "elab "];

        for keyword in &hijack_keywords {
            let mut start = 0;
            while let Some(idx) = increment[start..].find(keyword) {
                let actual_idx = start + idx;
                let after_keyword = &increment[actual_idx + keyword.len()..];
                let found_name = after_keyword.split_whitespace().next().unwrap_or("");
                // Any definition in the increment that isn't our theorem is suspicious
                if !found_name.is_empty() && found_name != self.theorem_name {
                    return true;
                }
                start = actual_idx + keyword.len();
            }
        }

        false
    }
}

impl TuringTool for Lean4MembraneTool {
    fn manifest(&self) -> &'static str {
        "Lean 4 Sandboxed Membrane (Anti-Identity-Theft)"
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }

    fn on_pre_append(&mut self, _author: &str, payload: &str) -> ToolSignal {
        // 1. Cognitive Defense: Check for Identity Theft / Hijacking
        if self.check_identity_theft(payload) {
            warn!(">>> [SHIELD] Identity Theft Detected! LLM tried to prove a different theorem.");
            return ToolSignal::Veto(format!("Identity Theft: Payload does not target theorem {}", self.theorem_name));
        }

        // 2a. Sorry Firewall: LLM must never use sorry (tactic or term mode)
        // Uses word-boundary check to avoid false positives on identifiers like sorry_lemma
        {
            let by_idx = payload.find(":= by");
            let tactic_region = if let Some(idx) = by_idx {
                &payload[idx..]
            } else {
                payload
            };
            for token in ["sorry", "sorryAx"] {
                // Check for word-boundary: sorry must not be preceded/followed by alphanumeric or _
                let mut search_start = 0;
                while let Some(pos) = tactic_region[search_start..].find(token) {
                    let abs_pos = search_start + pos;
                    let before_ok = abs_pos == 0 || !tactic_region.as_bytes()[abs_pos - 1].is_ascii_alphanumeric() && tactic_region.as_bytes()[abs_pos - 1] != b'_';
                    let after_pos = abs_pos + token.len();
                    let after_ok = after_pos >= tactic_region.len() || !tactic_region.as_bytes()[after_pos].is_ascii_alphanumeric() && tactic_region.as_bytes()[after_pos] != b'_';
                    if before_ok && after_ok {
                        warn!(">>> [SHIELD] Sorry/SorryAx detected in payload! Forbidden.");
                        return ToolSignal::Veto(format!("Forbidden: '{}' detected in tactic payload", token));
                    }
                    search_start = abs_pos + token.len();
                }
            }
        }

        // 2b. Configurable Physics Membrane: block forbidden tactics per problem domain
        {
            for kw in &self.forbidden_tactics {
                if payload.contains(kw.as_str()) {
                    warn!(">>> [SHIELD] Forbidden tactic '{}' detected!", kw);
                    return ToolSignal::Veto(format!("Physics Violation: Tactic '{}' is locked in this universe.", kw));
                }
            }
        }

        // 3. Construct the full verification code
        // MiniF2F theorems often end with `by sorry`.
        // The payload passed here should already have been processed to be ready for tactics.
        let test_code = format!(
            "{}\n  sorry", 
            payload
        );

        debug!("--- Lean4 Membrane Testing Code ---\n{}\n-----------------------------------", test_code);

        // 🌟 Dynamic Thermodynamics: Base 10s + 0.5s per Tactic line
        // As the proof tree deepens, we give the compiler more time to unwire it.
        let tactic_count = payload.lines().count() as u64;
        let gas_limit = Duration::from_secs(10 + (tactic_count / 2));

        match self.sandbox.execute_safely(&test_code, gas_limit) {
            Ok(output) => {
                if output.contains("error: No goals to be solved") {
                    info!("🎇 OMEGA NODE REACHED! Theorem proved perfectly! 🎇");
                    return ToolSignal::YieldReward {
                        payload: format!("{}\n  -- [OMEGA]", payload),
                        reward: 0.0, // Law 2: zero post-genesis minting (100B legacy purged)
                    };
                }

                // If it succeeds WITH sorry, it's a valid intermediate step.
                // Now check if it compiles WITHOUT sorry to see if it's the Omega node!
                if let Ok(omega_output) = self.sandbox.execute_safely(payload, gas_limit) {
                    if !omega_output.contains("error:") || omega_output.contains("error: No goals to be solved") {
                        info!("🎇 OMEGA NODE REACHED! Theorem proved perfectly! 🎇");
                        return ToolSignal::YieldReward {
                            payload: format!("{}\n  -- [OMEGA]", payload),
                            reward: 0.0, // Law 2: zero post-genesis minting (100B legacy purged)
                        };
                    }
                }
                
                ToolSignal::Pass
            }
            Err(e) => {
                // Semantic Guillotine (Gemini ruling):
                // "No goals to be solved" = ALL goals closed (Lean 4 only throws this
                // when absolutely zero goals remain). Three-layer defense:
                // Layer 1: sorry firewall (upstream, lines 78-102) blocks LLM sorry
                // Layer 2: Gemini condition check (here) — fast path if clean
                // Layer 3: double-check (fallback) — if other errors present
                if e.contains("No goals to be solved") {
                    let has_other_errors = e.lines()
                        .any(|l| l.contains("error:") && !l.contains("No goals to be solved"));
                    let has_sorry_warning = e.contains("declaration uses 'sorry'");

                    if !has_other_errors && !has_sorry_warning {
                        // Safe OMEGA: sorry purely redundant, no other errors, no cheating
                        info!("OMEGA (Guillotine): No goals + clean output — proof complete!");
                        return ToolSignal::YieldReward {
                            payload: format!("{}\n  -- [OMEGA]", payload),
                            reward: 0.0, // Law 2: zero post-genesis minting (100B legacy purged)
                        };
                    } else {
                        // Ambiguous: other errors or sorry warning → double-check
                        if let Ok(omega_output) = self.sandbox.execute_safely(payload, gas_limit) {
                            if !omega_output.contains("error:") {
                                info!("OMEGA (double-check verified in Err branch)");
                                return ToolSignal::YieldReward {
                                    payload: format!("{}\n  -- [OMEGA]", payload),
                                    reward: 0.0, // Law 2: zero post-genesis minting (100B legacy purged)
                                };
                            }
                        }
                        warn!("Lean4 Membrane VETO: No goals but other errors present (termination/type/sorry).");
                    }
                }
                warn!("Lean4 Membrane VETO: Compiler rejected the tactic or timed out.");
                ToolSignal::Veto(format!("Compiler/Sandbox Error:\n{}", e))
            }
        }
    }
}
