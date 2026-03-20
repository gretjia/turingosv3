# TuringOS v3 — Project Constitution

## WHAT: Project Definition

Silicon-Native Microkernel for LLM Formal Verification Swarm.

- **Tech Stack**: Rust 2021, tokio, reqwest, serde_json
- **Core Mission**: MiniF2F 244 道 Lean 4 定理证明
- **Architecture**: Star-Topology Microkernel — kernel.rs 是纯拓扑 + 数学中心，一切领域智能在外部 SKILL

## WHY: Core Philosophy

- **《苦涩的教训》**: 内核零领域知识，一切智能在外部 SKILL
- **机制与策略分离**: Kernel = 纯拓扑 + 数学, SKILL = 领域策略
- **Popperian 证伪**: Lean 4 编译器是绝对真理仲裁者
- **Austrian Economics**: 自由浮动质押，价格发现替代中央调度

## HOW: Inviolable Rules

### Layer 1 — Eternal Invariants (永恒不变量)

1. **kernel.rs 零领域知识** — 不可包含任何 Lean/数学/领域字符串
2. **仅 SKILL 可铸造 intrinsic_reward** — kernel 不可直接赋值
3. **Tape 是 Append-Only DAG** — 不可删除已写入节点
4. **质押必须 >= 1.0** — 零成本操作被禁止

### Layer 2 — Evolvable Parameters (可演进参数)

5. 并发度 N (当前 5, 目标 100)
6. Boltzmann 温度 T=0.5
7. Anti-Zombie 阈值 = 3 次连续重复
8. 活跃模型 = doubao-1-5-pro-32k-250115 (Volcengine)

### Destructive Operation Red Lines (破坏性操作红线)

9.  修改 kernel.rs 的纯数学逻辑 → **必须人工确认**
10. 删除 WAL 文件或实验数据 → **必须人工确认**
11. git push / 远程推送 → **必须人工确认**
12. 修改 bus.rs 的 SKILL 生命周期钩子 → **必须人工确认**

### Engineering Standards (工程规范)

13. `cargo check` 必须通过才可提交
14. `cargo test` 必须通过才可部署
15. `.env` 中的 API Key 不可提交到 git

### Key File Map (关键文件地图)

| 文件 | 角色 |
|------|------|
| `src/kernel.rs` | 神圣微内核 (纯拓扑 + Hayekian map-reduce) |
| `src/bus.rs` | TSP 事件总线 (SKILL 生命周期) |
| `src/sdk/tools/wallet.rs` | WalletTool (PoS 经济中央银行) |
| `experiments/minif2f_swarm/src/swarm.rs` | Boltzmann 路由器 |
| `experiments/minif2f_swarm/src/lean4_membrane_tool.rs` | Lean 4 编译膜 |
| `handover/ai-direct/LATEST.md` | 会话状态单一真相源 |
| `handover/bible.md` | 哲学基石 (禁止修改) |

### Context Management (上下文管理)

16. `handover/ai-direct/LATEST.md` — 当前状态真相源
17. `handover/bible.md` — 哲学基石，只读
18. `handover/` — 架构审计存档
22. `handover/architect-insights/` — 架构师口头洞察浓缩归档（每条≤50字本质）
23. 架构师分享非显而易见的设计原则时，必须通过 `/architect-ingest` 归档

### User Profile (用户画像)

19. 独狼研究员，零编程基础 vibe coder
20. 优势是品味、架构直觉、哲学深度
21. 沟通偏好：中文为主，技术术语可用英文
