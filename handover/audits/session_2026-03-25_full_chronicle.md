# TuringOS v3 — Session Chronicle (2026-03-25 ~ 2026-03-27)

**Scope**: 三代经济引擎进化 (Hayekian → AMM → Polymarket) + 7 次实验运行 + 根因分析链

---

## Timeline

### Phase 1: Run 4 分析 + 架构师指令接收 (2026-03-25 早)

**`b187ba3`** — Run 4 全局节点报告

Run 4 (Hayekian 引擎) 的 26 个节点完整分析:
- Golden Path 6 步, 价格 95B-100B
- 非 GP 最高价 494 → 价格断崖 1.9 亿倍
- 4 种失败模式: 寄生、跳步、平行宇宙、迟到重复
- 14/15 agent 参与, 6 个 GP 贡献者

此时架构师提出核心批判: **hayekian_map_reduce 是 GOSPLAN 事后分配**，系统性惩罚先驱者。

### Phase 2: TuringSwap AMM 经济引擎 (2026-03-25 中)

**架构师指令**: 废除 Hayekian，引入 Uniswap V2 AMM (x*y=k)

**`59d647a`** — 指令归档 + Gemini 独立审计

Gemini 审计判定 REJECT (认为 AMM 不解决风险倒挂)。用户明确否定 Gemini 审计结论，要求按架构师指令执行。

**`9ce1639`** — feat: TuringSwap AMM 经济引擎

核心代码变更:

```diff
# src/amm.rs (NEW)
+ pub struct UniswapPool { node_id, coin_reserve, token_reserve, k }
+ impl UniswapPool { launch, get_amount_in, swap_coin_for_token, swap_token_for_coin, spot_price, inject_liquidity }

# src/kernel.rs
- pub fn hayekian_map_reduce(&mut self) { ... }    // 删除: O(V+E) 反向传播
+ pub amms: HashMap<FileId, UniswapPool>            // 新增: per-node AMM 池
+ pub bounty_escrow: f64                            // 新增: 有限创世预算
+ pub fn create_pool, buy_citation, sell_tokens, liquidate_bounty, refresh_prices

# src/bus.rs
+ Phase 2: AMM 引用购买 (引用即买入, 100 tokens)
+ Phase 5: IDO 建池 + 1000 founder tokens
+ halt_and_settle: bounty 注入 GP 池 + token 套现

# src/sdk/tools/wallet.rs
+ pub portfolios: HashMap<String, HashMap<String, f64>>

# src/sdk/snapshot.rs
+ pub struct PoolSnapshot { coin_reserve, token_reserve, spot_price, citation_cost_100 }
```

Kernel-auditor 审计: CLEAN (1 advisory — InvestOnly 直接写 intrinsic_reward → 已修复为 AMM swap + refund)。
Codex 审计: 6 findings, 全部修复。

### Phase 3: Run 5 (TuringSwap AMM 首次实战)

**`28af785`** — Run 5 分析

| 维度 | Run 4 (Hayekian) | Run 5 (AMM) |
|------|------------------|-------------|
| 交易 | 37 | 15 |
| GP 步数 | 6 | 3 |
| 时间 | ~12 min | ~8 min |
| 引用成本 | 0 | 0.56→0.57 Coins (AMM 滑点) |

Golden Path 3 步 (Agent_10 单步压缩了 Run 4 的 Step 3-6):
1. `tx_1_by_9`: 定义 S(N) + 绝对收敛
2. `tx_7_by_6`: cos→Re + 闭式
3. `tx_15_by_10`: Taylor + 长除法 + 取实部 → -1/12 [OMEGA]

**`ca989ac`** — Gemini 审计: VALID WITH MINOR GAPS (符号抄写错误不影响代数正确性)

### Phase 4: Turing-Polymarket 预测市场 (2026-03-25 晚 ~ 2026-03-26)

架构师再次升级: 废除 AMM，引入 Polymarket 二元条件代币。

核心哲学: "每个节点是绝对物理隔离的热力学孤岛，资金绝对不跨池流动"

