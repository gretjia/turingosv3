# TuringOS v3 — 顶级对齐文件 (Master Alignment Document)

**本文件是所有 AI agent 的最高行为准则。当用户说"对齐"，即指对齐本文件。**

文件按时间顺序排列，越靠后权重越高（更新了思路），但原则性条文（Layer 1）永不被后续覆盖。

---

## 一、宪法层 — CLAUDE.md (永恒不变量)
*Source: `/CLAUDE.md` | 权重: 最高 | 不可被任何后续指令覆盖*

### Layer 1 — 四大永恒不变量
1. **kernel.rs 零领域知识** — 不可包含任何 Lean/数学/领域字符串
2. **仅 SKILL 可铸造 intrinsic_reward** — kernel 不可直接赋值
3. **Tape 是 Append-Only DAG** — 不可删除已写入节点
4. **投资必须 >= 1.0** — 零成本 Tape 写入操作被禁止（纯信息查询类免费工具不受此约束）

### Layer 2 — 可演进参数
5. 并发度 N（当前 15）
6. Boltzmann 温度 T=0.5（softmax 前沿选点）
7. Anti-Zombie 阈值 = 3 次连续重复
8. 模型配置（当前: DeepSeek V3.2 (deepseek-chat) + deepseek-reasoner + R1 三物种）

### 破坏性操作红线（必须人工确认）
9. 修改 kernel.rs 纯数学逻辑
10. 删除 WAL 文件或实验数据
11. git push / 远程推送
12. 修改 bus.rs 的 SKILL 生命周期钩子

---

## 二、哲学层 — bible.md (只读基石)
*Source: `handover/bible.md` | 权重: 宪法级 | 禁止修改*

### 核心公理
- **《苦涩的教训》**: 一切试图将人类聪明才智硬编码进系统的企图，终将败于算力暴力
- **纯粹状态机**: TuringOS 内核没有智能、偏好、大局观 — 只是图灵打字机
- **质押 = 热力学摩擦**: 阻止无限猴子打字机撑爆 Tape
- **禁止奥卡姆折叠**: 照单全收，绝对允许冗余，内核绝不打扫卫生
- **Hayekian 定价**: 价格 = 全网算力对真理方向的压缩信号
- **激励兼容 (Hurwicz)**: 自私贪婪 100% 转化为真理探索动能

---

## 三、架构层 — 反奥利奥三界大一统 (2026-03-19)
*Source: `handover/directives/2026-03-19_anti-oreo-free-pricing.md` | 权重: 高*

### 三界封神榜

| 界 | 映射 | 治理手段 | 绝对禁止 |
|----|------|---------|---------|
| **顶层白盒** | Event Bus + TOOLs | 硬限制 + 零自由度 | 不可用 Prompt 去"求"大模型 |
| **中间黑盒** | LLM Agents + SKILLs | 软引导 + 绝对自由度 | **严禁 Rust 代码干预定价/策略** |
| **底层白盒** | Kernel (DAG + 时间之箭) | 数学拓扑硬不变量 | 拒绝环状引用和历史覆写 |

### 三界边界判定规则
- `if balance < 1.0 { continue; }` → ✅ 顶层白盒本职工作
- `<FLOAT>` 占位符 → ✅ 中间黑盒主权归还
- SKILL 方差示例 → ✅ Markdown 思想钢印
- ~~`conservative_stake = balance * 0.01`~~ → ❌ 越界罪 — Rust 代码侵犯定价主权

---

## 四、宪法层 — 大宪章 (2026-03-20) ⭐ 最新最高权重
*Source: `handover/directives/2026-03-20_magna-carta-vfinal.md` | 权重: 最高（在不违背 Layer 1 前提下）*

### 三大立法
1. **信息平权 (Law 1)**: 黑盒使用白盒工具绝对免费 — 思考与求知零成本
2. **共识的代价 (Law 2)**: 唯一消耗货币的场景是投资 — 资本 = 拓扑确信度
3. **数字产权 (Law 3)**: 每个 Agent 有自己独立的 Skill 路径 — 物种演化

### 四大引擎
| 引擎 | 对齐立法 | 核心机制 | 实施状态 |
|------|---------|---------|---------|
| 认识论引擎 | Law 1 | MathlibOracle/PythonSandbox 免费工具 | ✅ Phase 3 已实施 |
| 纯粹资本引擎 | Law 2 | Invest 统一入口，矿工/VC/包工头三角色 | ✅ Phase 2 已实施 |
| 热力学截断引擎 | 兜底 | Semantic Guillotine — 三层 OMEGA 防御 | ✅ Phase 1 已实施 |
| 拉马克演化引擎 | Law 3 | Per-agent DNA (skills/agent_N/) | ⏸️ Phase 4 延后 |

