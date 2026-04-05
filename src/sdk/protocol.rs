/// TuringOS Agent Output Protocol
///
/// Universal structured output parsing for all agent types.
/// Replaces regex-based distill_pure_state with JSON <action> protocol.
///
/// LLM outputs end with:
///   <action>{"tool":"invest","tactic":"simp","amount":50.0}</action>
///   <action>{"tool":"search","query":"riemannZeta"}</action>
///   <action>{"tool":"invest","tactic":"rw [h]","amount":10.0,"node":"step_3_branch_1"}</action>

use serde_json::Value;

#[derive(Debug, Clone)]
pub struct AgentAction {
    pub tool: String,              // "invest" | "search" | "observe"
    pub tactic: Option<String>,    // Lean 4 tactic block (for invest)
    pub amount: Option<f64>,       // Investment amount (for invest)
    pub node: Option<String>,      // "self" or node_id (for invest)
    pub query: Option<String>,     // Search query (for search)
}

/// Parse the last <action>{...}</action> block from raw LLM output.
/// Falls back to legacy [Tactic: ...] [Tool: Wallet ...] format for backward compat.
pub fn parse_agent_output(raw: &str) -> Option<AgentAction> {
    // Try JSON <action> protocol first
    if let Some(action) = parse_action_json(raw) {
        return Some(action);
    }

    // Fallback: legacy [Tactic: ...] + [Tool: Wallet ...] format
    parse_legacy_format(raw)
}

fn parse_action_json(raw: &str) -> Option<AgentAction> {
    let tag_open = "<action>";
    let tag_close = "</action>";

    // Find the LAST <action> block (LLM may output multiple during reasoning)
    let start = raw.rfind(tag_open)?;
    let json_start = start + tag_open.len();
    let end = raw[json_start..].find(tag_close)?;
    let inner = raw[json_start..json_start + end].trim();

    // LLMs sometimes prefix JSON with the tool name: <action>append: {...}</action>
    // Find the first '{' to extract just the JSON object.
    let json_str = if let Some(brace) = inner.find('{') {
        &inner[brace..]
    } else {
        inner
    };

    let v: Value = serde_json::from_str(json_str).ok()?;

    let tool = v.get("tool")?.as_str()?.to_string();
    let tactic = v.get("tactic").and_then(|t| t.as_str()).map(|s| s.replace("\\n", "\n"));
    let amount = v.get("amount").and_then(|a| a.as_f64());
    let node = v.get("node").and_then(|n| n.as_str()).map(|s| s.to_string());
    let query = v.get("query").and_then(|q| q.as_str()).map(|s| s.to_string());

    Some(AgentAction { tool, tactic, amount, node, query })
}

fn parse_legacy_format(raw: &str) -> Option<AgentAction> {
    // Legacy: extract [Tactic: ...] and [Tool: Wallet | Action: Invest/Stake ...]
    use crate::sdk::membrane::distill_pure_state;

    let pure = distill_pure_state(raw)?;

    // Extract tactic content
    let mut tactic_str = pure.clone();
    let mut tool_call = String::new();

    // Separate wallet tool call
    if let Some(idx) = tactic_str.find("[Tool: Wallet") {
        tool_call = tactic_str[idx..].to_string();
        tactic_str = tactic_str[..idx].trim().to_string();
    }

    // Strip [Tactic: ...] wrapper
    if tactic_str.starts_with("[Tactic:") && tactic_str.ends_with("]") {
        tactic_str = tactic_str[8..tactic_str.len()-1].trim().to_string();
        tactic_str = tactic_str.replace("\\n", "\n");
    }
    // Strip [State: ...] wrapper
    if tactic_str.starts_with("[State:") && tactic_str.ends_with("]") {
        tactic_str = tactic_str[7..tactic_str.len()-1].trim().to_string();
    }

    // Check if this is a VC investment (State: INVEST)
    if tactic_str.to_uppercase().contains("INVEST") && !tool_call.is_empty() {
        // Parse wallet for VC invest
        let (node, amount) = parse_wallet_tag(&tool_call)?;
        return Some(AgentAction {
            tool: "invest".to_string(),
            tactic: None,
            amount: Some(amount),
            node: Some(node),
            query: None,
        });
    }

    // Check for research/search tool
    if let Some(idx) = raw.rfind("[Tool: MathlibOracle | Query: ") {
        let rest = &raw[idx + 30..];
        if let Some(end) = rest.find(']') {
            return Some(AgentAction {
                tool: "search".to_string(),
                tactic: None,
                amount: None,
                node: None,
                query: Some(rest[..end].trim().to_string()),
            });
        }
    }

    // Mining with wallet
    if !tool_call.is_empty() {
        let (node, amount) = parse_wallet_tag(&tool_call)?;
        return Some(AgentAction {
            tool: "invest".to_string(),
            tactic: Some(tactic_str),
            amount: Some(amount),
            node: Some(node),
            query: None,
        });
    }

    // No wallet = observation round (free, no Tape write)
    Some(AgentAction {
        tool: "observe".to_string(),
        tactic: if tactic_str.is_empty() { None } else { Some(tactic_str) },
        amount: None,
        node: None,
        query: None,
    })
}

fn parse_wallet_tag(tool_call: &str) -> Option<(String, f64)> {
    // Parse [Tool: Wallet | Action: Invest/Stake | Node: X | Amount: Y]
    let invest_tag = "Action: Invest | Node: ";
    let stake_tag = "Action: Stake | Node: ";
    let tag = if tool_call.contains(invest_tag) { invest_tag } else { stake_tag };

    let start = tool_call.find(tag)?;
    let rest = &tool_call[start + tag.len()..];
    let node_end = rest.find(" | Amount: ")?;
    let node = rest[..node_end].trim().to_string();
    let amt_rest = &rest[node_end + 11..];
    let amt_end = amt_rest.find(']')?;
    let amount: f64 = amt_rest[..amt_end].trim().parse().ok()?;

    Some((node, amount))
}
