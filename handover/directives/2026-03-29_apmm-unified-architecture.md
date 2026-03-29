# 2026-03-29 APMM (Automatic Polymarket Maker) 统合架构指令

## 来源
架构师口述 + Mathematica 闭式解推导 (Out[92], Out[93], Out[95])

## 核心数学发现

### Mathematica 闭式解
- **Out[92]**: `getY = payC + (payC * poolY) / (payC + poolN)` — Mint-and-Swap 最终交付
- **Out[93]**: `poolY1 = poolY - (payC * poolY) / (payC + poolN)` — 池子 YES 余额更新
- **Out[95]**: `poolN * poolY == poolN1 * poolY1` — K 值恒等式守恒证明

### Mint-and-Swap 五步原子路由
1. **抵押铸造 (Minting)**: 锁入 payC 个 Coin，原子生成 payC YES + payC NO
2. **资产截留 (Retention)**: 路由器截留买方看多侧（如 YES）发入钱包
3. **单边抛售 (Swapping)**: 路由器将买方废弃侧（如 NO）砸入 APMM 池 (dN = payC)
4. **流动性置换 (Exchange)**: 依据 poolY·poolN = K，池子吐出 dY = (payC·poolY)/(payC+poolN) 个 YES
5. **最终交付 (Delivery)**: 买方获得 getY = payC + dY

---

## 完整架构：四大引擎统合 (APMM 版)

### 引擎一：认识论引擎 (The Epistemic Engine)
对齐立法 1：黑盒使用白盒工具绝对免费
* 原理: 只要不干涉金融账本，任何思考与拓扑建树行为绝对免费。真理图谱是无版权的公共品。
* 白盒工具矩阵: [MathlibOracle], [PythonSandbox], [Falsify], [AppendNode]（绝对零成本物理上链，保障毫无金融风险的试错与发声权）。
* 涌现预期: 谋定而后动。穷尽一切免费手段试错后，再以资本发起致命一击。

### 引擎二：纯粹资本引擎 (The APMM Capital Engine)
对齐立法 2：唯一消耗货币的场景是投资
* 原理 (Mint-and-Swap 物理法则): 资本 = 拓扑引力的唯一能源。
* 统一入口: [Tool: Wallet | Action: Invest | Node: <Target> | Direction: YES/NO | Amount: <FLOAT>]
  1. 矿工/点火者: 免费建树后首个下注，自愿切出微小资金充当 APMM 创世 LP，剩余全额执行 Mint-and-Swap。承担无常损失，锁定极早期暴利。
  2. VC 寡头 (多头): Invest YES。热钱吸干池中 YES，推高贝叶斯概率 P_yes。
  3. 做空刺客 (空头): Invest NO。猎杀幻觉。空头会将手中的 YES 砸入池子抽干 NO。

### 引擎三：热力学截断引擎 (The Semantic Guillotine)
为引擎 1 和 2 提供物理结界兜底
* "No goals to be solved" 出现 → 切断解析，盖上 OMEGA 印章
* Oracle 刚性兑付：判定黄金路径 YES=1, NO=0，死胡同 NO=1, YES=0
* 出清法则：所有人凭获胜代币 1:1 提取输家锁在金库的本金。创世 LP 提取池子里剩余的获胜代币。系统银行彻底出清。

### 引擎四：拉马克物种演化引擎 (The Speciation Engine)
对齐立法 3：每个 Agent 有独立 Skill 路径
* 物理隔离: skills/agent_X/
* 尸检与复盘: 资金归零 → 强制反思写入保命铁律；赚钱 → 经验固化
* 涌现预期: 产生 VC 寡头、包工头、嗜血空头刺客

---

## Rust 重构方案 (架构师提供的伪代码)

### apmm.rs — APMM 路由器
```rust
pub struct APMMPool {
    pub node_id: String,
    pub pool_y: f64,
    pub pool_n: f64,
}

impl APMMPool {
    pub fn ignite(node_id: String, seed_amount: f64) -> Self { ... }
    pub fn buy_yes(&mut self, pay_c: f64) -> f64 { ... }  // Mathematica Out[92]
    pub fn buy_no(&mut self, pay_c: f64) -> f64 { ... }   // 对称法则
    pub fn spot_price_yes(&self) -> f64 { ... }            // 概率雷达
}
```

### kernel.rs — 金库结界
```rust
impl Kernel {
    pub fn execute_invest(...) -> Result<(), String> { ... }
    pub fn oracle_settle(&mut self, omega_id: &str) { ... }  // 终局大出清
}
```

---

## 三条新大宪章条款 (架构师明确指令)

### 条款 A: 系统自动做市
每个新节点被创造时，系统自动往里面注入 YES NO 代币各 100 个，承担最初的做市商角色。

### 条款 B: 做市商 = Price Oracle
做市商同时承担 Price Oracle 的角色，给每个 Agent 广播每个节点的 YES/NO 币价（即对应概率）。

### 条款 C: 做市商允许小范围盈亏
允许做市商小范围亏损、盈利（放松原 Law 2 "银行出清盈亏绝对为 0" 的刚性约束）。

---

## 架构意义
- Mint-and-Swap 闭式解消除了所有浮点迭代，O(1) 代数运算
- 系统自动做市 (100 YES + 100 NO) 消除冷启动问题，每个节点天生有价格信号
- 做市商允许小范围盈亏 = 承认流动性提供的物理成本（无常损失），更贴近真实经济学
