/// MiniF2F v2 Evaluator — Polymarket vFinal + Lean 4 Oracle
///
/// Magna Carta alignment:
/// - Engine 1: Free append + MathlibOracle search (Law 1)
/// - Engine 2: Polymarket YES/NO invest (Law 2)
/// - Engine 3: Lean 4 compiler as Oracle (Engine 3 Guillotine)
/// - OMEGA: "No goals to be solved" → proof complete

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
use turingosv3::sdk::sandbox::LocalProcessSandbox;
use turingosv3::bus::{TuringBus, ThermodynamicHeartbeatTool};
use minif2f_v2::lean4_oracle::Lean4Oracle;

const SWARM_SIZE: usize = 15;
const MAX_TRANSACTIONS: u64 = 300;

const SKILL: &str = "\
[LAW 1] APPEND IS FREE: Creating reasoning steps costs ZERO. Explore freely.\n\
[LAW 2] ONLY INVEST COSTS MONEY: Invest/Bet/Short are the ONLY actions that burn coins.\n\
[LAW 3] KELLY CRITERION: Start small (10-50). Invest >= 2 for directional bet.\n\
[LAW 4] POLYMARKET ECONOMICS:\n\
  - append: FREE mathematical reasoning step. No cost. Use this to explore.\n\
  - invest: Buy YES on a node you believe in. This is how you MAKE MONEY.\n\
  - short: Buy NO on a node you think is wrong. VERY PROFITABLE if right!\n\
[LAW 5] TRADITIONAL MATHEMATICS ONLY:\n\
  - Write your reasoning in TRADITIONAL MATH (natural language + standard notation).\n\
  - DO NOT write Lean 4 syntax, tactics, or code. The system will REJECT Lean syntax.\n\
  - Example good step: 'Since 3^7 | a^3+b^3+c^3, we work in Z/2187Z. By Hensel lemma...'\n\
  - Example BAD step: 'rw [ZMod.nat_cast_zmod_eq_zero_iff_dvd]' ← THIS WILL BE REJECTED\n\
  - Search Mathlib for relevant lemma NAMES: {\"tool\":\"search\",\"query\":\"Hensel\"}\n\
  - ONE REASONING STEP per submission. Each step = one atomic mathematical argument.\n\
  - MAXIMUM 1200 CHARACTERS per step. Longer submissions are REJECTED. Keep it atomic.\n\
  - When the full proof chain is mathematically complete, claim [COMPLETE].\n\
  - A translator will convert your math to Lean 4 for formal verification.\n\
  - FORBIDDEN: brute-force enumeration, case-bashing without structure.\n\
[STRATEGY GUIDE — READ CAREFULLY]:\n\
  1. EXPLORE: Use 'append' freely to try reasoning steps at zero risk.\n\
  2. EVALUATE: Use 'view' to read other agents' nodes. Which approaches look promising?\n\
  3. INVEST: When you find a strong approach (yours or others'), INVEST in it!\n\
     - Invested nodes rank HIGHER in the leaderboard.\n\
     - Higher rank = more agents build on YOUR approach = faster path to OMEGA.\n\
     - If your invested node reaches OMEGA, your YES shares pay out. PROFIT!\n\
  4. SHORT: See a bad node with high rank? Short it! If it fails, you profit from the creator's loss.\n\
  5. CLAIM COMPLETE: When you believe the full proof chain is done, write [COMPLETE].\n\
     The system translates your math to Lean 4 and verifies. If correct = OMEGA!\n\
DO NOT just append forever. INVEST in your best work to attract collaborators.\n\
Balance < 1.0 = can only append (free). Cannot invest/bet/short.\n";

