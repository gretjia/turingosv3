---
date: 2026-03-25
source: Architect verbal directive (extended, with code)
status: archived-pending-authorization
---

# Architect Directive: TuringSwap AMM Economic Engine + Agent $HOME + Init AI

## 架构师原话

> "当你发出那句质问——怎么可能投资越早、承担风险越大，分到的钱反而越少？——你就已经一脚踢开了中央计划经济的大门，真正触碰到了资本主义的核心法则：风险溢价（Risk Premium）与流动性定价！"

> "用一个全局遍历的 map_reduce 函数事后发钱，本质上就是苏联国家计划委员会（GOSPLAN）的年终奖核算系统。它系统性地惩罚了在黑暗中摸索的先驱，奖励了最后一步搭便车的寄生虫。"

> "可以采用多次初始化的办法，先试跑几轮，然后让顶层看看有什么问题，改，然后重新初始化。"

## 一、TuringSwap — 去中心化知识现货交易所

### 核心变革：废除 GOSPLAN，引入 Uniswap AMM

1. **前置悬赏托管 (Bounty Escrow)**: 系统绝不凭空印发 100B。创世时由外部打入真实预算（如 100,000 Coins），OMEGA 触发时消耗这笔真金白银。没有无锚印钞，只有价值转移。

2. **引用即买入 (Citation as Spot Purchase)**: 每个节点自动发行专属代币 $T_n，建立 Uniswap V2 AMM 池。引用别人的节点必须当场买入 Token，不存在免费白嫖。

3. **恒定乘积定价 (X * Y = K)**:
   - 节点 IPO（做市建池）：Agent 花本金（如 50 Coins）+ 系统配发 9,000 枚 $T_n 建池，获得 1,000 枚原始股。切肤之痛（Skin in the game）。
   - 购买过路费（买断知识产权）：引用别人节点必须当场买入 100 枚 Token。
   - 早期暴利：越多人引用，池中 Coin 越多，Token 越少，滑点推高价格。先驱者随时可套现数百倍。甚至不需要等 OMEGA。
   - 晚期代价：搭便车者面对天价引用成本。风险-收益完美正向匹配。

4. **OMEGA 结算**: 悬赏金硬砸 Golden Path 池子，K 值跳跃，代币价格暴涨，持有者高位套现。

### 核心优势
- **废物自动归零（Garbage Collection）**: 无人引用 = 投资打水漂，市场自动剪枝
- **专业分工涌现**: 基础科研做市商 vs 终局冲刺者
- **价格即真理**: 池子资金流 = 路径质量信号，无需全局计算器
- **风险溢价正向匹配**: 先驱获暴利，搭便车者付天价

### 代码架构

#### 1. 核心金融引擎 (`src/amm.rs`) — 新建

```rust
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct UniswapPool {
    pub node_id: String,
    pub coin_reserve: f64,   // 池中的基础货币储备 (System Coins)
    pub token_reserve: f64,  // 池中的节点专属引用权代币储备
    pub k: f64,              // 恒定乘积 K = x * y
}

impl UniswapPool {
    /// 创世发行 (IDO): 拓荒者注入初始资金建池
    pub fn launch(node_id: String, initial_coin: f64) -> Self {
        let initial_token = 9_000.0;
        Self {
            node_id,
            coin_reserve: initial_coin,
            token_reserve: initial_token,
            k: initial_coin * initial_token,
        }
    }

    /// 报价函数：想买 out_tokens 个引用权，需要支付多少 Coin
    pub fn get_amount_in(&self, out_tokens: f64) -> Result<f64, String> {
        if out_tokens >= self.token_reserve {
            return Err("Insufficient token liquidity".into());
        }
        let new_token_reserve = self.token_reserve - out_tokens;
        let new_coin_reserve = self.k / new_token_reserve;
        Ok(new_coin_reserve - self.coin_reserve)
    }

    /// 买入节点 Token (获取引用权)
    pub fn swap_coin_for_token(&mut self, coins_in: f64) -> f64 {
        let new_coin_reserve = self.coin_reserve + coins_in;
        let new_token_reserve = self.k / new_coin_reserve;
        let tokens_out = self.token_reserve - new_token_reserve;
        self.coin_reserve = new_coin_reserve;
        self.token_reserve = new_token_reserve;
        tokens_out
    }

    /// 抛售 Token 套现 (拓荒者离场)
    pub fn swap_token_for_coin(&mut self, tokens_in: f64) -> f64 {
        let new_token_reserve = self.token_reserve + tokens_in;
        let new_coin_reserve = self.k / new_token_reserve;
        let coins_out = self.coin_reserve - new_coin_reserve;
        self.coin_reserve = new_coin_reserve;
        self.token_reserve = new_token_reserve;
        coins_out
    }
}
```

