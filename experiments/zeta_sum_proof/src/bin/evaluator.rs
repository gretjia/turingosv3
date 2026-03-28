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
[LAW 1] APPEND IS FREE: Creating nodes costs ZERO. Explore freely, build the truth graph at no risk.\n\
[LAW 2] ONLY INVEST COSTS MONEY: Invest/Bet/Short are the ONLY actions that burn coins.\n\
[LAW 3] KELLY CRITERION: Start small (10-50). Save large bets for high-confidence steps. Invest >= 2 for directional bet.\n\
[LAW 4] POLYMARKET ECONOMICS:\n\
  - append: FREE node creation. Explore ideas at zero risk.\n\
  - invest: Buy YES on your own node = you believe it leads to OMEGA.\n\
  - bet: Buy YES on someone else's node = you back their work.\n\
  - short: Buy NO on any node = you believe it's a dead end. VERY PROFITABLE if you're right!\n\
  - KEY: If you spot a wrong node with high YES price, shorting it is extremely profitable.\n\
  - Your profit comes ONLY from finding mispriced probabilities.\n\
Balance < 1.0 = can only append (free). Cannot invest/bet/short.\n";

fn epoch_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[tokio::main]
async fn main() {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    info!("=== ζ-Sum Regularization Proof (Actor Model) ===");
    info!("N={}, Max Transactions={}", SWARM_SIZE, MAX_TRANSACTIONS);

    // --- Initialize Bus + Tools ---
    let kernel = Kernel::new(); // Polymarket: no bounty escrow, pure zero-sum
    let mut bus = TuringBus::new(kernel);
    bus.mount_tool(Box::new(ThermodynamicHeartbeatTool::new(1)));
    bus.mount_tool(Box::new(AntiZombiePruningTool::new(3)));
    // Superfluid clearing: Arbitrator removed. Heartbeat(1) ensures MapReduce every append.
    bus.mount_tool(Box::new(WalletTool::new()));
    bus.mount_tool(Box::new(MathStepMembrane::new()));

    let agent_ids: Vec<String> = (0..100).map(|i| format!("Agent_{}", i)).collect();
    bus.init_problem(&agent_ids);

    // --- Create Channels ---
    let mut init_snap = bus.get_immutable_snapshot();
    init_snap.generation = 1;
    let (tx_state, rx_state) = watch::channel(init_snap);
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

    // --- Free Action Heartbeat (shared atomic timestamp for liveness detection) ---
    // Agents update this on any action (Search/View/Observe). Reactor reads it to
    // distinguish "agents actively researching" from "true deadlock".
    let free_action_epoch = Arc::new(AtomicU64::new(epoch_secs()));

    // --- Spawn N Agent Loops ---
    for i in 0..SWARM_SIZE {
        let client = clients[i % clients.len()].clone();
        let mut rx = rx_state.clone();
        let tx = tx_mempool.clone();
        let problem = PROBLEM.to_string();
        let skill = SKILL.to_string();
        let search = search_tool.clone();
        let private_ctx = Arc::new(Mutex::new(String::new()));
        let heartbeat = free_action_epoch.clone();

        info!(">>> [SPAWN] Agent {} → {}", i, client.model_name());

        tokio::spawn(async move {
            let agent_name = format!("Agent_{}", i);
            let mut local_generation: u32 = 1;
            loop {
                // 1. Read snapshot (lock-free)
                let snapshot = rx.borrow().clone();

                // 1b. Detect generation change → purge phantom context
                if snapshot.generation != local_generation {
                    *private_ctx.lock().unwrap() = String::new();
                    local_generation = snapshot.generation;
                }

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
                    "append: {\"tool\":\"append\",\"tactic\":\"your step\"} (FREE — creates node, no market)\ninvest: {\"tool\":\"invest\",\"tactic\":\"your step\",\"amount\":PRICE} (creates node + buys YES)\nbet: {\"tool\":\"invest\",\"node\":\"node_id\",\"amount\":PRICE} (buy YES on existing node)\nshort: {\"tool\":\"short\",\"node\":\"node_id\",\"amount\":PRICE} (buy NO against a node)\nsearch: {\"tool\":\"search\",\"query\":\"term\"} (FREE)\nview: {\"tool\":\"view_node\",\"query\":\"node_id\"} (FREE)",
                );

                // 4. Invoke LLM
                let temp = 0.2 + 0.6 * (i as f32 / SWARM_SIZE.max(1) as f32);
                match client.resilient_generate(&p, i, temp).await {
                    Ok(raw) => {
                        if let Some(action) = parse_agent_output(&raw) {
                            match action.tool.as_str() {
                                "append" => {
                                    // FREE topology append — no wallet tag, no cost
                                    let tactic = action.tactic.unwrap_or_default();
                                    if !tactic.is_empty() {
                                        let _ = tx.send(MinerTx {
                                            agent_id: agent_name.clone(),
                                            model_name: client.model_name().to_string(),
                                            payload: tactic, // No wallet tag → WalletTool passes
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
                                        // SHORT: prefix tells WalletTool to set BetDirection::Short
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
                                    info!(">>> [SEARCH] {} free query: '{}'", agent_name, q);
                                }
                                "view_node" => {
                                    let id = action.query.unwrap_or_default();
                                    let snap = rx.borrow().clone();
                                    let result = view_node(&snap, &id);
                                    *private_ctx.lock().unwrap() = result;
                                    heartbeat.store(epoch_secs(), Ordering::Relaxed);
                                    info!(">>> [VIEW] {} views node: '{}'", agent_name, id);
                                }
                                _ => {
                                    heartbeat.store(epoch_secs(), Ordering::Relaxed);
                                    info!(">>> [OBSERVE] {} free observation.", agent_name);
                                }
                            }
                        }
                    }
                    Err(_) => { /* harness: retry on next snapshot */ }
                }

                // 5. FORCED INVESTMENT ROUND — separate LLM call for financial decisions
                // After the reasoning action, agent MUST make an investment decision.
                // This stimulates the Polymarket economy on every round.
                let snap_for_invest = rx.borrow().clone();
                let invest_balance = snap_for_invest.balances.get(&agent_name).copied().unwrap_or(0.0);
                if invest_balance >= 2.0 && !snap_for_invest.tape.files.is_empty() {
                    // Build investment-focused prompt with current DAG state
                    let node_list: String = snap_for_invest.tape.time_arrow.iter().rev().take(10)
                        .filter_map(|nid| snap_for_invest.tape.files.get(nid))
                        .map(|n| {
                            let preview: String = n.payload.chars().take(80).collect();
                            format!("[{}] P:{:.2} | {}", n.id, n.price, preview.replace('\n', " "))
                        })
                        .collect::<Vec<_>>().join("\n");

                    let invest_prompt = format!(
                        "You are an investor in a proof market. Your balance: {:.0} Coins.\n\
                        Recent nodes (most recent first):\n{}\n\n\
                        You MUST invest in ONE node. Choose the node you believe is most likely to be on the winning proof path.\n\
                        - To back a node: <action>{{\"tool\":\"invest\",\"node\":\"NODE_ID\",\"amount\":COINS}}</action>\n\
                        - To bet against a node: <action>{{\"tool\":\"short\",\"node\":\"NODE_ID\",\"amount\":COINS}}</action>\n\
                        - Minimum 2 Coins. Recommended: 5-20 Coins.\n\
                        Choose wisely — your profit depends on the final Oracle verdict.",
                        invest_balance, node_list
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
                                            info!(">>> [FORCED INVEST] {} bet YES {:.0} on {}", agent_name, amount, node);
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
                                            info!(">>> [FORCED SHORT] {} bet NO {:.0} on {}", agent_name, amount, node);
                                        }
                                    }
                                    _ => {} // skip if agent outputs non-investment action
                                }
                            }
                        }
                        Err(_) => {}
                    }
                }

                // 6. Wait for universe update
                if rx.changed().await.is_err() { break; }
            }
        });
    }
    drop(tx_mempool); // reactor keeps the receiver

    let agent_names: Vec<String> = (0..SWARM_SIZE).map(|i| format!("Agent_{}", i)).collect();

    info!(">>> TuringOS v3 Actor Model Booted. {} agents running independently. <<<", SWARM_SIZE);

    // --- Reactor Loop (single-threaded, serial, consistent) ---
    // Dual-condition rebirth: only when truly dead (solvent==0 OR absolute stagnation)
    let mut tx_count: u64 = 0;
    let mut generation: u32 = 1;
    let mut last_invest_epoch = epoch_secs();

    loop {
        match tokio::time::timeout(
            tokio::time::Duration::from_secs(30),
            rx_mempool.recv()
        ).await {
            Ok(Some(tx)) => {
                // Codex #3: only financial actions update liveness (free appends don't suppress rebirth)
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

                let mut snap = bus.get_immutable_snapshot();
                snap.generation = generation;
                let _ = tx_state.send(snap);

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
                // Timeout — dual-condition rebirth check
                let now = epoch_secs();
                let solvent_count = agent_names.iter()
                    .filter(|name| bus.get_agent_balance(name) >= 1.0)
                    .count();
                let last_free = free_action_epoch.load(Ordering::Relaxed);
                let secs_since_invest = now.saturating_sub(last_invest_epoch);
                let secs_since_free = now.saturating_sub(last_free);

                // Condition 1: True global bankruptcy
                let all_bankrupt = solvent_count == 0;
                // Condition 2: Absolute stagnation (no invest AND no free action for 60s)
                let absolute_stagnation = secs_since_invest >= 60 && secs_since_free >= 60;

                if all_bankrupt || absolute_stagnation {
                    let reason = if all_bankrupt { "Global bankruptcy" } else { "Absolute stagnation" };
                    error!("==== [MACROECONOMICS] {}! Generation {} perished. Solvent: {}/{} ====",
                        reason, generation, solvent_count, SWARM_SIZE);

                    for name in &agent_names {
                        let bal = bus.get_agent_balance(name);
                        if bal < 1.0 {
                            bus.graveyard.record_death("root",
                                &format!("Gen {} bankrupt: {} (bal: {:.2})", generation, name, bal));
                        }
                    }

                    generation += 1;
                    // Magna Carta Law 2: NO NEW MONEY after genesis.
                    info!(">>> [NO REBIRTH] Gen {} — zero new Coins. Free append only.", generation);

                    // Broadcast with generation tag — agents detect change and purge phantom context
                    let mut snap = bus.get_immutable_snapshot();
                    snap.generation = generation;
                    let _ = tx_state.send(snap);
                    last_invest_epoch = epoch_secs();
                } else {
                    // Magna Carta Law 1: respect free reading rights
                    if secs_since_free < 30 {
                        info!("[MARKET WATCH] No invest for {}s. Agents actively reading (last free {}s ago). Respecting Law 1.",
                            secs_since_invest, secs_since_free);
                    } else {
                        info!("[TIMEOUT] 30s idle. Solvent: {}/{}. Invest: {}s ago, Free: {}s ago.",
                            solvent_count, SWARM_SIZE, secs_since_invest, secs_since_free);
                    }
                }
            }
        }
    }

    // --- Final Output ---
    info!("==== EVALUATION COMPLETE ({} tx, {} generations) ====", tx_count, generation);
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
    // Summary Track: terminal-friendly truncated view
    info!("--- TAPE ({} nodes) ---", bus.kernel.tape.files.len());
    for (id, f) in &bus.kernel.tape.files {
        let char_count = f.payload.chars().count();
        let summary: String = f.payload.chars().take(150).collect::<String>().replace('\n', " ");
        let flag = if char_count > 150 { format!(" [+{}c]", char_count - 150) } else { String::new() };
        info!("{} | P:{:.0} | {}{}", id, f.price, summary, flag);
    }

    // Immutable Track: full payload dump to file (publication-grade)
    let dump_path = "/tmp/zeta_sum_tape_full.md";
    if let Ok(mut f) = std::fs::File::create(dump_path) {
        use std::io::Write;
        let _ = writeln!(f, "# zeta_sum_proof — Full Tape Dump\n");
        let _ = writeln!(f, "**Transactions**: {} | **Generations**: {} | **Nodes**: {}\n",
            tx_count, generation, bus.kernel.tape.files.len());

        // Golden Path (if OMEGA reached)
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

        // Full tape (all nodes, untruncated)
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
