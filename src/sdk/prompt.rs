/// TuringOS Universal Prompt Builder
///
/// Minimal prompt template for any domain. The OS provides state;
/// the LLM provides intelligence. No rules explanation, no role-playing.
/// "Gravity doesn't explain itself to apples."

pub fn build_agent_prompt(
    proof_state: &str,
    skill: &str,
    market_ticker: &str,
    graveyard: &str,
    balance: f64,
    tools_description: &str,
) -> String {
    format!(
        "{proof_state}\n\n\
         {skill}\n\
         {market_ticker}\n\
         {graveyard}\n\
         {tools_description}\n\
         [BALANCE: {balance:.2}]\n\n\
         Respond with <action>{{...}}</action> at the end.\n\
         You may think freely before the action block.",
    )
}

/// Default tools description for Lean 4 formal verification
pub fn lean4_tools() -> &'static str {
    r#"Available tools (output one inside <action> at the end):
  invest: {"tool":"invest","tactic":"your lean4 tactic","amount":YOUR_PRICE}
  invest in node: {"tool":"invest","node":"NODE_ID","amount":YOUR_PRICE}
  search: {"tool":"search","query":"search term"} (FREE, results next round)"#
}