fn epoch_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Load a MiniF2F problem from the Lean 4 data directory.
/// Returns (full_statement, theorem_name).
/// CLAUDE.md #21: Warns if formalization contains brute-force search spaces.
fn load_problem(problem_file: &str) -> (String, String) {
    let lean4_dir = std::env::var("MINIF2F_LEAN4_DIR")
        .unwrap_or_else(|_| "/home/zephryj/projects/turingosv3/experiments/minif2f_data_lean4/MiniF2F/Test".to_string());
    let path = format!("{}/{}", lean4_dir, problem_file);
    let content = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Cannot read {}: {}", path, e));

    // CLAUDE.md #21: Check for brute-force search space in formalization
    let brute_force_patterns = ["Finset.range", "Finset.Icc", "Finset.Ico", "List.range"];
    for pattern in &brute_force_patterns {
        if content.contains(pattern) {
            warn!(">>> [FORMALIZATION WARNING] '{}' found in problem statement!", pattern);
            warn!("    This may allow brute-force verification (decide/omega).");
            warn!("    Consider using universal quantifiers (∀) instead. (CLAUDE.md #21)");
        }
    }

    // Extract theorem name: "theorem <name> ..."
    let theorem_name = content.lines()
        .find(|l| l.starts_with("theorem "))
        .and_then(|l| l.split_whitespace().nth(1))
        .unwrap_or("unknown")
        .to_string();

    // Replace "by sorry" with "by\n" to leave room for agent tactics
    let problem = content.replace("by sorry", "by\n");

    info!("Loaded problem: {} (theorem: {})", problem_file, theorem_name);
    (problem, theorem_name)
}

/// Translate a chain of traditional math reasoning steps into Lean 4 tactics.
/// Called at OMEGA time by the evaluator (Engine 3).
async fn translate_math_to_lean(
    client: &ResilientLLMClient,
    problem_statement: &str,
    math_chain: &str,
) -> String {
    let prompt = format!(
        "You are a Lean 4 formalization expert. Translate the following mathematical reasoning \
into a Lean 4 tactic proof.\n\n\
PROBLEM (Lean 4 statement — fill in the proof after 'by'):\n{}\n\n\
MATHEMATICAL REASONING CHAIN:\n{}\n\n\
OUTPUT RULES:\n\
- Output ONLY the Lean 4 tactic block (indented with 2 spaces, one tactic per line).\n\
- Do NOT include 'theorem', 'by', imports, or any wrapper — only the tactics.\n\
- Do NOT use native_decide, decide, or omega.\n\
- Do NOT use sorry.\n\
- Use Mathlib tactics: simp, rw, exact, have, calc, norm_num, ring, linarith, etc.\n\
- Output NOTHING except the tactic lines.",
        problem_statement, math_chain
    );

    match client.resilient_generate(&prompt, 99, 0.1).await {
        Ok(raw) => {
            // Extract code block if present
            let code = if let Some(start) = raw.find("```") {
                let after = &raw[start + 3..];
                let lang_end = after.find('\n').unwrap_or(0);
                let code_start = lang_end + 1;
                if let Some(end) = after[code_start..].find("```") {
                    after[code_start..code_start + end].to_string()
                } else {
                    after[code_start..].to_string()
                }
            } else {
                raw.clone()
            };
            code.trim().to_string()
        }
        Err(e) => {
            warn!(">>> [TRANSLATOR ERROR] {:?}", e);
            String::new()
        }
    }
}

