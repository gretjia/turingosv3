use log::{info, warn, error};
use std::sync::{Arc, Mutex};
use tokio::sync::{mpsc, watch};
use turingosv3::kernel::{File, Kernel};
use turingosv3::drivers::llm_http::ResilientLLMClient;
use turingosv3::sdk::tools::wallet::WalletTool;
use turingosv3::sdk::tool::AntiZombiePruningTool;
use turingosv3::sdk::actor::{MinerTx, build_chain_from_snapshot, view_node};
use turingosv3::sdk::protocol::parse_agent_output;
use turingosv3::sdk::prompt;
use turingosv3::sdk::tools::search::SearchTool;
use turingosv3::bus::{TuringBus, ThermodynamicHeartbeatTool};
use zeta_sum_proof::math_membrane::MathStepMembrane;

const SWARM_SIZE: usize = 15;
const MAX_TRANSACTIONS: u64 = 200;

const PROBLEM: &str = r#"PROVE: 1 + 2 + 3 + 4 + ... = -1/12 (in the sense of regularization)

HINT FORMULA: M(m,N) = m * exp(-m/N) * cos(m/N)

KEY IDEA: For each fixed N, the series S(N) = Σ_{m=0}^∞ M(m,N) converges.
The limit as N→∞ of S(N) equals -1/12, even though the ordinary sum Σm diverges.

RULES:
- Write exactly ONE mathematical reasoning step
- Use only university-level calculus (series, limits, complex exponentials)
- Your step must logically follow from the previous steps shown above
- When the proof reaches -1/12, declare [COMPLETE]"#;

const SKILL: &str = "\
[LAW 1] INFORMATION IS FREE: ViewNode and Search cost ZERO. ALWAYS research before investing.\n\
[LAW 2] ONLY INVESTMENT IS RISK: The ONLY action that burns coins is Invest.\n\
[LAW 3] KELLY CRITERION: NEVER go all-in! Start small (10-100). Save large bets for high-confidence steps.\n\
Balance < 1.0 = PERMANENT DEATH. Bad step = investment BURNED.\n";