**`94f6ae1`** — feat: Turing-Polymarket

```diff
# src/amm.rs → DELETED
# src/prediction_market.rs (NEW)
+ pub struct BinaryMarket { node_id, yes_reserve, no_reserve, k, resolved }
+ impl BinaryMarket { create, buy_yes, buy_no, resolve, redeem, yes_price, no_price }
+ 10 unit tests (split-ignition, pioneer profit, short assassin, etc.)

# src/kernel.rs
- pub amms: HashMap<FileId, UniswapPool>
- pub bounty_escrow: f64
+ pub prediction_markets: HashMap<FileId, BinaryMarket>
+ pub fn create_market, buy_yes, buy_no, resolve_market, yes_price, redeem, refresh_prices

# src/bus.rs
- Phase 2: AMM 引用购买 (引用即买入)
+ Phase 3: Citations are FREE (拓扑与金融解耦, 大宪章复辟)
- Phase 5: IDO 建池 + founder tokens
+ Phase 5: Split-Ignition (1 Coin LP + auto-long)
- halt_and_settle: bounty 注入 + token 套现
+ halt_and_settle: Oracle 二元解算 (GP→YES, 非GP→NO) + redeem

# src/sdk/tools/wallet.rs
- portfolios: HashMap<String, HashMap<String, f64>>
+ portfolios: HashMap<String, HashMap<String, (f64, f64)>>  // (yes, no)
- on_halt: global_pool 分配 (legacy 禁用, Codex double-payment fix)
```

Kernel-auditor: CLEAN。Codex: 4 findings (double-payment, re-resolve guard, overflow, stale price), 全部修复。

### Phase 5: Run 6 (Polymarket 首次实战)

**`88d8732`** — Run 6 分析

53 tx, 35 nodes, 6-step GP, **第三条独立代数路径** (对称双指数 z₁,z₂)

**`f182de7`** — Run 6 经济学深度分析

发现 **P0 致命 Bug**: `math_membrane.rs:40` 中 `reward: 100_000_000_000.0` (Hayekian 遗产) 在 Polymarket 体制下破坏零和守恒。

根因追溯:
```
MathStepMembrane 检测 [COMPLETE] → 返回 YieldReward { reward: 100B }
    ↓
bus.rs: final_reward += 100B
    ↓
Split-Ignition: LP=1, auto-long=99,999,999,999
    ↓
Agent_1 获得 ~100B YES 份额
    ↓
OMEGA 解算: 100B YES × 1 Coin = 100B Coins 凭空铸造
```

OMEGA 节点价格 90,000,000,045 (应为 ~1.0)。Polymarket 铁律 1 (1 Coin = 1 YES + 1 NO) VIOLATED。

**五层做空缺失分析**:
1. 协议层: prompt 无 short 动作
2. 信息层: 所有 P_yes≈99%, 无差异化信号
3. 激励层: LP 深度 1 Coin, 做空 ROI 表面上低
4. 认知层: LLM 偏向建设性行为
5. 铸币扭曲: 100B 使利润集中在 OMEGA 终局

**`9471d75`** — Run 6 对齐审查

```
Layer 1 Invariants:     4/4 PASS
Polymarket 4 Iron Laws: 2/4 PASS, 1 VIOLATED (conservation), 1 DEGRADED
Anti-Oreo 3 Boundaries: 2/3 PASS, 1 BUG (100B mint)
Magna Carta 3 Laws:     2/3 PASS, 1 PARTIAL
```

### Phase 6: Harness 升级 + Run 6 修复 (2026-03-26)

**`3e77554`** — harness 升级

根因: dev-cycle 只审计 `src/`, 遗漏 `experiments/` 中的 SKILL 文件。

```diff
# .claude/skills/dev-cycle/SKILL.md
+ ### 4.5. MIGRATION SCAN (mandatory when economic engine changes)
+ 1. grep -rn "YieldReward|InvestOnly|intrinsic_reward|100_000_000" experiments/ src/
+ 2. For EACH experiment: verify compatible with new economic engine

# CLAUDE.md
+ 16. 经济引擎变更时必须全仓库 grep experiments/
```