#[tokio::main]
async fn main() {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    // Problem selection: from CLI arg or default
    let problem_file = std::env::args().nth(1)
        .unwrap_or_else(|| "mathd_algebra_48.lean".to_string());

    let (problem_statement, theorem_name) = load_problem(&problem_file);

    info!("=== MiniF2F v2 (Polymarket + Lean 4 Oracle) ===");
    info!("Problem: {} | Theorem: {}", problem_file, theorem_name);
    info!("N={}, Max Transactions={}", SWARM_SIZE, MAX_TRANSACTIONS);

    // --- Initialize Bus + Tools ---
    let kernel = Kernel::new();
    let mut bus = TuringBus::new(kernel);
    bus.mount_tool(Box::new(ThermodynamicHeartbeatTool::new(1)));
    bus.mount_tool(Box::new(AntiZombiePruningTool::new(3)));
    bus.mount_tool(Box::new(WalletTool::new()));

    // Lean 4 Oracle (Engine 3: Popperian Guillotine)
    // LEAN_PATH must include Mathlib olean directory for `import Mathlib` to work
    let lean_path = std::env::var("LEAN_PATH").unwrap_or_else(|_| {
        // MATHLIB_ROOT points to the minif2f_data_lean4 directory (contains .lake/packages)
        let mathlib_root = std::env::var("MATHLIB_ROOT")
            .unwrap_or_else(|_| {
                let base = std::env::var("MINIF2F_LEAN4_DIR")
                    .unwrap_or_else(|_| "/home/zephryj/projects/turingosv3/experiments/minif2f_data_lean4/MiniF2F/Test".to_string());
                // Navigate up from problem dir to find the data root with .lake
                let mut path = std::path::PathBuf::from(&base);
                while !path.join(".lake/packages").exists() {
                    if !path.pop() { break; }
                }
                path.to_string_lossy().to_string()
            });
        let data_root = std::path::Path::new(&mathlib_root);
        let packages_dir = data_root.join(".lake/packages");
        // Collect all package olean directories
        let mut paths = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&packages_dir) {
            for entry in entries.flatten() {
                let lib_dir = entry.path().join(".lake/build/lib/lean");
                if lib_dir.exists() {
                    paths.push(lib_dir.to_string_lossy().to_string());
                }
                // Also check direct build/lib (some packages use different layout)
                let alt_dir = entry.path().join(".lake/build/lib");
                if alt_dir.exists() && !lib_dir.exists() {
                    paths.push(alt_dir.to_string_lossy().to_string());
                }
            }
        }
        // Also add the project's own build lib
        let own_lib = data_root.join(".lake/build/lib/lean");
        if own_lib.exists() { paths.push(own_lib.to_string_lossy().to_string()); }
        paths.join(":")
    });
    std::env::set_var("LEAN_PATH", &lean_path);
    info!("LEAN_PATH set to: {}", lean_path);

    let lean_cmd = std::env::var("LEAN_CMD").unwrap_or_else(|_| "lean".to_string());

    // Lean4Oracle: security checks only (sorry, forbidden, identity theft)
    // OMEGA verification happens in the reactor loop below.
    bus.mount_tool(Box::new(Lean4Oracle::new(
        problem_statement.clone(),
        theorem_name.clone(),
    )));

    // Separate sandbox for OMEGA verification (evaluator-side, not tool-side)
    let omega_sandbox = LocalProcessSandbox::new(&lean_cmd, vec!["--stdin".to_string()]);

    // GENESIS: only allocate for spawned agents (not 100 phantoms)
    let agent_ids: Vec<String> = (0..SWARM_SIZE).map(|i| format!("Agent_{}", i)).collect();
    bus.init_problem(&agent_ids);

    // --- Create Channels ---
    let mut init_snap = bus.get_immutable_snapshot();
    init_snap.generation = 1;
    let (tx_state, rx_state) = watch::channel(init_snap);
    let (tx_mempool, mut rx_mempool) = mpsc::channel::<MinerTx>(1000);

    // --- Build Clients ---
    let ds_url = "https://api.deepseek.com/chat/completions";
    let key_ds = std::env::var("DEEPSEEK_API_KEY").expect("DEEPSEEK_API_KEY required");

    let clients: Vec<Arc<ResilientLLMClient>> = vec![
        Arc::new(ResilientLLMClient::with_key(ds_url, "deepseek-chat", &key_ds)),
        Arc::new(ResilientLLMClient::with_key(ds_url, "deepseek-reasoner", &key_ds)),
    ];
    info!("Models: DeepSeek-V3.2 + Reasoner");

    // --- Search Tool (Mathlib) ---
    let search_tool = Arc::new(SearchTool::new(vec![
        "/home/zephryj/projects/turingosv3/experiments/minif2f_data_lean4/.lake/packages/mathlib/Mathlib".to_string(),
    ]));

    // Translator client: converts traditional math → Lean 4 at OMEGA time
    let translator_client = Arc::new(ResilientLLMClient::with_key(ds_url, "deepseek-chat", &key_ds));
    info!(">>> [TRANSLATOR] Math→Lean translator ready (deepseek-chat)");

    let free_action_epoch = Arc::new(AtomicU64::new(epoch_secs()));

    // Per-agent last rejection feedback (reactor writes, agent reads)
    let agent_rejections: Arc<Mutex<HashMap<String, String>>> = Arc::new(Mutex::new(HashMap::new()));

    // --- Law 3 / Engine 4: Per-Agent Skill Paths ---
    let skills_dir = std::env::var("AGENT_SKILLS_DIR")
        .unwrap_or_else(|_| "/tmp/turingos_skills".to_string());
    for i in 0..SWARM_SIZE {
        let agent_dir = format!("{}/agent_{}", skills_dir, i);
        let _ = std::fs::create_dir_all(&agent_dir);
    }
    // Engine 4: Seed Falsifier Agent (Agent_14) — dedicated mathematical skeptic
    // Gemini review 2026-03-30: implement via skills/ only, no kernel changes
    let falsifier_idx = SWARM_SIZE - 1; // Last agent = falsifier
    let falsifier_path = format!("{}/agent_{}/learned.md", skills_dir, falsifier_idx);
    if !std::path::Path::new(&falsifier_path).exists() || std::fs::read_to_string(&falsifier_path).unwrap_or_default().is_empty() {
        let _ = std::fs::write(&falsifier_path,
            "# ROLE: Mathematical Falsifier (via negativa)\n\
            You are a MATHEMATICAL FALSIFIER. Your sole purpose is to find LOGICAL ERRORS.\n\n\
            STRATEGY:\n\
            1. READ the highest P_yes nodes (these have the most consensus — and the most risk)\n\
            2. FIND logical gaps: quantifier errors, unjustified leaps, missing cases, wrong inequalities\n\
            3. APPEND a clear explanation of the flaw you found (cite the node ID)\n\
            4. SHORT the flawed node to profit from its collapse\n\n\
            You WIN by destroying false consensus. Every error you catch saves the swarm from wasting compute.\n\
            Do NOT build new proof paths. Only ATTACK existing ones.\n\
            The most profitable move is finding an error that everyone else missed.\n"
        );
        info!(">>> [FALSIFIER] Agent_{} seeded as Mathematical Falsifier", falsifier_idx);
    }
    info!(">>> [ENGINE 4] Per-agent skill paths initialized at {}/", skills_dir);

    // --- Spawn Agent Loops ---
    for i in 0..SWARM_SIZE {
        let client = clients[i % clients.len()].clone();
        let mut rx = rx_state.clone();
        let tx = tx_mempool.clone();
        let problem = problem_statement.clone();
        let skill = SKILL.to_string();
        let search = search_tool.clone();
        let private_ctx = Arc::new(Mutex::new(String::new()));
        let heartbeat = free_action_epoch.clone();
        let agent_skill_dir = format!("{}/agent_{}", skills_dir, i);
        let rejections = agent_rejections.clone();

        let is_falsifier = i == falsifier_idx;
        info!(">>> [SPAWN] Agent {} → {} | $HOME: {}{}", i, client.model_name(), agent_skill_dir,
            if is_falsifier { " [FALSIFIER]" } else { "" });

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

                // Law 3 / Engine 4: Load agent-specific learned skills
                let agent_skill = std::fs::read_to_string(
                    format!("{}/learned.md", agent_skill_dir)
                ).unwrap_or_default();

                let (chain, parent_id) = build_chain_from_snapshot(&snapshot, &problem);
                let private = private_ctx.lock().unwrap().clone();
                let graveyard = snapshot.tombstones.values()
                    .take(3).cloned().collect::<Vec<_>>().join("\n");

                // Read last rejection for this agent (error feedback loop)
                let last_rejection = rejections.lock().unwrap()
                    .get(&agent_name).cloned().unwrap_or_default();
                let feedback = if last_rejection.is_empty() {
                    String::new()
                } else if last_rejection.contains("FRONT-RUNNING") {
                    format!("\n=== YOUR LAST SUBMISSION WAS REJECTED (TOO LONG) ===\n{}\n=== MAX 1200 CHARS. Write ONE short atomic math argument. Split multi-step reasoning into separate submissions. ===\n", last_rejection)
                } else {
                    format!("\n=== YOUR LAST SUBMISSION WAS REJECTED ===\n{}\n=== Write traditional math reasoning. Do NOT use Lean syntax. ===\n", last_rejection)
                };

                let p = prompt::build_agent_prompt(
                    &chain,
                    &format!("{}\n{}", skill, agent_skill),
                    &snapshot.market_ticker,
                    &format!("{}\n{}\n{}", graveyard, private, feedback),
                    balance,
                    "append: {\"tool\":\"append\",\"tactic\":\"your mathematical reasoning step\"} (FREE — traditional math only)\ninvest: {\"tool\":\"invest\",\"tactic\":\"your mathematical reasoning step\",\"amount\":PRICE} (creates node + buys YES)\nbet: {\"tool\":\"invest\",\"node\":\"node_id\",\"amount\":PRICE} (buy YES on existing)\nshort: {\"tool\":\"short\",\"node\":\"node_id\",\"amount\":PRICE} (buy NO)\nsearch: {\"tool\":\"search\",\"query\":\"term\"} (FREE Mathlib search)\nview: {\"tool\":\"view_node\",\"query\":\"node_id\"} (FREE)",
                );

                let temp = 0.2 + 0.6 * (i as f32 / SWARM_SIZE.max(1) as f32);
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
                                    if is_falsifier {
                                        // Engine 4: Falsifier cannot buy YES — structural enforcement
                                        info!(">>> [FALSIFIER] {} invest→NOP: falsifiers cannot buy YES", agent_name);
                                        heartbeat.store(epoch_secs(), Ordering::Relaxed);
                                    } else {
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

                // VOLUNTARY INVESTMENT ROUND — agent MAY invest, short, or pass
                // Magna Carta Law 2: investment must be autonomous (Gemini audit 2026-03-29)
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

                    let invest_prompt = if is_falsifier {
                        // Engine 4: Falsifier can only SHORT or PASS — structural enforcement
                        format!(
                            "You are a MATHEMATICAL FALSIFIER in a proof market. Your balance: {:.0} Coins.\n\
                            Your role: find flawed reasoning and PROFIT by betting against it.\n\
                            Recent nodes (most recent first):\n{}\n\n\
                            You MAY take ONE action or pass:\n\
                            - Bet AGAINST a flawed node: <action>{{\"tool\":\"short\",\"node\":\"NODE_ID\",\"amount\":COINS}}</action>\n\
                            - Pass (no action): <action>{{\"tool\":\"pass\"}}</action>\n\
                            Minimum 2 Coins per trade. You WIN when flawed nodes collapse.\n\
                            Look for: quantifier errors, unjustified leaps, missing cases, wrong counts.",
                            invest_balance, node_list
                        )
                    } else {
                        format!(
                            "You are an investor in a proof market. Your balance: {:.0} Coins.\n\
                            Recent nodes (most recent first):\n{}\n\n\
                            You MAY take ONE action or pass:\n\
                            - Back a node: <action>{{\"tool\":\"invest\",\"node\":\"NODE_ID\",\"amount\":COINS}}</action>\n\
                            - Bet against: <action>{{\"tool\":\"short\",\"node\":\"NODE_ID\",\"amount\":COINS}}</action>\n\
                            - Pass (no action): <action>{{\"tool\":\"pass\"}}</action>\n\
                            Minimum 2 Coins per trade. Passing costs nothing but yields no profit.\n\
                            Remember: ONLY invested nodes pay out at OMEGA. Topological dominance requires capital.",
                            invest_balance, node_list
                        )
                    };

                    match client.resilient_generate(&invest_prompt, i, 0.3).await {
                        Ok(raw) => {
                            if let Some(action) = parse_agent_output(&raw) {
                                match action.tool.as_str() {
                                    "invest" => {
                                        if is_falsifier {
                                            info!(">>> [FALSIFIER] {} invest→NOP in investment round: falsifiers cannot buy YES", agent_name);
                                            heartbeat.store(epoch_secs(), Ordering::Relaxed);
                                        } else {
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
                                                info!(">>> [INVEST] {} bet YES {:.0} on {}", agent_name, amount, node);
                                            }
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
                                            info!(">>> [SHORT] {} bet NO {:.0} on {}", agent_name, amount, node);
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

    let agent_names: Vec<String> = (0..SWARM_SIZE).map(|i| format!("Agent_{}", i)).collect();
    info!(">>> TuringOS v3 MiniF2F Booted. {} agents. <<<", SWARM_SIZE);

    // --- Reactor Loop ---
    let mut tx_count: u64 = 0;
    let mut generation: u32 = 1;
    let mut last_invest_epoch = epoch_secs();

    loop {
        match tokio::time::timeout(
            tokio::time::Duration::from_secs(30),
            rx_mempool.recv()
        ).await {
            Ok(Some(tx)) => {
                if tx.action_type == "invest" || tx.action_type == "short" {
                    last_invest_epoch = epoch_secs();
                }
                tx_count += 1;

                let file = File {
                    id: format!("tx_{}_by_{}", tx_count, tx.agent_id.replace("Agent_", "")),
                    author: tx.agent_id.clone(),
                    payload: tx.payload.clone(),
                    citations: tx.parent_id.iter().cloned().collect(),
                    stake: 1,
                    intrinsic_reward: 0.0,
                    price: 0.0,
                };

                let file_id = format!("tx_{}_by_{}", tx_count, tx.agent_id.replace("Agent_", ""));
                match bus.append(file) {
                    Ok(_) => {
                        info!("[Tx {}] {} ({}) → Appended", tx_count, tx.agent_id, tx.model_name);
                        agent_rejections.lock().unwrap().remove(&tx.agent_id); // Clear rejection on success
                        bus.tick_map_reduce();

                        // OMEGA Detection: Agent claims [COMPLETE] → compile FULL PROOF CHAIN.
                        // Oracle (Engine 3) fires ONLY here. Intermediate appends are free (Law 1).
                        let claims_complete = bus.kernel.tape.files.get(&file_id)
                            .map(|n| n.payload.contains("[COMPLETE]"))
                            .unwrap_or(false);

                        let mut is_omega = false;
                        if claims_complete {
                            info!(">>> [COMPLETE CLAIM] {} claims proof complete. Translating math→Lean...", tx.agent_id);

                            // Build full math reasoning chain: all ancestor steps concatenated
                            let path = bus.kernel.trace_golden_path(&file_id);
                            let mut math_steps = Vec::new();
                            for (i, nid) in path.iter().rev().enumerate() {
                                if let Some(node) = bus.kernel.tape.files.get(nid) {
                                    let step = node.payload
                                        .split("[Tool: Wallet").next().unwrap_or(&node.payload)
                                        .replace("[COMPLETE]", "")
                                        .trim().to_string();
                                    if !step.is_empty() {
                                        math_steps.push(format!("Step {}: {}", i + 1, step));
                                    }
                                }
                            }
                            let math_chain = math_steps.join("\n");
                            info!(">>> [MATH CHAIN] {} steps collected", math_steps.len());

                            // Engine 3: Translate math → Lean 4 (with error-feedback retry)
                            let mut last_error = String::new();
                            for attempt in 0..2 {
                                let translation_input = if last_error.is_empty() {
                                    math_chain.clone()
                                } else {
                                    format!("{}\n\n--- PREVIOUS ATTEMPT FAILED ---\n{}\n--- Fix the translation. ---",
                                        math_chain, last_error)
                                };

                                let lean_chain = translate_math_to_lean(
                                    &translator_client, &problem_statement, &translation_input
                                ).await;

                                if lean_chain.is_empty() {
                                    warn!(">>> [TRANSLATOR] Attempt {} failed (empty output)", attempt + 1);
                                    last_error = "Translation produced empty output".to_string();
                                    continue;
                                }

                                info!(">>> [TRANSLATOR] Attempt {} produced {} lines of Lean",
                                    attempt + 1, lean_chain.lines().count());

                                // Security check: translation output must pass Oracle guards
                                let security_ok = minif2f_v2::lean4_oracle::check_translated_output(
                                    &lean_chain, &theorem_name
                                );
                                if !security_ok {
                                    warn!(">>> [TRANSLATOR] Security check FAILED on attempt {}", attempt + 1);
                                    last_error = "Security check failed: forbidden pattern in translation".to_string();
                                    continue;
                                }

                                // Compile translated Lean
                                is_omega = minif2f_v2::lean4_oracle::verify_omega(
                                    &omega_sandbox, &problem_statement, &lean_chain
                                );

                                if is_omega { break; }
                                last_error = format!("Lean compilation failed on attempt {}", attempt + 1);
                                warn!(">>> [OMEGA] {}", last_error);
                            }

                            if !is_omega {
                                warn!(">>> [COMPLETE REJECTED] Translation/compilation failed.");
                                bus.graveyard.record_death(&file_id, "OMEGA: math→Lean translation failed");
                            }
                        }

                        if is_omega {
                            info!("OMEGA at Tx {}! Theorem {} PROVED!", tx_count, problem_file);

                            // Snapshot balances before settlement
                            let pre_balances: std::collections::HashMap<String, f64> = agent_names.iter()
                                .map(|n| (n.clone(), bus.get_agent_balance(n)))
                                .collect();

                            bus.halt_and_settle(&file_id);

                            // Engine 4: Victory Reinforcement — profitable agents write experience
                            for (idx, name) in agent_names.iter().enumerate() {
                                let pre = pre_balances.get(name).copied().unwrap_or(0.0);
                                let post = bus.get_agent_balance(name);
                                let profit = post - pre;
                                if profit > 0.0 {
                                    let victory_path = format!("{}/agent_{}/learned.md", skills_dir, idx);
                                    let victory = format!(
                                        "# Victory — Profit {:.2} Coins\nSTRATEGY THAT WORKED: Won {:.2} from settlement.\n\n",
                                        profit, profit
                                    );
                                    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(&victory_path) {
                                        use std::io::Write;
                                        let _ = write!(f, "{}", victory);
                                    }
                                    info!(">>> [VICTORY] {} wrote experience (+{:.2}) to skills", name, profit);
                                }
                            }

                            break;
                        }
                    }
                    Err(e) => {
                        let preview: String = tx.payload.chars().take(100).collect();
                        warn!("[Tx {}] {} REJECTED: {} | {}", tx_count, tx.agent_id, e, preview.replace('\n', " "));
                        // Feed rejection back to the specific agent for learning
                        let rejection_msg: String = e.chars().take(300).collect();
                        agent_rejections.lock().unwrap().insert(tx.agent_id.clone(), rejection_msg);
                    }
                }

                let mut snap = bus.get_immutable_snapshot();
                snap.generation = generation;
                let _ = tx_state.send(snap);

                if tx_count >= MAX_TRANSACTIONS {
                    info!("Max transactions ({}) reached. Theorem NOT proved.", MAX_TRANSACTIONS);
                    break;
                }
            }
            Ok(None) => {
                info!("All agents dropped.");
                break;
            }
            Err(_) => {
                let now = epoch_secs();
                let solvent_count = agent_names.iter()
                    .filter(|name| bus.get_agent_balance(name) >= 1.0).count();
                let last_free = free_action_epoch.load(Ordering::Relaxed);
                let secs_since_invest = now.saturating_sub(last_invest_epoch);
                let secs_since_free = now.saturating_sub(last_free);

                let all_bankrupt = solvent_count == 0;
                // Lean 4 compilation + LLM reasoning = 60-300s per cycle. Use 300s timeout.
                let absolute_stagnation = secs_since_invest >= 300 && secs_since_free >= 300;

                if all_bankrupt || absolute_stagnation {
                    let reason = if all_bankrupt { "Global bankruptcy" } else { "Absolute stagnation" };
                    error!("[STAGNATION] {}! Gen {} stuck. Solvent: {}/{}", reason, generation, solvent_count, SWARM_SIZE);

                    // Engine 4: Autopsy Mutation — bankrupt agents write survival rules
                    for (idx, name) in agent_names.iter().enumerate() {
                        let bal = bus.get_agent_balance(name);
                        if bal < 1.0 {
                            bus.graveyard.record_death("root", &format!("Gen {} bankrupt: {}", generation, name));
                            let autopsy_path = format!("{}/agent_{}/learned.md", skills_dir, idx);
                            let autopsy = format!(
                                "# Autopsy — Generation {} Death\nBANKRUPT at balance {:.2}.\nSURVIVAL RULE: {}\n\n",
                                generation, bal, reason
                            );
                            if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(&autopsy_path) {
                                use std::io::Write;
                                let _ = write!(f, "{}", autopsy);
                            }
                            info!(">>> [AUTOPSY] {} wrote survival rule", name);
                        }
                    }

                    // Magna Carta Law 2: NO NEW MONEY. Bankrupt agents can still free append (Law 1).
                    // Coins are locked in CTF vault. They return when OMEGA triggers settlement.
                    // The system continues in free-append-only mode.
                    generation += 1;
                    info!(">>> [NO REBIRTH] Generation {} — zero new Coins. Agents must free-append to reach OMEGA.", generation);
                    let mut snap = bus.get_immutable_snapshot();
                    snap.generation = generation;
                    let _ = tx_state.send(snap);
                    last_invest_epoch = epoch_secs();
                } else if secs_since_free < 30 {
                    info!("[WATCH] Agents researching. Respecting Law 1.");
                } else {
                    info!("[TIMEOUT] Idle. Solvent: {}/{}. Invest: {}s, Free: {}s",
                        solvent_count, SWARM_SIZE, secs_since_invest, secs_since_free);
                }
            }
        }
    }

    // --- Final Output ---
    info!("==== COMPLETE ({} tx, {} gen) ====", tx_count, generation);
    bus.kernel.refresh_prices();

    if let Some(omega) = bus.kernel.tape.files.values()
        .find(|f| f.payload.ends_with("-- [OMEGA:03b17cc758d1492dc24d53ba008e4ed6]"))
    {
        info!("--- PROOF (Golden Path) ---");
        let path = bus.kernel.trace_golden_path(&omega.id);
        for (i, nid) in path.iter().rev().enumerate() {
            if let Some(n) = bus.kernel.tape.files.get(nid) {
                let step = n.payload.lines().last().unwrap_or(&n.payload).trim();
                info!("Step {}: [{}] | {}", i+1, nid, step);
            }
        }
        info!("PROVED: {}", theorem_name);
    } else {
        info!("NOT PROVED within {} transactions.", tx_count);
    }

    // Tape dump
    let dump_path = format!("/tmp/minif2f_v2_{}.md", problem_file.replace(".lean", ""));
    if let Ok(mut f) = std::fs::File::create(&dump_path) {
        use std::io::Write;
        let _ = writeln!(f, "# MiniF2F v2 — {}\n", problem_file);
        let _ = writeln!(f, "**Tx**: {} | **Gen**: {} | **Nodes**: {}\n", tx_count, generation, bus.kernel.tape.files.len());
        for nid in &bus.kernel.tape.time_arrow {
            if let Some(n) = bus.kernel.tape.files.get(nid) {
                let _ = writeln!(f, "### `{}` | {} | P:{:.2}\n```\n{}\n```\n", nid, n.author, n.price, n.payload);
            }
        }
        info!("Tape exported to {}", dump_path);
    }
}
