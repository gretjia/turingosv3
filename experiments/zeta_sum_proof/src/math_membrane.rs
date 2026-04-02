use turingosv3::sdk::tool::{TuringTool, ToolSignal};
use log::{info, warn};

/// Math Step Membrane — Pure Market Validation
///
/// No compiler, no numerical checker. The market is the truth arbiter.
/// Steps are validated only for non-emptiness.
/// [COMPLETE] triggers OMEGA (pending terminal Lean 4 oracle).
pub struct MathStepMembrane;

impl MathStepMembrane {
    pub fn new() -> Self { Self }
}

impl TuringTool for MathStepMembrane {
    fn manifest(&self) -> &'static str {
        "Math Step Membrane (Market-Only Validation)"
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }

    fn on_pre_append(&mut self, _author: &str, payload: &str) -> ToolSignal {
        // Extract the step content (after the problem statement prefix)
        let step = if let Some(idx) = payload.rfind('\n') {
            payload[idx..].trim()
        } else {
            payload.trim()
        };

        // Empty or trivially short step → VETO (filters prompt template leaks like "your step")
        if step.len() < 20 {
            warn!(">>> [MEMBRANE] Step too short ({} chars): '{}'", step.len(), step);
            return ToolSignal::Veto(format!("Step too short ({} chars). Write a real mathematical reasoning step.", step.len()));
        }

        // [COMPLETE] → NOT auto-OMEGA. Mark as pending, let market price it.
        // DeepSeek verification only triggers when price >= 90% (architect directive 2026-04-02).
        if step.contains("[COMPLETE]") {
            info!(">>> [MEMBRANE] COMPLETE declared by {}. Pending market validation (need P >= 90%).", _author);
            // Pass through — reactor loop will check price threshold and call DeepSeek
        }

        // Everything else passes — market handles quality
        ToolSignal::Pass
    }
}