**跨节点投资缺失根因追溯** (事实证据链):

```
evaluator.rs:144 prompt 模板:
  invest: {"tool":"invest","tactic":"your step","amount":PRICE}
                                                          ↑ 无 node 字段
    ↓
LLM 输出: {"tool":"invest","tactic":"...","amount":10}  (无 node)
    ↓
evaluator.rs:156: node = action.node.unwrap_or("self")  → 永远 "self"
    ↓
WalletTool: target == "self" → YieldReward (创建新节点)
    ↓
100% SELF-INVEST
```

**结论**: 不是 LLM 行为偏差，是 prompt 模板的物理约束。Protocol 层和执行层完全就绪。

**`2cd8146`** — fix: Run 6 全部问题修复

```diff
# src/sdk/tool.rs
+ #[derive(Debug, Clone, Copy, PartialEq)]
+ pub enum BetDirection { Long, Short }
  pub enum ToolSignal {
-     InvestOnly { target_node: String, amount: f64 },
+     InvestOnly { target_node: String, amount: f64, direction: BetDirection },
  }

# src/sdk/tools/wallet.rs
+ let (real_target, direction) = if target.starts_with("SHORT:") {
+     (target["SHORT:".len()..].to_string(), BetDirection::Short)
+ } else {
+     (target.clone(), BetDirection::Long)
+ };

# src/bus.rs
+ let result = match invest_direction {
+     BetDirection::Long => self.kernel.buy_yes(&invest_target, invest_amount),
+     BetDirection::Short => self.kernel.buy_no(&invest_target, invest_amount),
+ };
+ pub ticker_top_n: usize,  // 可配置, 替代硬编码 3

# src/kernel.rs get_market_ticker
- "Market Cap: {:.2} Coins"
+ "P_yes: {:.1}%"  /  "RESOLVED: YES/NO"

# src/prediction_market.rs yes_price
+ match self.resolved {
+     Some(true) => 1.0,
+     Some(false) => 0.0,
+     None => { self.no_reserve / total }
+ }

# experiments/zeta_sum_proof/src/math_membrane.rs
- return ToolSignal::YieldReward { reward: 100_000_000_000.0 };
+ return ToolSignal::Modify(format!("{}\n  -- [OMEGA]", payload));

# experiments/zeta_sum_proof/src/bin/evaluator.rs
+ "short" match case → SHORT:<node> prefix
+ bet/short 在 prompt 模板
+ LAW 4 POLYMARKET 在 SKILL prompt
```

Kernel-auditor: CLEAN。10/10 tests PASS。

### Phase 7: Run 7 (Polymarket v2 全修复版)

**`7408c88`** — Run 7 分析

40 tx, 31 nodes, 2 generations, 4-step GP

| 指标 | Run 6 (bug) | Run 7 (fixed) |
|------|-------------|---------------|
| OMEGA 价格 | 90,000,000,045 | **1.0** ✅ |
| 系统铸币 | 100B | **0** ✅ |
| 非 GP 价格 | 1.0 | **0.0** ✅ |
| 跨节点投资 | 0 | **0** (未变) |
| 做空 | 0 | **0** (未变) |

Golden Path 4 步 (第四条独立代数路径: a=1/N + 显式 (1-i)^n 代入):
1. `tx_5_by_7` (Reasoner): 定义 + ratio test
2. `tx_21_by_3` (V3.2): z=exp(-(1-i)/N), Re(z/(1-z)²)
3. `tx_32_by_3` (V3.2): Taylor 展开 + (1-z)² 系数
4. `tx_40_by_12` (V3.2): (1-i)²=-2i 代入 → -1/12 [OMEGA]

3 条替代方法论发现: sinh 路径, 系数匹配法, 压缩路径

