# 架构师指令 0x00：粉碎"贪婪独裁"与"流动性陷阱"，重启纯粹的物理与演化法则

**Date**: 2026-03-23
**Context**: zeta_sum_proof Run 1 实盘审计 — 35min, 51 append, 0 OMEGA, 全体破产停滞
**Verdict**: 无锁物理底座坚不可摧，应用层犯下"中央计划经济"错误

---

## 猛药一 [P1 修复]：粉碎星形拓扑，召回"玻尔兹曼概率云" (The Boltzmann Resurrection)

**病理**: `actor.rs` 写死 `let parent_id = best_node.id.clone()` — 贪婪独裁 (Greedy Dictatorship)。
价格高 ≠ 绝对真理。强制所有 Agent 只买最贵节点 → 星形拓扑，浅层内卷。

**处方**: 废除 `best_node` 强绑定。Actor 获取快照后，通过温度参数 T 进行 Boltzmann 轮盘赌采样：

```rust
use rand::distributions::{WeightedIndex, Distribution};
use std::f64::consts::E;

pub fn softmax_route(snapshot: &UniverseSnapshot, temperature: f64) -> String {
    let leaves = snapshot.get_active_frontiers();
    if leaves.is_empty() { return "genesis".to_string(); }

    let max_price = leaves.iter().map(|n| n.market_price).fold(f64::NEG_INFINITY, f64::max);

    let mut weights = Vec::with_capacity(leaves.len());
    for node in &leaves {
        let w = E.powf((node.market_price - max_price) / temperature);
        weights.push(w.max(1e-9));
    }

    let dist = WeightedIndex::new(&weights).expect("Softmax collapsed");
    let mut rng = rand::thread_rng();
    let selected = &leaves[dist.sample(&mut rng)];

    log::info!(">>> [BOLTZMANN] Selected Node {} (Price: {:.2}) as parent.", selected.id, selected.market_price);
    selected.id.clone()
}
```

**涌现预期**: 大热节点卡死时，温度 T 让资金漫溢到未探索深层分支。星形→深空大树。

---

## 猛药二 [P0 修复]：破除死锁！引入"熊彼特破坏"与"拉马克世代交替"

**病理**: 全体破产 → `rx.changed().await` 永久死锁。无人发交易 → reactor 无 input → 无新 snapshot → 死锁。
哈耶克不救济！但必须执行"第十一章破产重组 (Chapter 11 Reorganization)"：
旧世代物理死亡 → Graveyard DNA 刻录 → 新世代 (Generation N+1) 携带遗传记忆重生。

**处方**: Tokio 超时机制 + 无锁广播唤醒：

```rust
let mut consecutive_timeouts = 0;
let mut generation = 1;

loop {
    match tokio::time::timeout(Duration::from_secs(30), rx_miner.recv()).await {
        Ok(Some(miner_tx)) => {
            consecutive_timeouts = 0;
            // ... 正常串行处理 ...
            let _ = tx_state.send(bus.read().await.get_immutable_snapshot());
        },
        Ok(None) => break,
        Err(_) => {
            consecutive_timeouts += 1;
            let mut b = bus.write().await;
            let solvent = agents.iter().filter(|a| b.get_agent_balance(&a.id) >= 1.0).count();

            if solvent == 0 || consecutive_timeouts >= 2 {
                log::error!("==== [MACROECONOMICS] MARKET COLLAPSE! Gen {} perished. ====", generation);
                for a in &agents { b.execute_autopsy_to_skill_path(&a.id); }
                generation += 1;
                for a in &agents { b.fund_agent(&a.id, 10000.0); }
                let _ = tx_state.send(b.get_immutable_snapshot());
                consecutive_timeouts = 0;
            }
        }
    }
}
```

---

## 猛药三 [P3 修复]：废除阈值大坝，开启"超流体实时清算" (Superfluid Clearing)

**病理**: `OverwhelmingGapArbitrator` 要求暴涨 1.5x 才触发 MapReduce。价格到 9000 后冻结（需 13500 才触发）。
现代架构下 MapReduce 时间复杂度 O(V+E) 极微小，无需"涨停板"。

**处方**: 斩首 Arbitrator 阈值！每笔合法投资后立即触发全局市场清算：

```rust
if successfully_invested {
    // 删除旧的 if (current_max >= last_max * 1.5) 阈值判断
    self.kernel.hayekian_map_reduce();
    log::debug!(">>> [MARKET CLEARING] Real-time Time-Arrow MapReduce executed.");
}
```

---

## 猛药四 [P2, P4-P6 修复]：语义正名与"大宪章凯利公式"强注入

### 4a. Wallet 语义精准化 (`wallet.rs`)

**病理**: Balance: 5990 被报为 "Bankrupt" — 日志污染。

**处方**: 区分真破产 vs 杠杆超标：
```rust
if balance < amount {
    if balance < 1.0 {
        return ToolSignal::Veto(format!("Bankrupt: Balance is 0.00. You are liquidated."));
    } else {
        return ToolSignal::Veto(format!(
            "Margin Call: Insufficient liquidity. Balance: {:.2}, Requested: {:.2}. Reduce your stake!",
            balance, amount
        ));
    }
}
```

### 4b. System Prompt 强化 (`skills/economic_operative.md`)

**病理**: Agent 不理解信息免费 + 投资收费，梭哈暴毙，不用 Search/View。

**处方**: 注入凯利公式风险管理思想钢印：
- LAW 1: INFORMATION IS 100% FREE — MUST ViewNode/SearchGraveyard BEFORE investing
- LAW 2: ONLY INVESTMENT IS RISK
- THE KELLY CRITERION: NEVER go all-in. Start small (12.3, 55.0). Save capital for final OMEGA push.

---

## 架构师验收誓言

四大修复不新增任何同步锁。仅通过恢复热力学概率、剥离价格阻断阈值、引入世代交替生命周期，对齐三大立法与反奥利奥架构。
