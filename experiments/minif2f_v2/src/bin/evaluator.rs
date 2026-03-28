/// MiniF2F v2 Evaluator — Polymarket vFinal + Lean 4 Oracle
///
/// Magna Carta alignment:
/// - Engine 1: Free append + MathlibOracle search (Law 1)
/// - Engine 2: Polymarket YES/NO invest (Law 2)
/// - Engine 3: Lean 4 compiler as Oracle (Engine 3 Guillotine)
/// - OMEGA: "No goals to be solved" → proof complete

use log::{info, warn, error};
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
[LAW 1] APPEND IS FREE: Creating proof steps costs ZERO. Explore freely.\n\
[LAW 2] ONLY INVEST COSTS MONEY: Invest/Bet/Short are the ONLY actions that burn coins.\n\
[LAW 3] KELLY CRITERION: Start small (10-50). Invest >= 2 for directional bet.\n\
[LAW 4] POLYMARKET ECONOMICS:\n\
  - append: FREE tactic submission. Lean 4 compiler validates. No cost.\n\
  - invest: Buy YES on your own node = you believe your tactic is correct.\n\
  - bet: Buy YES on someone else's node = you back their approach.\n\
  - short: Buy NO on any node = you believe it's wrong. VERY PROFITABLE if right!\n\
  - Your profit comes ONLY from finding mispriced probabilities.\n\
[LAW 5] LEAN 4 IS ABSOLUTE TRUTH: The compiler decides. No negotiation.\n\
  - If compiler says 'error' → your tactic is WRONG, period.\n\
  - If compiler says 'No goals to be solved' → OMEGA (proof complete!).\n\
  - Search Mathlib freely to find lemmas before writing tactics.\n\
Balance < 1.0 = can only append (free). Cannot invest/bet/short.\n";

fn epoch_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Load a MiniF2F problem from the Lean 4 data directory.
/// Returns (full_statement, theorem_name).
fn load_problem(problem_file: &str) -> (String, String) {
    let lean4_dir = std::env::var("MINIF2F_LEAN4_DIR")
        .unwrap_or_else(|_| "/home/zephryj/projects/turingosv3/experiments/minif2f_data_lean4/MiniF2F/Test".to_string());
    let path = format!("{}/{}", lean4_dir, problem_file);
    let content = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Cannot read {}: {}", path, e));

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
    let lean_cmd = std::env::var("LEAN_CMD").unwrap_or_else(|_| "lean".to_string());
    let sandbox = LocalProcessSandbox::new(&lean_cmd, vec!["--run".to_string()]);
    bus.mount_tool(Box::new(Lean4Oracle::new(
        problem_statement.clone(),
        theorem_name.clone(),
        Box::new(sandbox),
    )));

    let agent_ids: Vec<String> = (0..100).map(|i| format!("Agent_{}", i)).collect();
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

    let free_action_epoch = Arc::new(AtomicU64::new(epoch_secs()));

    // --- Law 3 / Engine 4: Per-Agent Skill Paths ---
    let skills_dir = std::env::var("AGENT_SKILLS_DIR")
        .unwrap_or_else(|_| "/tmp/turingos_skills".to_string());
    for i in 0..SWARM_SIZE {
        let agent_dir = format!("{}/agent_{}", skills_dir, i);
        let _ = std::fs::create_dir_all(&agent_dir);
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

        info!(">>> [SPAWN] Agent {} → {} | $HOME: {}", i, client.model_name(), agent_skill_dir);

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

                let p = prompt::build_agent_prompt(
                    &chain,
                    &format!("{}\n{}", skill, agent_skill), // Per-agent skill injection
                    &snapshot.market_ticker,
                    &format!("{}\n{}", graveyard, private),
                    balance,
                    "append: {\"tool\":\"append\",\"tactic\":\"your lean4 tactic\"} (FREE — Lean 4 validates)\ninvest: {\"tool\":\"invest\",\"tactic\":\"your lean4 tactic\",\"amount\":PRICE} (creates node + buys YES)\nbet: {\"tool\":\"invest\",\"node\":\"node_id\",\"amount\":PRICE} (buy YES on existing)\nshort: {\"tool\":\"short\",\"node\":\"node_id\",\"amount\":PRICE} (buy NO)\nsearch: {\"tool\":\"search\",\"query\":\"term\"} (FREE Mathlib search)\nview: {\"tool\":\"view_node\",\"query\":\"node_id\"} (FREE)",
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
                        bus.tick_map_reduce();

                        // SECURITY: Check the APPENDED node payload (post-Lean4Oracle Modify),
                        // NOT the raw MinerTx.payload. This prevents comment-injection attacks
                        // where an agent writes "-- [OMEGA:03b17cc758d1492dc24d53ba008e4ed6]" in a Lean 4 comment.
                        // Only Lean4Oracle can inject [OMEGA] via ToolSignal::Modify after
                        // verifying "No goals to be solved" with the compiler.
                        let is_omega = bus.kernel.tape.files.get(&file_id)
                            .map(|n| n.payload.ends_with("-- [OMEGA:03b17cc758d1492dc24d53ba008e4ed6]"))
                            .unwrap_or(false);
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
                let absolute_stagnation = secs_since_invest >= 60 && secs_since_free >= 60;

                if all_bankrupt || absolute_stagnation {
                    let reason = if all_bankrupt { "Global bankruptcy" } else { "Absolute stagnation" };
                    error!("[REBIRTH] {}! Gen {} dead. Solvent: {}/{}", reason, generation, solvent_count, SWARM_SIZE);

                    // Engine 4: Autopsy Mutation — bankrupt agents write survival rules
                    for (idx, name) in agent_names.iter().enumerate() {
                        let bal = bus.get_agent_balance(name);
                        if bal < 1.0 {
                            bus.graveyard.record_death("root", &format!("Gen {} bankrupt: {}", generation, name));
                            // Write autopsy to agent's skill directory
                            let autopsy_path = format!("{}/agent_{}/learned.md", skills_dir, idx);
                            let autopsy = format!(
                                "# Autopsy — Generation {} Death\nBANKRUPT at balance {:.2}.\nSURVIVAL RULE: {}\n\n",
                                generation, bal, reason
                            );
                            if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(&autopsy_path) {
                                use std::io::Write;
                                let _ = write!(f, "{}", autopsy);
                            }
                            info!(">>> [AUTOPSY] {} wrote survival rule to {}", name, autopsy_path);
                        }
                    }

                    generation += 1;
                    for name in &agent_names { bus.fund_agent(name, 10000.0); }
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
