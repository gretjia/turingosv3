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
use turingosv3::bus::{TuringBus, ThermodynamicHeartbeatTool};
use zeta_sum_proof::math_membrane::MathStepMembrane;

const SWARM_SIZE: usize = 90;
const MAX_TRANSACTIONS: u64 = 6000;

// Architect 2026-04-01: Role trifecta (scaled to 90)
const MATH_COUNT: usize = 30;  // Agent 0-29:   proof builders
const BULL_COUNT: usize = 30;  // Agent 30-59:  YES investors (做多)
const BEAR_COUNT: usize = 30;  // Agent 60-89:  NO shorters (做空)

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

fn epoch_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[tokio::main]
async fn main() {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    info!("=== ζ-Sum Regularization Proof — Role Trifecta (5M/5B+/5B-) ===");
    info!("N={}, Max Tx={}", SWARM_SIZE, MAX_TRANSACTIONS);

    // --- Initialize Bus + Tools ---
    let kernel = Kernel::new();
    let mut bus = TuringBus::new(kernel);
    bus.mount_tool(Box::new(ThermodynamicHeartbeatTool::new(1)));
    bus.mount_tool(Box::new(AntiZombiePruningTool::new(3)));
    bus.mount_tool(Box::new(WalletTool::new()));
    bus.mount_tool(Box::new(MathStepMembrane::new()));

    let agent_ids: Vec<String> = (0..SWARM_SIZE).map(|i| format!("Agent_{}", i)).collect();
    bus.init_problem(&agent_ids);

    // --- Create Channels ---
    let mut init_snap = bus.get_immutable_snapshot();
    init_snap.generation = 1;
    let (tx_state, rx_state) = watch::channel(init_snap);
    let (tx_mempool, mut rx_mempool) = mpsc::channel::<MinerTx>(1000);

    // --- Build Client: WEAK MODEL (Architect: 从 Qwen3.5-9B 开始) ---
    let sf_url = "https://api.siliconflow.cn/v1/chat/completions";
    let key_sf = std::env::var("SILICONFLOW_API_KEY").expect("SILICONFLOW_API_KEY required");

    let clients: Vec<Arc<ResilientLLMClient>> = vec![
        Arc::new(ResilientLLMClient::with_key(sf_url, "Pro/Qwen/Qwen2.5-7B-Instruct", &key_sf)),
    ];
    info!("Weak model: Pro/Qwen2.5-7B-Instruct (SiliconFlow Pro) — all 15 agents same model");

    // --- Search Tool ---
    let search_tool = Arc::new(SearchTool::new(vec![
        "/home/zephryj/projects/turingosv3/experiments/minif2f_data_lean4/.lake/packages/mathlib/Mathlib".to_string(),
    ]));

    // --- Free Action Heartbeat ---
    let free_action_epoch = Arc::new(AtomicU64::new(epoch_secs()));

    // --- Per-agent rejection feedback ---
    let agent_rejections: Arc<Mutex<HashMap<String, String>>> = Arc::new(Mutex::new(HashMap::new()));

    // --- Engine 4: Seed Role Trifecta ---
    let skills_dir = std::env::var("AGENT_SKILLS_DIR")
        .unwrap_or_else(|_| "/tmp/turingos_zeta_skills".to_string());
    for i in 0..SWARM_SIZE {
        let agent_dir = format!("{}/agent_{}", skills_dir, i);
        let _ = std::fs::create_dir_all(&agent_dir);

        let role_path = format!("{}/learned.md", agent_dir);
        let (role_name, role_content) = if i < MATH_COUNT {
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
        } else if i < MATH_COUNT + BULL_COUNT {
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
    info!(">>> [ENGINE 4] Role trifecta: {}M/{}B+/{}B-", MATH_COUNT, BULL_COUNT, BEAR_COUNT);

    // --- Spawn Agent Loops ---
    for i in 0..SWARM_SIZE {
        let client = clients[i % clients.len()].clone();
        let mut rx = rx_state.clone();
        let tx = tx_mempool.clone();
        let problem = PROBLEM.to_string();
        let skill = SKILL.to_string();
        let search = search_tool.clone();
        let private_ctx = Arc::new(Mutex::new(String::new()));
        let heartbeat = free_action_epoch.clone();
        let agent_skill_dir = format!("{}/agent_{}", skills_dir, i);
        let rejections = agent_rejections.clone();

        let agent_role = if i < MATH_COUNT { "MATH" }
            else if i < MATH_COUNT + BULL_COUNT { "BULL" }
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

                // Read last rejection feedback
                let last_rejection = rejections.lock().unwrap()
                    .get(&agent_name).cloned().unwrap_or_default();
                let feedback = if last_rejection.is_empty() {
                    String::new()
                } else if last_rejection.contains("FRONT-RUNNING") {
                    format!("\n=== YOUR LAST SUBMISSION WAS REJECTED (TOO LONG) ===\n{}\n=== Write ONE short atomic math argument. ===\n", last_rejection)
                } else {
                    format!("\n=== YOUR LAST SUBMISSION WAS REJECTED ===\n{}\n===\n", last_rejection)
                };

                let p = prompt::build_agent_prompt(
                    &chain,
                    &format!("{}\n{}", skill, agent_skill),
                    &snapshot.market_ticker,
                    &format!("{}\n{}\n{}", graveyard, private, feedback),
                    balance,
                    "append: {\"tool\":\"append\",\"tactic\":\"your step\"} (FREE)\ninvest: {\"tool\":\"invest\",\"tactic\":\"your step\",\"amount\":PRICE} (creates node + buys YES)\nbet: {\"tool\":\"invest\",\"node\":\"node_id\",\"amount\":PRICE} (buy YES on existing)\nshort: {\"tool\":\"short\",\"node\":\"node_id\",\"amount\":PRICE} (buy NO)\nsearch: {\"tool\":\"search\",\"query\":\"term\"} (FREE)\nview: {\"tool\":\"view_node\",\"query\":\"node_id\"} (FREE)",
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

                    let role_bias = if i < MATH_COUNT {
                        "You are a proof builder. Only invest when you see clearly correct or flawed steps. Prefer PASS if unsure."
                    } else if i < MATH_COUNT + BULL_COUNT {
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

    let agent_names: Vec<String> = (0..SWARM_SIZE).map(|i| format!("Agent_{}", i)).collect();
    info!(">>> TuringOS v3 ζ-Sum Booted. {} agents (5M/5B+/5B-). <<<", SWARM_SIZE);

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
                        // Feed rejection back to agent
                        agent_rejections.lock().unwrap()
                            .insert(tx.agent_id.clone(), format!("{}", e));
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
                        reason, generation, solvent_count, SWARM_SIZE);

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
        let _ = writeln!(f, "**Model**: Pro/Qwen2.5-7B-Instruct (SiliconFlow Pro) — all agents same model\n");
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
