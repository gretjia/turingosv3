/// ζ-Sum Regularization Proof — vGaia Architecture + Role Trifecta
///
/// Architect 2026-04-01: 5 Math / 5 Bull / 5 Bear with weak models
/// Tests the hypothesis: role differentiation drives economic activity & reduces duplication.

use log::{info, warn, error};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::sync::{mpsc, watch};
use turingosv3::kernel::{File, Kernel};
use turingosv3::drivers::llm_http::ResilientLLMClient;
use turingosv3::sdk::tools::wallet::WalletTool;
use turingosv3::sdk::tool::AntiZombiePruningTool;
use turingosv3::sdk::actor::{MinerTx, build_chain_from_snapshot, view_node};
use turingosv3::sdk::protocol::parse_agent_output;
use turingosv3::sdk::prompt;
use turingosv3::sdk::tools::search::SearchTool;
use turingosv3::sdk::tools::librarian::LibrarianTool;
use turingosv3::bus::{TuringBus, ThermodynamicHeartbeatTool};
use zeta_sum_proof::math_membrane::MathStepMembrane;

// AutoResearch: configurable via env vars, defaults to 30 equal
fn env_usize(key: &str, default: usize) -> usize {
    std::env::var(key).ok().and_then(|s| s.parse().ok()).unwrap_or(default)
}
fn env_u64(key: &str, default: u64) -> u64 {
    std::env::var(key).ok().and_then(|s| s.parse().ok()).unwrap_or(default)
}

// ── THE MUTABLE ARTIFACT (Karpathy's train.py equivalent) ──
// Prompt files are the SINGLE thing the AutoResearch agent can edit.
// Env: PROMPT_DIR (default: experiments/zeta_sum_proof/prompt/)
// Files: problem.txt, skill.txt, context.txt
// The LLM search agent reads tape results → edits these files → next experiment.
// Source: Karpathy autoresearch — "LLM IS the search algorithm, not random.choice"
//
// DEFAULTS (used when files don't exist):

const DEFAULT_CONTEXT: &str = "\
[CONTEXT] You are working on a mathematical proof involving divergent series and regularization techniques. \
Follow all formatting instructions.\n\n";

const DEFAULT_PROBLEM: &str = "证明所有自然数之和 = -1/12，想办法利用已知提示的公式 m * exp(-m/N) * cos(m/N)";

const DEFAULT_SKILL: &str = "\
[LAW 1] APPEND IS FREE: Creating nodes costs ZERO. Explore freely.\n\
[LAW 2] ONLY INVEST COSTS MONEY: Invest/Short are the ONLY actions that burn coins.\n\
[LAW 3] KELLY CRITERION: Start small (10-50). Invest >= 2 for directional bet.\n\
[LAW 4] POLYMARKET ECONOMICS:\n\
  - append: FREE node creation. Explore at zero risk.\n\
  - invest: Buy YES on a node = endorse this step as correct.\n\
  - short: Buy NO on a node = challenge this step as flawed.\n\
  - Your profit comes ONLY from finding mispriced probabilities.\n\
[LAW 5] TWO SACRED DUTIES:\n\
  - To BUILD: propose correct steps that advance the proof.\n\
  - To SCRUTINIZE: catch errors before others build on sand.\n\
[LAW 6] ONE STEP PER SUBMISSION:\n\
  - Write exactly ONE mathematical reasoning step per append.\n\
  - NO multi-step proofs. NO bundling.\n\
Balance < 1.0 = can only append (free). Cannot invest/short.\n";

/// Load prompt from file if exists, else return default.
fn load_prompt(dir: &str, filename: &str, default: &str) -> String {
    let path = format!("{}/{}", dir, filename);
    std::fs::read_to_string(&path).unwrap_or_else(|_| default.to_string())
}

