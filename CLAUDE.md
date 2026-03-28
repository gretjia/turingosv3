# TuringOS v3 — Project Constitution

## WHAT: Project Definition

Silicon-Native Microkernel for LLM Formal Verification Swarm.

- **Tech Stack**: Rust 2021, tokio, reqwest, serde_json
- **Core Mission**: MiniF2F 244 道 Lean 4 定理证明
- **Architecture**: Star-Topology Microkernel — kernel.rs 是纯拓扑 + 数学中心，一切领域智能在外部 SKILL

## WHY: Core Philosophy (对齐大宪章 `handover/directives/2026-03-20_magna-carta-vfinal.md`)

- **《苦涩的教训》**: 内核零领域知识，一切智能在外部 SKILL
- **机制与策略分离**: Kernel = 纯拓扑 + 数学 (零盈亏金库), SKILL = 领域策略
- **Popperian 证伪**: Lean 4 编译器是绝对真理仲裁者 (Oracle 断头台)
- **Austrian Economics / Polymarket**: YES/NO 二元条件代币守恒，价格 = 贝叶斯概率，银行盈亏 = 0

## HOW: Inviolable Rules

### Layer 1 — Eternal Invariants (永恒不变量，对齐大宪章三大立法)

1. **kernel.rs 零领域知识** — 不可包含任何 Lean/数学/领域字符串
2. **Tape 是 Append-Only DAG** — 不可删除已写入节点
3. **信息平权 (大宪章 Law 1)** — 建树 (append_node) 绝对零成本。黑盒使用白盒工具免费。禁止 IP 垄断。拓扑与金融物理剥离。
4. **共识的代价 (大宪章 Law 2)** — 唯一消耗货币的行为是投资 (invest YES/NO)。1 Coin = 1 YES + 1 NO (CTF 守恒铸造)。银行出清盈亏 = 0。废除一切补贴、悬赏、intrinsic_reward 铸币。
5. **数字产权 (大宪章 Law 3)** — 每个 Agent 有独立 Skill 路径。物种演化。

### Layer 2 — Evolvable Parameters (可演进参数)

6. 并发度 N (当前 15, 目标 100)
7. Boltzmann 温度 T=0.5
8. Anti-Zombie 阈值 = 3 次连续重复
9. 活跃模型 = DeepSeek V3.2 (deepseek-chat) + DeepSeek Reasoner

### Destructive Operation Red Lines (破坏性操作红线)

10. 修改 kernel.rs 的纯数学逻辑 → **必须人工确认**
11. 删除 WAL 文件或实验数据 → **必须人工确认**
12. git push / 远程推送 → **必须人工确认**
13. 修改 bus.rs 的 SKILL 生命周期钩子 → **必须人工确认**

### Engineering Standards (工程规范)