### Gemini 裁决 (Lean 4 形式化证明专家, 2026-03-21)
- `"No goals to be solved"` = 所有 goals 已关闭（Lean 4 精确语义，非局部）
- Termination failure = Lean 4 kernel 拒绝 = 不算证明成功
- 安全 OMEGA 条件: "No goals" 是唯一 error AND 无 `declaration uses 'sorry'` 警告
- Sorry-cheating 漏洞被上游 sorry 防火墙 + Gemini 条件 + double-check 三层覆盖

### 实施原则（架构师指令, 2026-03-21）
> "暂时不追求绝对效率，保证本项目原则对齐，为了以后更泛化的能力进行能力储备。目前所做的所有测试仅仅是为了让 TuringOS 可以尽快成为一个未来的真正的 agent OS。"

**推论**:
- 优先构建普世 OS 能力（Core SDK），而非单实验优化
- 先在实验中原型验证，验证后提升到 Core SDK
- 代码结构按 OS 接口设计，即使当前只有一个实验在用
- 所有协议（输出格式、工具接口）必须 domain-agnostic

### 无锁物理学 + 无认知溢价（架构师指令, 2026-03-21）
> "Append-Only DAG = 无锁。Agent 读快照，写子节点。不需要锁。"
> "系统绝对不为思考过程的汗水买单。价格唯一来源 = Agent 的 Amount 质押。"

**推论**:
- Actor Model: watch/mpsc 消息传递替代 block_on 同步阻塞
- 慢模型和快模型在相对论时空中共存，互不阻塞
- 无认知溢价 = 奥地利学派主观价值论，非马克思劳动价值论

### 苦涩的教训在 Prompt 层的映射（Run 13 洞察, 2026-03-21）
> "不要把复杂的社会契约编码进 prompt。保持 prompt 极简，让社会分工从 Tape 上的价格信号中自然涌现。压缩即智能。"

**推论**:
- Prompt 只给状态（proof state + market + graveyard + balance），不给规则解释
- 大宪章的规则通过 Bus/Tool 的硬执行体现，不通过 Markdown 说教体现
- "重力不需要向苹果解释自己" — 投资失败 = 烧毁，不需要在 prompt 中解释 Slashing Law

---

## 五、设计原则层 — 架构师洞察
*Source: `handover/architect-insights/` | 权重: 中 | 指导性*

- **价格即经验**: 质押价格 = Agent 历史经验的压缩编码 (Hayekian dispersed knowledge)
- **Skill 即 DNA**: 幸存 Agent 的 skill 组合是演化资产
- **自由浮动质押**: 价格发现替代中央调度

---

## 六、对齐检查清单

当接到任何变更请求时，按此顺序检查：

```
1. [L1] 是否违反 Layer 1 四大不变量？ → 违反则 BLOCK
2. [AO] 是否违反反奥利奥三界边界？ → 违反则 BLOCK
3. [MC] 是否符合大宪章三大立法？ → 不符合则 REVIEW
4. [HE] 是否引入硬编码数值？ → 引入则 BLOCK
5. [L2] 是否修改 Layer 2 可演进参数？ → 需用户确认
6. [RL] 是否触发破坏性操作红线？ → 需人工确认
```

---

## 七、零硬编码原则

**绝对禁止在 Rust 代码中硬编码影响 LLM 行为的数值。**

| 允许 | 禁止 |
|------|------|
| `Amount: <FLOAT>` (占位符) | `Amount: 500` (固定值) |
| Layer 2 参数 (10000 初始, 100B OMEGA) | `conservative_stake = balance * 0.01` |
| SKILL 示例中的数值 (仅为格式演示) | Prompt 中的固定定价建议 |
| 架构师指令中的说明性数值 | 将说明性数值实装为代码常量 |

---

## 八、关键文件索引

| 文件 | 角色 | 修改权限 |
|------|------|---------|
| `CLAUDE.md` | 宪法 | Layer 2 可改，Layer 1 不可改 |
| `handover/bible.md` | 哲学基石 | **只读，禁止修改** |
| `handover/ALIGNMENT.md` | **本文件 — 顶级对齐** | 需人工确认 |
| `handover/directives/` | 架构师指令存档 | append-only |
| `handover/architect-insights/` | 设计洞察 | append-only |
| `handover/ai-direct/LATEST.md` | 会话状态 | 每次会话更新 |
| `skills/economic_operative.md` | 经济 SKILL | 按大宪章更新 |
| `src/kernel.rs` | 神圣微内核 | **红线: 需人工确认** |
| `src/bus.rs` | 事件总线 | **红线: 需人工确认** |