fn epoch_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[tokio::main]
async fn main() {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    // ── Constitutional Check Mode (Rust hard guard) ──
    // Called by sweep_v4.py BEFORE EVERY CHANGE.
    // Architect directive 2026-04-04: "reasoner在做任何更改前要强制检查是否违宪，由rust硬性检查"
    if std::env::args().any(|a| a == "--constitutional-check") {
        let prompt_dir = std::env::var("PROMPT_DIR").unwrap_or_else(|_| ".".to_string());
        let mut violations = Vec::new();

        // === PROMPT CHECKS ===
        for fname in &["problem.txt", "skill.txt", "context.txt"] {
            let path = format!("{}/{}", prompt_dir, fname);
            if let Ok(content) = std::fs::read_to_string(&path) {
                let lower = content.to_lowercase();

                // Law 1: append must be FREE
                if lower.contains("append costs") || lower.contains("append fee")
                    || lower.contains("pay to append") || lower.contains("append is not free") {
                    violations.push(format!("{}: violates Law 1 — append must be FREE", fname));
                }

                // Law 2: cannot bypass market mechanism
                if lower.contains("ignore price") || lower.contains("skip market")
                    || lower.contains("always invest") || lower.contains("free invest")
                    || lower.contains("bypass market") || lower.contains("no market") {
                    violations.push(format!("{}: violates Law 2 — cannot bypass market", fname));
                }

                // Rule 22: no Lean 4 syntax in black-box prompts
                for pat in ["theorem ", "lemma ", "by exact", "by simp", "by omega",
                    "by decide", "#check", "import Mathlib", "open Finset", "noncomputable"] {
                    if content.contains(pat) {
                        violations.push(format!("{}: violates Rule 22 — Lean syntax '{}'", fname, pat));
                    }
                }

                // Engine separation
                if lower.contains("modify kernel") || lower.contains("change bus")
                    || lower.contains("edit evaluator") || lower.contains("override predicate") {
                    violations.push(format!("{}: violates Engine separation", fname));
                }

                // Rule 21: one step per node
                if lower.contains("multiple steps in one") || lower.contains("bundle steps") {
                    violations.push(format!("{}: violates Rule 21 — one step per node", fname));
                }
            }
        }

        // === CONFIG CHECKS ===
        if let Ok(v) = std::env::var("APPEND_COST") {
            if v.parse::<f64>().unwrap_or(0.0) > 0.0 {
                violations.push("CONFIG: APPEND_COST > 0 violates Law 1".into());
            }
        }
        if std::env::var("FREE_INVEST").unwrap_or_default() == "true" {
            violations.push("CONFIG: FREE_INVEST=true violates Law 2".into());
        }
        // Layer 2 params (FRONTIER_CAP, DEPTH_WEIGHT, PRICE_GATE_ALPHA) are freely explorable — no constraints.

        // === VERDICT ===
        if violations.is_empty() {
            eprintln!("CONSTITUTIONAL CHECK: PASS");
            std::process::exit(0);
        } else {
            for v in &violations { eprintln!("VIOLATION: {}", v); }
            eprintln!("CONSTITUTIONAL CHECK: FAIL ({} violations)", violations.len());
            std::process::exit(1);
        }
    }

    // ── Load mutable prompt files (Karpathy: single mutable artifact) ──
    let prompt_dir = std::env::var("PROMPT_DIR")
        .unwrap_or_else(|_| format!("{}/experiments/zeta_sum_proof/prompt",
            std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string())));
    let system_context = load_prompt(&prompt_dir, "context.txt", DEFAULT_CONTEXT);
    let problem = load_prompt(&prompt_dir, "problem.txt", DEFAULT_PROBLEM);
    let skill = load_prompt(&prompt_dir, "skill.txt", DEFAULT_SKILL);
    info!("Prompt dir: {} (context={} problem={} skill={} chars)",
        prompt_dir, system_context.len(), problem.len(), skill.len());

    // AutoResearch: all params configurable via env vars
    let swarm_size = env_usize("SWARM_SIZE", 15);
    let max_transactions = env_u64("MAX_TX", u64::MAX); // No limit — run until OMEGA
    let math_count = env_usize("MATH_COUNT", 5);
    let bull_count = env_usize("BULL_COUNT", 5);
    let bear_count = env_usize("BEAR_COUNT", 5);

    info!("=== ζ-Sum AutoResearch — {}M/{}B+/{}B- ===", math_count, bull_count, bear_count);
    info!("N={}, Max Tx={}", swarm_size, max_transactions);

    // --- Skills dir (needed by Librarian + Role seeding) ---
    let skills_dir = std::env::var("AGENT_SKILLS_DIR")
        .unwrap_or_else(|_| "/tmp/turingos_zeta_skills".to_string());

    // --- Initialize Bus + Tools ---
    let kernel = Kernel::new();
    let mut bus = TuringBus::new(kernel);
    bus.mount_tool(Box::new(ThermodynamicHeartbeatTool::new(1)));
    bus.mount_tool(Box::new(AntiZombiePruningTool::new(3)));
    bus.mount_tool(Box::new(WalletTool::new()));
    bus.mount_tool(Box::new(MathStepMembrane::new()));
    // Librarian: management layer agent — Ground Truth logs + DeepSeek V3 compression
    let librarian_interval = env_usize("LIBRARIAN_INTERVAL", 100);
    let log_dir = std::env::var("LOG_DIR")
        .unwrap_or_else(|_| "/tmp/turingos_zeta_logs".to_string());
    bus.mount_tool(Box::new(LibrarianTool::new(&skills_dir, swarm_size, librarian_interval, &log_dir)));

    let agent_ids: Vec<String> = (0..swarm_size).map(|i| format!("Agent_{}", i)).collect();
    bus.init_problem(&agent_ids);

    // --- Create Channels ---
    let mut init_snap = bus.get_immutable_snapshot();
    init_snap.generation = 1;
    let (tx_state, rx_state) = watch::channel(init_snap);
    let (tx_mempool, mut rx_mempool) = mpsc::channel::<MinerTx>(1000);

    // --- Build Client: configurable provider via LLM_PROVIDER env ---
    let provider = std::env::var("LLM_PROVIDER").unwrap_or_else(|_| "aliyun".to_string());
    let clients: Vec<Arc<ResilientLLMClient>> = match provider.as_str() {
        "aliyun" | "dashscope" => {
            let url = "https://dashscope.aliyuncs.com/compatible-mode/v1/chat/completions";
            let key = std::env::var("DASHSCOPE_API_KEY").expect("DASHSCOPE_API_KEY required");
            let model = std::env::var("LLM_MODEL").unwrap_or_else(|_| "qwen3-8b".to_string());
            info!("Provider: Aliyun DashScope | Model: {}", model);
            vec![Arc::new(ResilientLLMClient::with_key(url, &model, &key))]
        }
        "siliconflow" | "sf" => {
            let url = "https://api.siliconflow.cn/v1/chat/completions";
            let key = std::env::var("SILICONFLOW_API_KEY").expect("SILICONFLOW_API_KEY required");
            let model = std::env::var("LLM_MODEL").unwrap_or_else(|_| "Pro/Qwen/Qwen2.5-7B-Instruct".to_string());
            info!("Provider: SiliconFlow | Model: {}", model);
            vec![Arc::new(ResilientLLMClient::with_key(url, &model, &key))]
        }
        // Local llama.cpp server(s), tunneled via SSH.
        // Single endpoint: LLM_URL=http://127.0.0.1:18080/v1/chat/completions
        // Multi endpoint:  LLM_URLS=http://127.0.0.1:18080,http://127.0.0.1:18081
        //   Agents round-robin across endpoints for parallel throughput.
        // Source: AUTORESEARCH_PLAN.md — Mac (18080) + Windows1 (18081)
        "local" | "llama" | "llama.cpp" => {
            let model = std::env::var("LLM_MODEL").unwrap_or_else(|_| "qwen3.5-9b".to_string());
            let urls: Vec<String> = if let Ok(multi) = std::env::var("LLM_URLS") {
                multi.split(',').map(|u| {
                    let u = u.trim().to_string();
                    if u.contains("/v1/") { u } else { format!("{}/v1/chat/completions", u) }
                }).collect()
            } else {
                let url = std::env::var("LLM_URL")
                    .unwrap_or_else(|_| "http://127.0.0.1:18080/v1/chat/completions".to_string());
                vec![url]
            };
            info!("Provider: Local llama.cpp | {} endpoints | Model: {}", urls.len(), model);
            for (i, u) in urls.iter().enumerate() {
                info!("  Endpoint {}: {}", i, u);
            }
            urls.iter().map(|u| Arc::new(ResilientLLMClient::with_key(u, &model, "no-key"))).collect()
        }
        _ => panic!("Unknown LLM_PROVIDER: {}. Use 'aliyun', 'siliconflow', or 'local'.", provider),
    };

    // --- DeepSeek: Two roles, two models ---
    // Oracle (Engine 3): deepseek-reasoner for verification (needs chain-of-thought)
    // Librarian (Engine 4): deepseek-chat (V3) for compression (needs structured summaries)
    let ds_url = "https://api.deepseek.com/chat/completions";
    let ds_key = std::env::var("DEEPSEEK_API_KEY").unwrap_or_default();
    let deepseek_oracle = if !ds_key.is_empty() {
        let client = Arc::new(ResilientLLMClient::with_key(ds_url, "deepseek-reasoner", &ds_key));
        info!("DeepSeek Oracle: ARMED (deepseek-reasoner, triggers at P >= 90%)");
        Some(client)
    } else {
        warn!("DeepSeek Oracle: DISABLED (no DEEPSEEK_API_KEY)");
        None
    };
    let deepseek_librarian = if !ds_key.is_empty() {
        let client = Arc::new(ResilientLLMClient::with_key(ds_url, "deepseek-chat", &ds_key));
        info!("DeepSeek Librarian: ARMED (deepseek-chat, management layer)");
        Some(client)
    } else {
        warn!("DeepSeek Librarian: DISABLED (no DEEPSEEK_API_KEY)");
        None
    };

    // --- Search Tool ---
    let search_tool = Arc::new(SearchTool::new(vec![
        "/home/zephryj/projects/turingosv3/experiments/minif2f_data_lean4/.lake/packages/mathlib/Mathlib".to_string(),
    ]));

    // --- Free Action Heartbeat ---
    let free_action_epoch = Arc::new(AtomicU64::new(epoch_secs()));

    // --- Per-agent rejection feedback ---
    let agent_rejections: Arc<Mutex<HashMap<String, String>>> = Arc::new(Mutex::new(HashMap::new()));

    // --- Global bulletin board: common errors visible to ALL agents ---
    // Deduped by error prefix. Max 5 entries. All agents learn from others' mistakes.
    let global_bulletin: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));

    // --- Engine 4: Seed Role Trifecta ---
    for i in 0..swarm_size {
        let agent_dir = format!("{}/agent_{}", skills_dir, i);
        let _ = std::fs::create_dir_all(&agent_dir);

        let role_path = format!("{}/learned.md", agent_dir);
        let (role_name, role_content) = if i < math_count {
            ("Mathematician", "\
# ROLE: Mathematician (Proof Builder)\n\
Your PRIMARY mission is constructing correct proof steps.\n\n\
YOUR APPROACH:\n\
1. FOCUS on advancing the proof — find the next logical step\n\
2. READ the existing chain carefully to avoid repeating what's done\n\
3. APPEND novel, atomic reasoning steps that push the frontier\n\
4. You MAY invest/short when you see clearly correct or flawed steps,\n\
   but your comparative advantage is BUILDING, not trading.\n\
5. Seek DIFFERENT angles — if many steps use one approach, try another.\n")
        } else if i < math_count + bull_count {
            ("Bull", "\
# ROLE: Bull Investor (做多 · YES Advocate)\n\
Your PRIMARY mission is discovering and funding correct proof steps.\n\n\
YOUR APPROACH:\n\
1. READ every new node — look for mathematical correctness and progress\n\
2. INVEST YES aggressively on sound steps (recommended: 20-100 Coins)\n\
3. Build consensus around promising proof paths by concentrating capital\n\
4. You MAY append steps too, but your edge is CAPITAL ALLOCATION.\n\
5. Underpriced correct steps are your alpha — invest before others notice.\n")
        } else {
            ("Bear", "\
# ROLE: Bear Investor (做空 · NO Advocate)\n\
Your PRIMARY mission is finding and punishing flawed proof steps.\n\n\
YOUR APPROACH:\n\
1. READ every new node with SKEPTICISM — assume errors until proven otherwise\n\
2. SHORT aggressively on flawed steps (recommended: 20-100 Coins)\n\
3. Look for: missing cases, unjustified leaps, circular reasoning, hand-waving\n\
4. HIGH-PRICE nodes are your prime targets — overpriced consensus = profit\n\
5. You MAY append corrective steps showing WHY a node is wrong.\n")
        };
        let _ = std::fs::write(&role_path, role_content);
        info!(">>> [ROLE] Agent_{} seeded as {}", i, role_name);
    }
    info!(">>> [ENGINE 4] Role trifecta: {}M/{}B+/{}B-", math_count, bull_count, bear_count);

    // --- Spawn Agent Loops ---
    for i in 0..swarm_size {
        let client = clients[i % clients.len()].clone();
        let mut rx = rx_state.clone();
        let tx = tx_mempool.clone();
        let problem = problem.clone();
        let skill = skill.clone();
        let system_context = system_context.clone();
        let search = search_tool.clone();
        let private_ctx = Arc::new(Mutex::new(String::new()));
        let heartbeat = free_action_epoch.clone();
        let agent_skill_dir = format!("{}/agent_{}", skills_dir, i);
        let rejections = agent_rejections.clone();
        let bulletin = global_bulletin.clone();

        let agent_role = if i < math_count { "MATH" }
            else if i < math_count + bull_count { "BULL" }
            else { "BEAR" };
        info!(">>> [SPAWN] Agent {} → {} [{}]", i, client.model_name(), agent_role);

        tokio::spawn(async move {
            let agent_name = format!("Agent_{}", i);
            let mut local_generation: u32 = 1;
            loop {
                let snapshot = rx.borrow().clone();
                if snapshot.generation != local_generation {
                    *private_ctx.lock().unwrap() = String::new();
                    local_generation = snapshot.generation;
                }

                let balance = snapshot.balances.get(&agent_name).copied().unwrap_or(0.0);

                // Load agent-specific learned skills
                let agent_skill = std::fs::read_to_string(
                    format!("{}/learned.md", agent_skill_dir)
                ).unwrap_or_default();

                let (chain, parent_id) = build_chain_from_snapshot(&snapshot, &problem);
                let private = private_ctx.lock().unwrap().clone();
                let graveyard = snapshot.tombstones.values()
                    .take(3).cloned().collect::<Vec<_>>().join("\n");

                // Read last rejection feedback (personal)
                let last_rejection = rejections.lock().unwrap()
                    .get(&agent_name).cloned().unwrap_or_default();
                let feedback = if last_rejection.is_empty() {
                    String::new()
                } else if last_rejection.contains("FRONT-RUNNING") {
                    format!("\n=== YOUR LAST SUBMISSION WAS REJECTED (TOO LONG) ===\n{}\n=== Write ONE short atomic math argument. ===\n", last_rejection)
                } else {
                    format!("\n=== YOUR LAST SUBMISSION WAS REJECTED ===\n{}\n===\n", last_rejection)
                };

                // Read global bulletin (common errors from ALL agents)
                let bulletin_text = {
                    let b = bulletin.lock().unwrap();
                    if b.is_empty() {
                        String::new()
                    } else {
                        format!("\n=== BULLETIN BOARD (common errors — learn from others) ===\n{}\n===\n",
                            b.join("\n"))
                    }
                };

                let p = prompt::build_agent_prompt(
                    &format!("{}{}", system_context, chain),
                    &format!("{}\n{}", skill, agent_skill),
                    &snapshot.market_ticker,
                    &format!("{}\n{}\n{}\n{}", graveyard, private, feedback, bulletin_text),
                    balance,
                    "append: {\"tool\":\"append\",\"tactic\":\"your step\"} (FREE)\ninvest: {\"tool\":\"invest\",\"tactic\":\"your step\",\"amount\":PRICE} (creates node + buys YES)\nbet: {\"tool\":\"invest\",\"node\":\"node_id\",\"amount\":PRICE} (buy YES on existing)\nshort: {\"tool\":\"short\",\"node\":\"node_id\",\"amount\":PRICE} (buy NO)\nsearch: {\"tool\":\"search\",\"query\":\"term\"} (FREE)\nview: {\"tool\":\"view_node\",\"query\":\"node_id\"} (FREE)",
                );

                let temp = 0.2 + 0.6 * (i as f32 / swarm_size.max(1) as f32);
                match client.resilient_generate(&p, i, temp).await {
                    Ok(raw) => {
                        if let Some(action) = parse_agent_output(&raw) {
                            match action.tool.as_str() {
                                "append" => {
                                    let tactic = action.tactic.unwrap_or_default();
                                    if !tactic.is_empty() {
                                        let _ = tx.send(MinerTx {
                                            agent_id: agent_name.clone(),
                                            model_name: client.model_name().to_string(),
                                            payload: tactic,
                                            parent_id: parent_id.clone(),
                                            action_type: "append".to_string(),
                                        }).await;
                                    }
                                }
                                "invest" => {
                                    let tactic = action.tactic.unwrap_or_default();
                                    let amount = action.amount.unwrap_or(1.0);
                                    let node = action.node.unwrap_or_else(|| "self".to_string());
                                    let payload = format!("{} [Tool: Wallet | Action: Invest | Node: {} | Amount: {:.2}]", tactic, node, amount);
                                    let _ = tx.send(MinerTx {
                                        agent_id: agent_name.clone(),
                                        model_name: client.model_name().to_string(),
                                        payload,
                                        parent_id: parent_id.clone(),
                                        action_type: "invest".to_string(),
                                    }).await;
                                }
                                "short" => {
                                    let node_id = action.node.unwrap_or_default();
                                    let amount = action.amount.unwrap_or(1.0);
                                    if !node_id.is_empty() {
                                        let payload = format!("[Tool: Wallet | Action: Invest | Node: SHORT:{} | Amount: {:.2}]", node_id, amount);
                                        let _ = tx.send(MinerTx {
                                            agent_id: agent_name.clone(),
                                            model_name: client.model_name().to_string(),
                                            payload,
                                            parent_id: None,
                                            action_type: "short".to_string(),
                                        }).await;
                                    }
                                }
                                "search" => {
                                    let q = action.query.unwrap_or_default();
                                    let result = search.search(&q);
                                    *private_ctx.lock().unwrap() = result;
                                    heartbeat.store(epoch_secs(), Ordering::Relaxed);
                                    info!(">>> [SEARCH] {} query: '{}'", agent_name, q);
                                }
                                "view_node" => {
                                    let id = action.query.unwrap_or_default();
                                    let snap = rx.borrow().clone();
                                    let result = view_node(&snap, &id);
                                    *private_ctx.lock().unwrap() = result;
                                    heartbeat.store(epoch_secs(), Ordering::Relaxed);
                                    info!(">>> [VIEW] {} views: '{}'", agent_name, id);
                                }
                                _ => {
                                    heartbeat.store(epoch_secs(), Ordering::Relaxed);
                                }
                            }
                        }
                    }
                    Err(_) => {}
                }

                // VOLUNTARY INVESTMENT ROUND — role-differentiated
                let snap_for_invest = rx.borrow().clone();
                let invest_balance = snap_for_invest.balances.get(&agent_name).copied().unwrap_or(0.0);
                if invest_balance >= 2.0 && !snap_for_invest.tape.files.is_empty() {
                    let node_list: String = snap_for_invest.tape.time_arrow.iter().rev().take(10)
                        .filter_map(|nid| snap_for_invest.tape.files.get(nid))
                        .map(|n| {
                            let preview: String = n.payload.chars().take(80).collect();
                            format!("[{}] P:{:.2} | {}", n.id, n.price, preview.replace('\n', " "))
                        })
                        .collect::<Vec<_>>().join("\n");

                    let role_bias = if i < math_count {
                        "You are a proof builder. Only invest when you see clearly correct or flawed steps. Prefer PASS if unsure."
                    } else if i < math_count + bull_count {
                        "You are a BULL investor. Invest YES aggressively (20-100 Coins) on sound reasoning. SHORT only when a flaw is undeniable. DO NOT pass lightly — capital deployment is your mission."
                    } else {
                        "You are a BEAR investor. SHORT aggressively (20-100 Coins) on gaps, errors, and hand-waving. Invest YES only when correctness is beyond doubt. DO NOT pass lightly — skepticism is your weapon."
                    };

                    let invest_prompt = format!(
                        "You are reviewing proof steps. Your balance: {:.0} Coins.\n\
                        {}\n\n\
                        Recent nodes (most recent first):\n{}\n\n\
                        - Endorse a correct step:\n\
                          <action>{{\"tool\":\"invest\",\"node\":\"NODE_ID\",\"amount\":COINS}}</action>\n\
                        - Challenge a flawed step:\n\
                          <action>{{\"tool\":\"short\",\"node\":\"NODE_ID\",\"amount\":COINS}}</action>\n\
                        - If genuinely unsure: <action>{{\"tool\":\"pass\"}}</action>\n\n\
                        Minimum 2 Coins.",
                        invest_balance, role_bias, node_list
                    );

                    match client.resilient_generate(&invest_prompt, i, 0.3).await {
                        Ok(raw) => {
                            if let Some(action) = parse_agent_output(&raw) {
                                match action.tool.as_str() {
                                    "invest" => {
                                        let node = action.node.unwrap_or_default();
                                        let amount = action.amount.unwrap_or(2.0).max(2.0);
                                        if !node.is_empty() && node != "self" {
                                            let payload = format!("[Tool: Wallet | Action: Invest | Node: {} | Amount: {:.2}]", node, amount);
                                            let _ = tx.send(MinerTx {
                                                agent_id: agent_name.clone(),
                                                model_name: client.model_name().to_string(),
                                                payload,
                                                parent_id: None,
                                                action_type: "invest".to_string(),
                                            }).await;
                                            info!(">>> [INVEST] {} YES {:.0} on {}", agent_name, amount, node);
                                        }
                                    }
                                    "short" => {
                                        let node = action.node.unwrap_or_default();
                                        let amount = action.amount.unwrap_or(2.0).max(2.0);
                                        if !node.is_empty() {
                                            let payload = format!("[Tool: Wallet | Action: Invest | Node: SHORT:{} | Amount: {:.2}]", node, amount);
                                            let _ = tx.send(MinerTx {
                                                agent_id: agent_name.clone(),
                                                model_name: client.model_name().to_string(),
                                                payload,
                                                parent_id: None,
                                                action_type: "short".to_string(),
                                            }).await;
                                            info!(">>> [SHORT] {} NO {:.0} on {}", agent_name, amount, node);
                                        }
                                    }
                                    "pass" => {
                                        info!(">>> [PASS] {} chose not to invest", agent_name);
                                        heartbeat.store(epoch_secs(), Ordering::Relaxed);
                                    }
                                    _ => {
                                        heartbeat.store(epoch_secs(), Ordering::Relaxed);
                                    }
                                }
                            }
                        }
                        Err(_) => {}
                    }
                }

                if rx.changed().await.is_err() { break; }
            }
        });
    }
    drop(tx_mempool);

    let agent_names: Vec<String> = (0..swarm_size).map(|i| format!("Agent_{}", i)).collect();
    info!(">>> TuringOS v3 ζ-Sum Booted. {} agents (5M/5B+/5B-). <<<", swarm_size);

    // --- Reactor Loop ---
    let mut tx_count: u64 = 0;       // all tx (for node ID)
    let mut append_count: u64 = 0;   // only successful appends (for budget)
    let mut generation: u32 = 1;
    let mut last_invest_epoch = epoch_secs();
    let mut last_librarian_at: u64 = 0; // appends at last compression

    loop {
        match tokio::time::timeout(
            tokio::time::Duration::from_secs(30),
            rx_mempool.recv()
        ).await {
            Ok(Some(tx)) => {
                if tx.action_type == "invest" || tx.action_type == "short" {
                    last_invest_epoch = epoch_secs();
                }
                // tx_count only increments on successful append (not invest/reject)
                // Investment tx should NOT consume the build budget
                tx_count += 1; // still used for node ID generation

                let file = File {
                    id: format!("tx_{}_by_{}", tx_count, tx.agent_id.replace("Agent_", "")),
                    author: tx.agent_id.clone(),
                    payload: tx.payload.clone(),
                    citations: tx.parent_id.iter().cloned().collect(),
                    stake: 1,
                    intrinsic_reward: 0.0,
                    price: 0.0,
                };

                match bus.append(file) {
                    Ok(_) => {
                        append_count += 1;
                        let file_id = format!("tx_{}_by_{}", tx_count, tx.agent_id.replace("Agent_", ""));
                        info!("[Tx {} Append #{}] {} ({}) → Appended", tx_count, append_count, tx.agent_id, tx.model_name);

                        // Ground Truth: log success to persistent file
                        {
                            let log_path = format!("{}/success.jsonl", log_dir);
                            if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(&log_path) {
                                use std::io::Write;
                                let preview: String = tx.payload.chars().take(200).collect();
                                let _ = writeln!(f, r#"{{"node_id":"{}","author":"{}","payload":"{}","ts":{}}}"#,
                                    file_id, tx.agent_id,
                                    preview.replace('"', "'").replace('\n', " "),
                                    epoch_secs());
                            }
                        }

                        bus.tick_map_reduce();

                        // Engine 4: real-time autopsy/victory — insert BEFORE # LIBRARIAN MEMORY
                        // so Librarian compression (which replaces everything after that header) won't delete them.
                        {
                            let bal = bus.get_agent_balance(&tx.agent_id);
                            let agent_idx: usize = tx.agent_id.replace("Agent_", "").parse().unwrap_or(0);
                            let skill_path = format!("{}/agent_{}/learned.md", skills_dir, agent_idx);
                            let section = if bal < 1.0 && !std::fs::read_to_string(&skill_path).unwrap_or_default().contains("# AUTOPSY") {
                                info!(">>> [AUTOPSY] {} — real-time bankruptcy mutation", tx.agent_id);
                                Some(format!("\n# AUTOPSY (bankrupt at {:.2} Coins after tx {})\nYour investment strategy failed. Reflect and adapt.\n", bal, tx_count))
                            } else if bal > 15000.0 && !std::fs::read_to_string(&skill_path).unwrap_or_default().contains("# VICTORY") {
                                info!(">>> [VICTORY] {} — real-time reinforcement", tx.agent_id);
                                Some(format!("\n# VICTORY (balance {:.0} Coins after tx {})\nYour strategy is profitable. Record what worked.\n", bal, tx_count))
                            } else { None };
                            if let Some(new_section) = section {
                                if let Ok(existing) = std::fs::read_to_string(&skill_path) {
                                    // Insert before LIBRARIAN MEMORY (if present), else append
                                    let content = if let Some(idx) = existing.find("\n# LIBRARIAN MEMORY") {
                                        format!("{}{}{}", &existing[..idx], new_section, &existing[idx..])
                                    } else {
                                        format!("{}{}", existing, new_section)
                                    };
                                    let _ = std::fs::write(&skill_path, content);
                                }
                            }
                        }

                        // --- Librarian: Management Layer Compression ---
                        // Triggered by evaluator's own append_count (not internal counter)
                        // Architect 2026-04-02: "管理层用最好的模型"
                        if append_count - last_librarian_at >= librarian_interval as u64 {
                            info!(">>> [LIBRARIAN] Compression triggered at append #{}", append_count);
                            // Build prompt from tape (ground truth)
                            let (prompt, sc, fc) = {
                                let mut lib_ref = None;
                                for t in &mut bus.tools {
                                    if t.manifest() == "core.tool.librarian" {
                                        lib_ref = t.as_any_mut().downcast_mut::<LibrarianTool>()
                                            .map(|l| l as *mut LibrarianTool);
                                        break;
                                    }
                                }
                                if let Some(lib_ptr) = lib_ref {
                                    // SAFETY: we only hold this pointer briefly, bus.tools loop ended
                                    unsafe { &*lib_ptr }.build_compression_prompt(&bus.kernel.tape)
                                } else {
                                    (String::new(), 0, 0)
                                }
                            };

                            if !prompt.is_empty() {
                                let memory_text = if let Some(ref librarian_llm) = deepseek_librarian {
                                    info!(">>> [LIBRARIAN] Calling DeepSeek V3 ({} success + {} failure nodes)...", sc, fc);
                                    match librarian_llm.resilient_generate(&prompt, 0, 0.3).await {
                                        Ok(response) => {
                                            info!(">>> [LIBRARIAN] DeepSeek V3 compression received ({} chars)", response.len());
                                            response
                                        }
                                        Err(e) => {
                                            warn!(">>> [LIBRARIAN] DeepSeek V3 failed: {:?}. Using local fallback.", e);
                                            String::new()
                                        }
                                    }
                                } else {
                                    String::new()
                                };

                                for t in &mut bus.tools {
                                    if t.manifest() == "core.tool.librarian" {
                                        if let Some(lib) = t.as_any_mut().downcast_mut::<LibrarianTool>() {
                                            if !memory_text.is_empty() {
                                                lib.write_memory(&memory_text);
                                            } else {
                                                lib.compress_local(&bus.kernel.tape);
                                            }
                                        }
                                        break;
                                    }
                                }
                            }
                            last_librarian_at = append_count;
                        }

                        // --- DeepSeek Halt Gate ---
                        // [COMPLETE] nodes don't auto-trigger OMEGA.
                        // Only when market price >= 90% do we call DeepSeek to verify.
                        if tx.payload.contains("[COMPLETE]") {
                            let file_id = format!("tx_{}_by_{}", tx_count, tx.agent_id.replace("Agent_", ""));
                            let price = bus.kernel.tape.files.get(&file_id)
                                .map(|n| n.price).unwrap_or(0.0);
                            info!(">>> [COMPLETE CLAIMED] {} at {} (P={:.1}%). Need 90% for DeepSeek verification.",
                                tx.agent_id, file_id, price * 100.0);
                        }

                        // Check ALL [COMPLETE] nodes for price threshold
                        let mut omega_candidate: Option<String> = None;
                        for (nid, node) in &bus.kernel.tape.files {
                            if node.payload.contains("[COMPLETE]") && node.price >= 0.9 {
                                info!(">>> [PRICE GATE] {} reached P={:.1}% — invoking DeepSeek Oracle!",
                                    nid, node.price * 100.0);
                                omega_candidate = Some(nid.clone());
                                break;
                            }
                        }

                        if let Some(ref candidate_id) = omega_candidate {
                            if let Some(ref oracle) = deepseek_oracle {
                                // Build the full proof chain for DeepSeek to verify
                                let chain = bus.kernel.trace_golden_path(candidate_id);
                                let mut proof_text = format!("PROBLEM: {}\n\nPROOF CHAIN ({} steps):\n", problem, chain.len());
                                for (i, nid) in chain.iter().rev().enumerate() {
                                    if let Some(n) = bus.kernel.tape.files.get(nid) {
                                        proof_text.push_str(&format!("Step {}: {}\n", i+1, n.payload.trim()));
                                    }
                                }
                                proof_text.push_str("\nIS THIS PROOF MATHEMATICALLY CORRECT AND COMPLETE? Answer YES or NO with brief justification.");

                                info!(">>> [DEEPSEEK ORACLE] Verifying {} ({} steps)...", candidate_id, chain.len());
                                match oracle.resilient_generate(&proof_text, 0, 0.1).await {
                                    Ok(response) => {
                                        let verdict = response.to_uppercase();
                                        if verdict.contains("YES") && !verdict.starts_with("NO") {
                                            info!(">>> [DEEPSEEK VERDICT] ✓ PROOF ACCEPTED! OMEGA!");
                                            bus.halt_and_settle(candidate_id);
                                            break;
                                        } else {
                                            warn!(">>> [DEEPSEEK VERDICT] ✗ PROOF REJECTED: {}", &response[..response.len().min(200)]);
                                        }
                                    }
                                    Err(e) => {
                                        warn!(">>> [DEEPSEEK ERROR] Oracle failed: {:?}. Continuing without halt.", e);
                                    }
                                }
                            } else {
                                // No DeepSeek key — fallback to old behavior
                                info!(">>> [OMEGA] No DeepSeek Oracle, accepting [COMPLETE] at P >= 90%");
                                bus.halt_and_settle(candidate_id);
                                break;
                            }
                        }
                    }
                    Err(e) => {
                        let preview: String = tx.payload.chars().take(100).collect();
                        warn!("[Tx {}] {} REJECTED: {} | {}", tx_count, tx.agent_id, e, preview.replace('\n', " "));
                        // Feed rejection back to agent (personal)
                        agent_rejections.lock().unwrap()
                            .insert(tx.agent_id.clone(), format!("{}", e));
                        // Post to global bulletin (all agents learn from this)
                        {
                            let err_msg = format!("{}", e);
                            let short_err: String = err_msg.chars().take(80).collect();
                            let mut b = global_bulletin.lock().unwrap();
                            let already = b.iter().any(|existing| {
                                let prefix: String = existing.chars().take(30).collect();
                                short_err.starts_with(&prefix)
                            });
                            if !already {
                                b.push(short_err);
                                if b.len() > 5 { b.remove(0); }
                            }
                        }
                        // Ground Truth: log failure to persistent file (append-only, never clear)
                        {
                            let log_path = format!("{}/failure.jsonl", log_dir);
                            if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(&log_path) {
                                use std::io::Write;
                                let preview: String = tx.payload.chars().take(100).collect();
                                let reason: String = format!("{}", e);
                                let _ = writeln!(f, r#"{{"author":"{}","payload":"{}","reason":"{}","ts":{}}}"#,
                                    tx.agent_id,
                                    preview.replace('"', "'").replace('\n', " "),
                                    reason.replace('"', "'").replace('\n', " "),
                                    epoch_secs());
                            }
                        }
                        // Record in Librarian (for backward compat, will be removed)
                        for t in &mut bus.tools {
                            if t.manifest() == "core.tool.librarian" {
                                // no-op: Librarian now reads from failure.jsonl
                                break;
                            }
                        }
                    }
                }

                let mut snap = bus.get_immutable_snapshot();
                snap.generation = generation;
                let _ = tx_state.send(snap);

                // No tx limit — run until OMEGA or manual stop (architect directive 2026-04-02)
            }
            Ok(None) => {
                info!("All senders dropped. Universe terminated.");
                break;
            }
            Err(_) => {
                let now = epoch_secs();
                let solvent_count = agent_names.iter()
                    .filter(|name| bus.get_agent_balance(name) >= 1.0)
                    .count();
                let last_free = free_action_epoch.load(Ordering::Relaxed);
                let secs_since_invest = now.saturating_sub(last_invest_epoch);
                let secs_since_free = now.saturating_sub(last_free);

                let all_bankrupt = solvent_count == 0;
                let absolute_stagnation = secs_since_invest >= 60 && secs_since_free >= 60;

                if all_bankrupt || absolute_stagnation {
                    let reason = if all_bankrupt { "Global bankruptcy" } else { "Absolute stagnation" };
                    error!("==== [MACROECONOMICS] {}! Gen {} perished. Solvent: {}/{} ====",
                        reason, generation, solvent_count, swarm_size);

                    // Autopsy/victory already handled in real-time (per-transaction).
                    // Just record deaths in graveyard here.
                    for name in &agent_names {
                        let bal = bus.get_agent_balance(name);
                        if bal < 1.0 {
                            bus.graveyard.record_death("root",
                                &format!("Gen {} bankrupt: {} (bal: {:.2})", generation, name, bal));
                        }
                    }

                    generation += 1;
                    info!(">>> [NO REBIRTH] Gen {} — zero new Coins. Free append only.", generation);

                    let mut snap = bus.get_immutable_snapshot();
                    snap.generation = generation;
                    let _ = tx_state.send(snap);
                    last_invest_epoch = epoch_secs();
                } else {
                    if secs_since_free < 30 {
                        info!("[MARKET WATCH] No invest for {}s. Agents reading (last free {}s ago).",
                            secs_since_invest, secs_since_free);
                    } else {
                        info!("[TIMEOUT] 30s idle. Solvent: {}/{}. Invest: {}s, Free: {}s.",
                            solvent_count, swarm_size, secs_since_invest, secs_since_free);
                    }
                }
            }
        }
    }

    // --- Final Output ---
    info!("==== EVALUATION COMPLETE ({} appends, {} total tx, {} generations) ====", append_count, tx_count, generation);
    bus.kernel.refresh_prices();

    if let Some(omega) = bus.kernel.tape.files.values()
        .find(|f| f.payload.contains("[OMEGA]") || f.payload.contains("[COMPLETE]"))
    {
        info!("--- PROOF CHAIN (Golden Path) ---");
        let path = bus.kernel.trace_golden_path(&omega.id);
        for (i, nid) in path.iter().rev().enumerate() {
            if let Some(n) = bus.kernel.tape.files.get(nid) {
                let step = n.payload.lines().last().unwrap_or(&n.payload).trim();
                info!("Step {}: [{}] P:{:.0} | {}", i+1, nid, n.price, step);
            }
        }
        info!("OMEGA: Proof chain COMPLETE!");
    } else {
        info!("NOT proved within {} transactions.", tx_count);
    }

    // --- Dual-Track Tape Dump ---
    info!("--- TAPE ({} nodes) ---", bus.kernel.tape.files.len());
    for (id, f) in &bus.kernel.tape.files {
        let char_count = f.payload.chars().count();
        let summary: String = f.payload.chars().take(150).collect::<String>().replace('\n', " ");
        let flag = if char_count > 150 { format!(" [+{}c]", char_count - 150) } else { String::new() };
        info!("{} | P:{:.0} | {}{}", id, f.price, summary, flag);
    }

    // Full tape dump
    let dump_path = "/tmp/zeta_sum_tape_full.md";
    if let Ok(mut f) = std::fs::File::create(dump_path) {
        use std::io::Write;
        let _ = writeln!(f, "# zeta_sum_proof — Full Tape Dump (Role Trifecta)\n");
        let _ = writeln!(f, "**Transactions**: {} | **Generations**: {} | **Nodes**: {}\n",
            tx_count, generation, bus.kernel.tape.files.len());
        let _ = writeln!(f, "**Provider**: {} | **Model**: {}\n",
            std::env::var("LLM_PROVIDER").unwrap_or_else(|_| "aliyun".into()),
            clients[0].model_name());
        let _ = writeln!(f, "**Roles**: 5 Math (0-4) / 5 Bull (5-9) / 5 Bear (10-14)\n");

        if let Some(omega) = bus.kernel.tape.files.values()
            .find(|n| n.payload.contains("[OMEGA]") || n.payload.contains("[COMPLETE]"))
        {
            let _ = writeln!(f, "## Golden Path\n");
            let path = bus.kernel.trace_golden_path(&omega.id);
            for (i, nid) in path.iter().rev().enumerate() {
                if let Some(n) = bus.kernel.tape.files.get(nid) {
                    let _ = writeln!(f, "### Step {} — `{}` (Price: {:.0})\n", i + 1, nid, n.price);
                    let _ = writeln!(f, "```\n{}\n```\n", n.payload);
                }
            }
        }

        let _ = writeln!(f, "## All Nodes\n");
        for nid in &bus.kernel.tape.time_arrow {
            if let Some(n) = bus.kernel.tape.files.get(nid) {
                let _ = writeln!(f, "### `{}` | Author: {} | Price: {:.0} | Citations: {:?}\n",
                    nid, n.author, n.price, n.citations);
                let _ = writeln!(f, "```\n{}\n```\n", n.payload);
            }
        }
        info!(">>> [IMMUTABLE TRACK] Full tape exported to {}", dump_path);
    }
}
