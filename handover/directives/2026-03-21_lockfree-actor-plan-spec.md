# PLAN & SPEC: 无锁 Actor Model + 奥地利学派重构

**Date**: 2026-03-21
**Architect Directive**: `2026-03-21_lockfree-austrian-naked-swarm.md`
**Scope**: Core SDK + zeta_sum_proof 首发实验
**原则**: 先在 zeta_sum_proof 原型验证，验证后提升到 Core SDK

---

## 架构变更总览

```
BEFORE (batch synchronous):
  evaluator loop {
    build Input (balances, tombstones, tape)
    output = agent.delta(&input)      ← BLOCKS: spawn 15 agents, wait ALL, drain
    bus.append(output)                 ← serial
    tick_map_reduce()
  }

AFTER (continuous asynchronous):
  // N agent loops run independently forever
  agent_loop(agent_id) {
    snapshot = rx_state.borrow()       ← instant, lock-free
    payload = invoke_llm(snapshot)     ← 1 sec or 30 min, doesn't matter
    tx_mempool.send(payload)           ← non-blocking submit
    rx_state.changed().await           ← wait for universe update
  }

  // Single-threaded reactor processes submissions FIFO
  reactor_loop {
    tx = rx_mempool.recv()             ← next submission
    bus.append(tx)                     ← serial, consistent
    tx_state.send(new_snapshot)        ← instant broadcast to all agents
  }
```

**核心收益**:
- 慢模型 (R1, 20min) 和快模型 (32B, 30s) 完美共存，互不阻塞
- 无 drain timeout 问题（因为没有 drain）
- 无 collect-all 瓶颈
- "步" 的概念消失 → 变成连续的交易流
- 并发开销趋近于零

---

## Phase 0: Core Infrastructure (底层白盒)

### 0.1: Immutable Snapshot Type
**文件**: `src/sdk/snapshot.rs` (NEW)

```rust
/// 宇宙的不可变快照 — 过去的时空是固化的
#[derive(Clone)]
pub struct UniverseSnapshot {
    pub tape: Tape,                              // DAG 的完整拷贝
    pub balances: HashMap<String, f64>,           // 所有 agent 余额
    pub market_ticker: String,                    // 大盘价格排行
    pub tombstones: HashMap<String, String>,      // 公墓记录
    pub frontier_order_book: String,              // Order Book (3 chains)
}
```

### 0.2: Bus Snapshot Method
**文件**: `src/bus.rs` — 新增方法

```rust
impl TuringBus {
    pub fn get_immutable_snapshot(&self) -> UniverseSnapshot {
        let mut balances = HashMap::new();
        for i in 0..100 {
            let aid = format!("Agent_{}", i);
            balances.insert(aid.clone(), self.get_agent_balance(&aid));
        }
        let mut tombstones = HashMap::new();
        for id in self.kernel.tape.files.keys() {
            let g = self.get_tombstones(id);
            if !g.is_empty() { tombstones.insert(id.clone(), g); }
        }
        // root tombstones
        let rg = self.get_tombstones("root");
        if !rg.is_empty() { tombstones.insert("root".to_string(), rg); }

        UniverseSnapshot {
            tape: self.kernel.tape.clone(),
            balances,
            market_ticker: self.kernel.get_market_ticker(3),
            tombstones,
            frontier_order_book: String::new(), // built by experiment layer
        }
    }
}
```

**Layer 1 检查**: 不触及 kernel 内部逻辑，只是新的读取接口 → ✅

---

## Phase 1: Actor Runtime (Core SDK)

### 1.1: MinerTx 类型
**文件**: `src/sdk/actor.rs` (NEW)

```rust
/// 矿工提交的交易 — 从 Agent 到 Reactor
pub struct MinerTx {
    pub agent_id: String,
    pub model_name: String,
    pub payload: String,           // raw LLM output
    pub parent_id: Option<String>, // which node this builds on
}
```

### 1.2: Agent Loop Function
**文件**: `src/sdk/actor.rs`

