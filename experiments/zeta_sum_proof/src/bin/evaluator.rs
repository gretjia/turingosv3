use log::{info, warn};
use std::sync::{Arc, Mutex};
use tokio::sync::{mpsc, watch};
use turingosv3::kernel::{File, Kernel};
use turingosv3::drivers::llm_http::ResilientLLMClient;
use turingosv3::sdk::tools::wallet::WalletTool;
use turingosv3::sdk::tool::{AntiZombiePruningTool, OverwhelmingGapArbitratorTool};
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

const SKILL: &str = "Balance < 1.0 = YOU DIE.\nBad step = investment BURNED.\nPrice your investment based on your confidence.\n";

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
    bus.mount_tool(Box::new(OverwhelmingGapArbitratorTool::new(1.5)));
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
        Arc::new(ResilientLLMClient::with_key(sf_url, "deepseek-ai/DeepSeek-R1-Distill-Qwen-32B", &key_sf)),
        Arc::new(ResilientLLMClient::with_key(ds_url, "deepseek-reasoner", &key_ds)),
        Arc::new(ResilientLLMClient::with_key(sf_url, "Pro/deepseek-ai/DeepSeek-R1", &key_sf2)),
    ];
    info!("Miner: R1-Distill-32B | Scholar: deepseek-reasoner | Explorer: R1");

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

    info!(">>> TuringOS v3 Actor Model Booted. {} agents running independently. <<<", SWARM_SIZE);

    // --- Reactor Loop (single-threaded, serial, consistent) ---
    let mut tx_count: u64 = 0;
    while let Some(tx) = rx_mempool.recv().await {
        tx_count += 1;

        // Build file from MinerTx
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

                if tx.payload.contains("[OMEGA]") {
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

        // Broadcast new snapshot
        let _ = tx_state.send(bus.get_immutable_snapshot());

        if tx_count >= MAX_TRANSACTIONS {
            info!("Max transactions ({}) reached.", MAX_TRANSACTIONS);
            break;
        }
    }

    // --- Final Output ---
    info!("==== EVALUATION COMPLETE ({} transactions) ====", tx_count);
    bus.kernel.hayekian_map_reduce();

    // Trace golden path
    if let Some(omega) = bus.kernel.tape.files.values().find(|f| f.payload.contains("[OMEGA]")) {
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