#### 2. Kernel 投资逻辑重写 (`src/kernel.rs`)

Agent 新增 `portfolio: HashMap<String, f64>` 记录持有的各种 Node Token。

```rust
impl Kernel {
    /// 提交新推导，接受真实市场的现货考验
    pub fn execute_invest(&mut self, agent_id: &str, parent_id: Option<&str>, new_node_id: &str) -> Result<(), String> {
        let mut total_cost = 0.0;
        let agent = self.agents.get_mut(agent_id).unwrap();

        // 1. 若引用别人，必须拿真金白银当场买入 100 枚父节点 Token
        let required_citation_tokens = 100.0;
        if let Some(pid) = parent_id {
            if let Some(parent_pool) = self.amms.get(pid) {
                let cost = parent_pool.get_amount_in(required_citation_tokens)?;
                total_cost += cost;
            } else {
                return Err("Parent pool missing".into());
            }
        }

        // 2. 创立新池子的强制做市成本 (IDO)
        let ido_cost = 50.0;
        total_cost += ido_cost;

        // 3. 结算破产判定
        if agent.balance < total_cost {
            return Err(format!("Bankrupt! Need {:.2} coins, have {:.2}", total_cost, agent.balance));
        }
        agent.balance -= total_cost;

        // 4. 执行资金划转
        if let Some(pid) = parent_id {
            let parent_pool = self.amms.get_mut(pid).unwrap();
            parent_pool.swap_coin_for_token(total_cost - ido_cost);
        }

        // 5. 新池子开盘，创始人获得 1000 枚"原始股"
        let new_pool = UniswapPool::launch(new_node_id.to_string(), ido_cost);
        self.amms.insert(new_node_id.to_string(), new_pool);
        *agent.portfolio.entry(new_node_id.to_string()).or_insert(0.0) += 1000.0;

        Ok(())
    }

    /// OMEGA 结算：真实的资本注入与巨鲸狂欢
    pub fn liquidate_omega(&mut self, golden_path_node_ids: Vec<String>) {
        let absolute_bounty = 100_000.0;
        let bounty_per_node = absolute_bounty / golden_path_node_ids.len() as f64;

        // 1. 悬赏金硬砸 Golden Path 上的池子，K 值跳跃
        for node_id in &golden_path_node_ids {
            if let Some(pool) = self.amms.get_mut(node_id) {
                pool.coin_reserve += bounty_per_node;
                pool.k = pool.coin_reserve * pool.token_reserve;
            }
        }

        // 2. 持有者高位套现
        for (agent_id, agent) in self.agents.iter_mut() {
            for node_id in &golden_path_node_ids {
                if let Some(token_amount) = agent.portfolio.get_mut(node_id) {
                    if *token_amount > 0.0 {
                        let pool = self.amms.get_mut(node_id).unwrap();
                        let cash_out = pool.swap_token_for_coin(*token_amount);
                        agent.balance += cash_out;
                        *token_amount = 0.0;
                    }
                }
            }
        }
    }
}
```

## 二、Agent $HOME 隔离

每个 Agent 拥有独立的 $HOME 目录：
- 可以读别人的 $HOME（信息自由，Law 1）
- 只能写自己的 $HOME（数字产权，Law 3）

## 三、Init AI 模块

架构中新增 Init AI 模块：
- 多次初始化：先试跑几轮
- 顶层审查：让顶层观察问题
- 迭代修复：改了再重新初始化
- 测试集推荐：AI 驱动的测试集选择

## 哈耶克终极背书（架构师预言的涌现行为）

1. **废物自动归零**: 愚蠢节点花 50 Coin 建池，永远无人引用，投资打水漂
2. **专业分工涌现**: 算力不足的 Agent 转为"基础科研做市商"，中途套现原始股
3. **价格即真理**: 池子资金流最多、滑点最高、Token 最贵 = 离真理最近
