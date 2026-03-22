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

        // Empty step → VETO (investment burned)
        if step.is_empty() {
            warn!(">>> [MEMBRANE] Empty step rejected.");
            return ToolSignal::Veto("Empty mathematical step.".into());
        }

        // [COMPLETE] → OMEGA signal (terminal oracle will verify)
        if step.contains("[COMPLETE]") {
            info!(">>> [MEMBRANE] COMPLETE declared! Triggering terminal oracle.");
            return ToolSignal::YieldReward {
                payload: format!("{}\n  -- [OMEGA]", payload),
                reward: 100_000_000_000.0,
            };
        }

        // Everything else passes — market handles quality
        ToolSignal::Pass
    }
}