```rust
pub async fn agent_loop(
    agent_id: usize,
    client: Arc<ResilientLLMClient>,
    mut rx_state: watch::Receiver<UniverseSnapshot>,
    tx_mempool: mpsc::Sender<MinerTx>,
    problem: String,
    skill: String,
    private_context: Arc<Mutex<String>>,  // per-agent private buffer
) {
    let supervisor = AgentSupervisor::new(agent_id, 100);
    loop {
        // 1. Read snapshot (lock-free, instant)
        let snapshot = rx_state.borrow_and_update().clone();

        // 2. Check if this agent is bankrupt
        let balance = snapshot.balances.get(&format!("Agent_{}", agent_id))
            .copied().unwrap_or(0.0);
        if balance < 1.0 {
            // Wait for rebirth (balance refill via redistribution)
            let _ = rx_state.changed().await;
            continue;
        }

        // 3. Build prompt from snapshot + private context
        let chain = build_proof_chain(&snapshot, &problem);
        let private = private_context.lock().unwrap().clone();
        let prompt = build_agent_prompt(&chain, &skill, &snapshot, balance, &private);

        // 4. Invoke LLM (can take 1 sec or 30 min — doesn't block anyone)
        let temp = supervisor.apply_cognitive_divergence(0.5);
        match client.resilient_generate(&prompt, agent_id, temp).await {
            Ok(raw) => {
                if let Some(action) = parse_agent_output(&raw) {
                    match action.tool.as_str() {
                        "invest" => {
                            tx_mempool.send(MinerTx {
                                agent_id: format!("Agent_{}", agent_id),
                                model_name: client.model_name().to_string(),
                                payload: /* reconstruct payload */,
                                parent_id: /* selected frontier node */,
                            }).await.ok();
                        }
                        "search" => {
                            // Execute search, inject into PRIVATE context only
                            let result = search_tool.search(&action.query.unwrap_or_default());
                            *private_context.lock().unwrap() = result;
                            // Don't submit to mempool — free action
                        }
                        _ => { /* observe — do nothing, wait for next snapshot */ }
                    }
                }
            }
            Err(_) => { /* harness handles retries */ }
        }

        // 5. Wait for universe to update before next iteration
        let _ = rx_state.changed().await;
    }
}
```

### 1.3: Reactor Loop
**文件**: 实验层 `evaluator.rs`

```rust
// [顶层白盒：单线程事件反应堆]
let mut kernel_steps = 0u64;
while let Some(tx) = rx_mempool.recv().await {
    kernel_steps += 1;

    // 1. Parse and construct File
    let file = File { ... from tx ... };

    // 2. bus.append (serial, consistent — WalletTool → Membrane → Kernel)
    match bus.append(file) {
        Ok(_) => {
            info!("[Tx {}] Appended: {} by {}", kernel_steps, tx.agent_id, tx.model_name);
            bus.tick_map_reduce();

            // OMEGA check
            if payload.contains("[OMEGA]") {
                bus.halt_and_settle(&file_id);
                break;
            }
        }
        Err(e) => {
            warn!("[Tx {}] REJECTED: {}", kernel_steps, e);
        }
    }

    // 3. Broadcast new snapshot to all agents (O(1), non-blocking)
    tx_state.send(bus.get_immutable_snapshot()).unwrap();

    // 4. Max transactions check
    if kernel_steps >= MAX_KERNEL_STEPS { break; }
}
```

---

## Phase 2: zeta_sum_proof 重构

### 2.1: evaluator.rs 重写
**变更**: main() 变成 async，使用 tokio runtime

```rust
#[tokio::main]
async fn main() {
    // 1. Initialize bus, tools, wallets (same as before)
    // 2. Create channels
    let (tx_state, rx_state) = watch::channel(bus.get_immutable_snapshot());
    let (tx_mempool, mut rx_mempool) = mpsc::channel::<MinerTx>(1000);

    // 3. Spawn N agent loops
    for i in 0..SWARM_SIZE {
        let client = clients[i % clients.len()].clone();
        let rx = rx_state.clone();
        let tx = tx_mempool.clone();
        let private = Arc::new(Mutex::new(String::new()));
        tokio::spawn(agent_loop(i, client, rx, tx, PROBLEM.to_string(), skill.clone(), private));
    }
    drop(tx_mempool); // reactor owns the last sender

    // 4. Run reactor
    // (code from Phase 1.3 above)
}
```