#[tokio::main]
async fn main() {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    info!("=== ζ-Sum Regularization Proof (Actor Model) ===");
    info!("N={}, Max Transactions={}", SWARM_SIZE, MAX_TRANSACTIONS);

    // --- Initialize Bus + Tools ---
    let kernel = Kernel::new();
    let mut bus = TuringBus::new(kernel);
    bus.mount_tool(Box::new(ThermodynamicHeartbeatTool::new(1)));
    bus.mount_tool(Box::new(AntiZombiePruningTool::new(3)));
    // Superfluid clearing: Arbitrator removed. Heartbeat(1) ensures MapReduce every append.
    bus.mount_tool(Box::new(WalletTool::new()));
    bus.mount_tool(Box::new(MathStepMembrane::new()));

    let agent_ids: Vec<String> = (0..100).map(|i| format!("Agent_{}", i)).collect();
    bus.init_problem(&agent_ids);

    // --- Create Channels ---
    let (tx_state, rx_state) = watch::channel(bus.get_immutable_snapshot());
    let (tx_mempool, mut rx_mempool) = mpsc::channel::<MinerTx>(1000);

    // --- Build Clients (3 species) ---
    let sf_url = "https://api.siliconflow.cn/v1/chat/completions";
    let ds_url = "https://api.deepseek.com/chat/completions";
    let key_sf = std::env::var("SILICONFLOW_API_KEY").expect("SILICONFLOW_API_KEY required");
    let key_sf2 = std::env::var("SILICONFLOW_API_KEY_SECONDARY").unwrap_or_else(|_| key_sf.clone());
    let key_ds = std::env::var("DEEPSEEK_API_KEY").unwrap_or_else(|_| key_sf.clone());

    let clients: Vec<Arc<ResilientLLMClient>> = vec![
        Arc::new(ResilientLLMClient::with_key(ds_url, "deepseek-chat", &key_ds)),
        Arc::new(ResilientLLMClient::with_key(ds_url, "deepseek-reasoner", &key_ds)),
        Arc::new(ResilientLLMClient::with_key(sf_url, "Pro/deepseek-ai/DeepSeek-R1", &key_sf2)),
    ];
    info!("Worker: DeepSeek-V3.2 (deepseek-chat) | Scholar: deepseek-reasoner | Explorer: R1");

    // --- Search Tool (for free queries) ---
    let search_tool = Arc::new(SearchTool::new(vec![
        "/Users/zephryj/projects/turingosv3/experiments/minif2f_data_lean4/.lake/packages/mathlib/Mathlib".to_string(),
        "/home/zephryj/projects/turingosv3/experiments/minif2f_data_lean4/.lake/packages/mathlib/Mathlib".to_string(),
    ]));

    // --- Spawn N Agent Loops ---
    for i in 0..SWARM_SIZE {
        let client = clients[i % clients.len()].clone();
        let mut rx = rx_state.clone();
        let tx = tx_mempool.clone();
        let problem = PROBLEM.to_string();
        let skill = SKILL.to_string();
        let search = search_tool.clone();
        let private_ctx = Arc::new(Mutex::new(String::new()));

        info!(">>> [SPAWN] Agent {} → {}", i, client.model_name());

        tokio::spawn(async move {
            let agent_name = format!("Agent_{}", i);
            loop {
                // 1. Read snapshot (lock-free)
                let snapshot = rx.borrow().clone();

                // 2. Check bankruptcy
                let balance = snapshot.balances.get(&agent_name).copied().unwrap_or(0.0);
                if balance < 1.0 {
                    // Wait for universe change (maybe rebirth)
                    if rx.changed().await.is_err() { break; }
                    continue;
                }

                // 3. Build prompt
                let (chain, parent_id) = build_chain_from_snapshot(&snapshot, &problem);
                let private = private_ctx.lock().unwrap().clone();
                let graveyard = snapshot.tombstones.values()
                    .take(3)
                    .cloned()
                    .collect::<Vec<_>>()
                    .join("\n");

                let p = prompt::build_agent_prompt(
                    &chain,
                    &skill,
                    &snapshot.market_ticker,
                    &format!("{}\n{}", graveyard, private),
                    balance,
                    "invest: {\"tool\":\"invest\",\"tactic\":\"your step\",\"amount\":PRICE}\nsearch: {\"tool\":\"search\",\"query\":\"term\"} (FREE)\nview: {\"tool\":\"view_node\",\"query\":\"node_id\"} (FREE)",
                );

                // 4. Invoke LLM
                let temp = 0.2 + 0.6 * (i as f32 / SWARM_SIZE.max(1) as f32);
                match client.resilient_generate(&p, i, temp).await {
                    Ok(raw) => {
                        if let Some(action) = parse_agent_output(&raw) {
                            match action.tool.as_str() {
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
                                "search" => {
                                    let q = action.query.unwrap_or_default();
                                    let result = search.search(&q);
                                    *private_ctx.lock().unwrap() = result;
                                    info!(">>> [SEARCH] {} free query: '{}'", agent_name, q);
                                }
                                "view_node" => {
                                    let id = action.query.unwrap_or_default();
                                    let snap = rx.borrow().clone();
                                    let result = view_node(&snap, &id);
                                    *private_ctx.lock().unwrap() = result;
                                    info!(">>> [VIEW] {} views node: '{}'", agent_name, id);
                                }
                                _ => {
                                    info!(">>> [OBSERVE] {} free observation.", agent_name);
                                }
                            }
                        }
                    }
                    Err(_) => { /* harness: retry on next snapshot */ }
                }

                // 5. Wait for universe update
                if rx.changed().await.is_err() { break; }
            }
        });
    }
    drop(tx_mempool); // reactor keeps the receiver

    let agent_names: Vec<String> = (0..SWARM_SIZE).map(|i| format!("Agent_{}", i)).collect();

    info!(">>> TuringOS v3 Actor Model Booted. {} agents running independently. <<<", SWARM_SIZE);

    // --- Reactor Loop (single-threaded, serial, consistent) ---
    // Superfluid reactor with 30s timeout for deadlock detection + generation rebirth
    let mut tx_count: u64 = 0;
    let mut generation: u32 = 1;
    let mut consecutive_timeouts: u32 = 0;

    loop {
        match tokio::time::timeout(
            tokio::time::Duration::from_secs(30),
            rx_mempool.recv()
        ).await {
            Ok(Some(tx)) => {
                consecutive_timeouts = 0;
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

                match bus.append(file) {
                    Ok(_) => {
                        info!("[Tx {}] {} ({}) → Appended", tx_count, tx.agent_id, tx.model_name);
                        bus.tick_map_reduce();

                        if tx.payload.contains("[OMEGA]") || tx.payload.contains("[COMPLETE]") {
                            let file_id = format!("tx_{}_by_{}", tx_count, tx.agent_id.replace("Agent_", ""));
                            info!("OMEGA at Tx {}!", tx_count);
                            bus.halt_and_settle(&file_id);
                            break;
                        }
                    }
                    Err(e) => {
                        let preview: String = tx.payload.chars().take(100).collect();
                        warn!("[Tx {}] {} REJECTED: {} | {}", tx_count, tx.agent_id, e, preview.replace('\n', " "));
                    }
                }

                let _ = tx_state.send(bus.get_immutable_snapshot());

                if tx_count >= MAX_TRANSACTIONS {
                    info!("Max transactions ({}) reached.", MAX_TRANSACTIONS);
                    break;
                }
            }
            Ok(None) => {
                info!("All senders dropped. Universe terminated.");
                break;
            }
            Err(_) => {
                // Timeout — check for market collapse (all agents bankrupt)
                consecutive_timeouts += 1;

                let solvent_count = agent_names.iter()
                    .filter(|name| bus.get_agent_balance(name) >= 1.0)
                    .count();

                if solvent_count == 0 || consecutive_timeouts >= 2 {
                    error!("==== [MACROECONOMICS] MARKET COLLAPSE! Generation {} perished. Solvent: {}/{} ====",
                        generation, solvent_count, SWARM_SIZE);

                    // Record deaths in graveyard
                    for name in &agent_names {
                        let bal = bus.get_agent_balance(name);
                        if bal < 1.0 {
                            bus.graveyard.record_death("root",
                                &format!("Gen {} bankrupt: {} (bal: {:.2})", generation, name, bal));
                        }
                    }

                    // Generation rebirth: Chapter 11 reorganization
                    generation += 1;
                    info!(">>> [REBIRTH] Spawning Generation {} with fresh capital!", generation);
                    for name in &agent_names {
                        bus.fund_agent(name, 10000.0);
                    }

                    // Broadcast new snapshot — wakes all blocked agents instantly
                    let _ = tx_state.send(bus.get_immutable_snapshot());
                    consecutive_timeouts = 0;
                } else {
                    info!("[TIMEOUT] 30s idle. Solvent: {}/{}. Waiting...", solvent_count, SWARM_SIZE);
                }
            }
        }
    }

    // --- Final Output ---
    info!("==== EVALUATION COMPLETE ({} tx, {} generations) ====", tx_count, generation);
    bus.kernel.hayekian_map_reduce();

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

    info!("--- TAPE ({} nodes) ---", bus.kernel.tape.files.len());
    for (id, f) in &bus.kernel.tape.files {
        info!("{} | P:{:.0} | {}", id, f.price,
            f.payload.chars().take(100).collect::<String>().replace('\n', " "));
    }
}