14. `cargo check` 必须通过才可提交
15. `cargo test` 必须通过才可部署
16. `.env` 中的 API Key 不可提交到 git
17. **经济引擎变更时必须全仓库 grep** — 修改 kernel 定价/wallet/bus 结算/reward 信号时，必须扫描 `experiments/*/src/` 中所有 SKILL 实现的兼容性。Run 6 100B-mint 事件教训: 仅审计 `src/` 而遗漏 `experiments/` 导致 Hayekian 遗产在 Polymarket 体制下破坏零和守恒。
18. **宪法违规零容忍** — 审计中发现的大宪章违规不可标注为"可接受"、"低优先级"或"Phase N 延后"。宪法不存在"可接受的违反"。所有 KNOWN VIOLATION 必须在当前 dev-cycle 内修复或获得架构师明确豁免。教训: `fund_agent` 印钞、`global_pool` 再���配、`rebirth 10K` 注入等违反 Law 2 的代码被 4 次审计放过，每次都标注"可接受"，直到架构师亲自指出。
19. **创世后零印钞** — `on_init` 是唯一合法的 Coin 注入点 (GENESIS)。之后系统绝不创造新 Coins。`fund_agent`、`redistribute_pool` 已废除。rebirth 不注入新钱。破产 Agent 靠 Law 1 (免费 append) 存活。
20. **苦涩的教训: 禁止 Over-Alignment** — 新宪法条款生效时，必须从第一性原理重新设计每个组件，不可在旧组件上打补丁。审计不仅检查"规则是否被正确实现"，还必须检查**"组件的职责范围是否与宪法设计意图一致"**。教训: Lean4Oracle 从旧版复制后只做安全补丁 (sorry/identity theft/forbidden)，但从未质疑"Oracle 该不该拦截中间步骤" — 这是 Engine 2 (市场) 的职责而非 Engine 3 (Oracle) 的职责。过度安全 = 扼杀涌现。每个 Engine 只做自己分内的事: Engine 1 = 免费工具, Engine 2 = 市场定价, Engine 3 = OMEGA 验证, Engine 4 = 物种演化。
21. **一步一节点, 禁止抢跑 (Front-Running)** — 每次 append 只允许写一个原子推理步骤。禁止在一个节点中打包多个步骤 (如 37 行完整证明)。原因: (1) 每个步骤必须是独立可定价的 (Engine 2 需要对每步做 YES/NO 判断); (2) 打包多步 = 垄断证明路径, 剥夺其他 Agent 的协作机会; (3) 单步才能产生有效的 DAG 分支探索。内核 (bus.rs) 通过 `max_tactic_lines` 物理限制。
22. **形式化不可引入暴力搜索空间** — 写 Lean 4 定理声明时，禁止使用 `Finset.range N` 等有限范围来限定搜索空间。这会让 `decide`/`omega` 暴力穷举变得可行，绕过构造性推理。正确做法: 用全称量词 (`∀ b : ℕ`) + 等价条件 (`↔`) 逼迫 Agent 发现数学结构。教训: AIME P1 用 `Finset.range 100` 形式化 → Agent 用 `decide` 遍历 100 个数暴力验证 → 看似 "证明" 实则穷举。形式化本身制造了暴力搜索机会。

### Key File Map (关键文件地图)

| 文件 | 角色 |
|------|------|
| `src/kernel.rs` | 神圣微内核 (纯拓扑 + 零盈亏金库 + Oracle 清算) |
| `src/prediction_market.rs` | BinaryMarket CPMM (YES/NO 恒定乘积 + LP 追踪) |
| `src/bus.rs` | TSP 事件总线 (SKILL 生命周期 + 拓扑/金融剥离编排) |
| `src/sdk/tools/wallet.rs` | WalletTool (余额 + YES/NO/LP 持仓组合) |
| `handover/directives/2026-03-20_magna-carta-vfinal.md` | **大宪章 vFinal** — 三大立法 + 四大引擎 |
| `handover/ai-direct/LATEST.md` | 会话状态单一真相源 |
| `handover/bible.md` | 哲学基石 (禁止修改) |

### Context Management (上下文管理)

18. `handover/ai-direct/LATEST.md` — 当前状态真相源
19. `handover/bible.md` — 哲学基石，只读
20. `handover/directives/2026-03-20_magna-carta-vfinal.md` — 大宪章，最高立法权威
21. `handover/` — 架构审计存档
22. `handover/architect-insights/` — 架构师口头洞察浓缩归档（每条≤50字本质）
23. 架构师分享非显而易见的设计原则时，必须通过 `/architect-ingest` 归档
24. 审计等重输出通过 Agent 子进程执行，仅返回 verdict + 关键发现，防止主上下文膨胀

### Hardware Topology (硬件拓扑)

| 节点 | 角色 | SSH 别名 | 备注 |
|------|------|---------|------|
| **omega-vm** (当前机器) | GCP 主控, 代码仓库, Git | localhost | 16GB, 无 GPU |
| **zephrymac-studio** | 架构师 Mac, Apple M4 32GB | `ssh zephrymac-studio` (ProxyJump hk-wg, port 2227) | Lean 4 应安装在此 |
| **linux1-lx** | 深圳工作站, AMD AI Max 395 128GB | `ssh linux1-lx` (ProxyJump hk-wg, port 2226) | 高性能计算 |
| **windows1-w1** | 深圳工作站, AMD AI Max 395 128GB | `ssh windows1-w1` (ProxyJump hk-wg, port 2228) | 数据存储 |

网络路由: omega-vm → HK 公网跳板 (43.161.252.57) → WireGuard → 深圳局域网
完整拓扑详见: `handover/network_topology_and_ssh.md`

### User Profile (用户画像)

25. 独狼研究员，零编程基础 vibe coder
26. 优势是品味、架构直觉、哲学深度
27. 沟通偏好：中文为主，技术术语可用英文