**对齐审查**:
```
Layer 1: 4/4 PASS
Polymarket: 3/4 PASS, 1 DEGRADED (价格无差异化)
Anti-Oreo: 3/3 PASS
Magna Carta: 3/3 PASS
vs Run 6: 铁律 1 VIOLATED→PASS, Anti-Oreo BUG→PASS, Law 2 PARTIAL→PASS
```

**bet/short 未涌现根因** (事实证据链):

1. **代码路径**: 全部可用 (buy_yes/buy_no/BetDirection typed)
2. **Prompt**: 包含 bet/short 说明 + LAW 4 POLYMARKET
3. **MathStepMembrane**: 不会阻塞跟投 (wallet 标签 62 字符 ≥ 20)

三个行为层根因:

| 根因 | 证据 | 可修复性 |
|------|------|---------|
| **信息环境均质**: 31 个 AUTO-LONG 全部 P_yes=99-100% | 日志: 所有 `(P_yes=99.0%)` 或 `(P_yes=100.0%)` | ✅ 降低 auto-long 比例 |
| **任务偏向推进**: prompt 核心指令是 "Write the NEXT step" | evaluator.rs prompt 结构 | ✅ agent 角色分化 |
| **LLM 建设性偏差**: 训练数据中无"做空学术推导"行为模式 | 推测 (无法用日志证伪) | ❌ 模型先验 |

---

## 跨 Run 代数路径对比 (反作弊证据)

| Run | 步数 | 代数路径 | 关键差异 |
|-----|------|---------|---------|
| 4 (Hayekian) | 6 | Re(单复指数), x=1/N | 显式 c₀=i/2, c₁=0, c₂=-1/12 系数 |
| 5 (AMM) | 3 | Re(单复指数), x=1/N | Agent_10 单步压缩 Taylor+除法+取 Re |
| 6 (Polymarket v1) | 6 | 对称双指数 z₁=i-1, z₂=-(i+1) | 1/z₁²+1/z₂²=0 对消 |
| 7 (Polymarket v2) | 4 | Re(单复指数), a=1/N | 显式 (1-i)²=-2i, (1-i)³=-2(1+i) 代入 |

4 次运行, 3 种不同的符号体系 (x, ε, a), 2 种不同的代数路径 (Re 单复 vs 对称双指数)。**强证据表明 agent 在做真实推理而非模板背诵。**

---

## 经济引擎进化对比

| 维度 | Hayekian (Run 4) | AMM (Run 5) | Polymarket v1 (Run 6) | Polymarket v2 (Run 7) |
|------|------------------|-------------|----------------------|----------------------|
| 定价 | gamma^depth 反向传播 | 池 coin_reserve | P_yes 概率 | P_yes 概率 |
| 引用成本 | 免费 | 0.56 Coins | 免费 | 免费 |
| 系统铸币 | 100B (无锚) | 100K bounty | 100B (bug) | **0** ✅ |
| 清算 | global_pool 按 stake 分 | bounty→池→套现 | 二元 YES/NO | 二元 YES/NO |
| 跨池流动 | 全局反向传播 | bounty 注入 | 无 (bug 除外) | **绝对零** ✅ |
| 做空 | 不可能 | 不可能 | 不可能 (协议缺失) | 可能 (未涌现) |

---

## 残留问题

| 优先级 | 问题 | 状态 |
|--------|------|------|
| P1 | 价格信号均质 (P_yes≈99%) | 根因: Split-Ignition auto-long。修复: 降低比例 |
| P1 | bet/short 未涌现 | 根因: 信息环境+任务偏向+LLM 偏差 |
| P2 | 3 个非活跃实验 100B 残留 | lean4_membrane_tool.rs × 3 (15 处) |
| P2 | OMEGA 无数学验证 | [COMPLETE] 字符串匹配, Lean 4 未实现 |
| P2 | SiliconFlow R1 API Key 失效 | 5 个 agent 401 |
| P3 | LP seed 1.0 硬编码 | bus.rs:374, 低优先级 |
| P3 | Veto 不退款 | 已存在 bug, WalletTool 扣款后被 Veto 不返还 |