### 2.2: swarm.rs → 删除或大幅简化
**变更**: `SpeculativeSwarmAgent` 和 `AIBlackBox::delta()` 被 Actor Model 替代

`swarm.rs` 的核心逻辑（Boltzmann 路由、prompt 构建、tactic 解析）拆分为独立函数供 `agent_loop` 调用。`SpeculativeSwarmAgent` struct 不再需要。

### 2.3: 保留不变的组件
- `math_membrane.rs` — 不变
- `harness.rs` — AgentSupervisor 被 agent_loop 内部使用
- `wal.rs` — 不变
- Core SDK (protocol.rs, prompt.rs, search.rs, wallet.rs, bus.rs, kernel.rs) — 不变

---

## Phase 3: 私有 Agent 上下文 (草稿本)

### 3.1: Per-Agent Private Buffer
每个 agent_loop 持有 `Arc<Mutex<String>>` 作为私有上下文。

**免费工具结果 → 私有注入**:
- SearchTool 结果 → 仅写入请求者的 private_context
- PythonSandbox 结果 → 仅写入请求者的 private_context
- 其他 agent 永远看不到这些结果

**投资行为 → 公开写入 DAG**:
- 投资的 tactic 通过 mempool → reactor → bus.append → 公开
- 所有 agent 通过下一个 snapshot 看到

**对齐检查**:
- 大宪章 Law 1: 免费工具私有 = 思考免费 ✅
- 大宪章 Law 2: 投资是唯一公开成本 ✅
- 反奥利奥: 私有上下文是 agent 的自主选择，不是系统干预 ✅

---

## Phase 4: 裸核盲测 (Future — 延后到 Actor Model 验证后)

### 概念设计 (不在本次实施)
- `experiments/naked_swarm/` — 新实验
- `MockMembrane` — 纯 Rust 图迷宫判定器
- Agent prompt: 零领域知识，只知道目标是 [OMEGA]
- 验证目标: 群体智能涌现独立于大模型能力

---

## Layer 1 合规验证

| 不变量 | Phase 0 | Phase 1 | Phase 2 | Phase 3 |
|--------|---------|---------|---------|---------|
| kernel.rs 零领域知识 | ✅ | ✅ | ✅ | ✅ |
| Append-Only DAG | ✅ | ✅ (reactor 串行写入) | ✅ | ✅ |
| SKILL-only reward | ✅ | ✅ | ✅ | ✅ |
| 投资 >= 1.0 | ✅ | ✅ | ✅ | ✅ (免费工具不投资) |

## 反奥利奥合规

| 检查 | 状态 |
|------|------|
| Rust 不干预 LLM 策略 | ✅ agent_loop 只做 IO routing |
| 数值由 LLM 决定 | ✅ Amount = agent 自由定价 |
| 无认知溢价 | ✅ 系统不补贴思考时间 |
| 私有上下文 = agent 主权 | ✅ 草稿本结果不泄漏 |

## 实施顺序

```
Phase 0 (snapshot.rs + bus method)    → Core SDK, ~50 行
Phase 1 (actor.rs)                    → Core SDK, ~150 行
Phase 2 (zeta_sum_proof refactor)     → 实验层, ~200 行重写
Phase 3 (private context)             → agent_loop 内, ~30 行
Phase 4 (naked swarm)                 → 延后
```

## 关键决策点

1. **AIBlackBox trait**: 是否废弃？
   - 建议: 保留作为 legacy 兼容层 (minif2f/zeta_regularization 继续用)
   - 新实验用 Actor Model

2. **Snapshot 频率**: 每次 append 后广播 vs 批量?
   - 建议: 每次 append 后立即广播 (事件驱动，最小延迟)

3. **Agent 数量**: 固定 N 还是动态?
   - 建议: 固定 N (tokio::spawn 一次，死循环)，破产 agent 在 loop 内 sleep 等待 rebirth

4. **MAX_KERNEL_STEPS 语义变更**: 从"swarm 轮次"变为"总交易数"
   - 100 步 × 15 agents = 最多 1500 transactions (但实际更少，因为 search/observe 不提交)
