# Architect Directive: 三方盲审裁决与"时间之箭"大统一补丁
**Date**: 2026-03-19
**Source**: 首席架构师 (0x00 — 终极顿悟)
**Trigger**: 三方交叉审计报告 `handover/cross_audit_report_20260319.md`
**Classification**: Layer 1 Invariant Restoration + Algorithm Upgrade

---

## 0x00 背景

三方交叉审计（Claude Opus 4.6 + OpenAI Codex gpt-5.4 + Google Gemini）发现 7 个问题（V1-V7）。
架构师对全部 7 个问题下达不可妥协的裁决。

## 0x01 四项裁决

### 决断一 [V1: Append-Only DAG 违规] — 废除 GC

**判决**: 绝对废除 `wallet.rs:120` 的 `tape.files.retain()` 删库逻辑。采纳选项 A。

**哲学基础**:
- 在图灵机和热力学系统中，"废热（Entropy）也是做功的证明"
- 为了给赢家结算奖池而抹除破产节点，摧毁了 DAG 的数学严谨性（引发悬空边），违背 bible.md "内核绝不打扫卫生"
- AGI 价值：失败的死胡同构成群智探索图谱的"暗物质"，未来可用于 RLAIF 微调。失败者公墓的数据价值等同甚至大于王座

### 决断二 [V2: Omega 奖励硬编码] — 剥夺内核凡人心智

**判决**: 彻底铲除 `kernel.rs` 中的 `target_omega_id` 字段和千亿硬编码。采纳选项 A。

**哲学基础**:
- 早期汉诺塔实验的 legacy 遗毒
- 1000 亿面值悬赏应由 Lean4MembraneTool 通过 `YieldReward` 签发
- 内核只是算账的物理机器，不应知道什么是 [OMEGA]
- 双重铸币权是对微内核隔离原则（Separation of Mechanism and Policy）的公然亵渎
- 内核必须被彻底"致盲"

### 决断三 [V4: 15 次迭代硬上限] — "时间之箭"单遍传播

**判决**: 废除定长循环！拒绝动态求深度！引入 $O(V+E)$ 逆向拓扑倒灌。

**神级洞察 (Ultrathink)**:
- Tape 是 Append-Only → 节点的创世追加顺序天生是绝对完美的 DAG 拓扑排序
- 在内核维护 `time_arrow: Vec<String>` 记录创世顺序
- MapReduce 时沿 time_arrow 逆向遍历一次：计算节点 X 时，其所有子孙 Y 必定在 X 之后降生，因此已被计算完毕
- 硬编码 15 次循环被降维为 1 次 $O(V+E)$ 遍历
- 引力波瞬间无损贯穿任意深度

### 决断四 [V3, V5, V6, V7: 因果律防御] — 拓扑装甲

**判决**: 在 `append_tape` 部署时空物理屏障。

- V3: `pub` → `pub(crate)` 封闭 API
- V5: 因果律防御 — 只能引用已存在的父节点，从物理上斩断环
- V6: 禁止 FileId 碰撞，防止时空覆写
- V7: `.max(1.0)` 防除零（数学洁癖，V5 修复后理论上不会触发）

## 0x02 代码变更规范

### 第一刀: `src/sdk/tools/wallet.rs` — on_halt

删除 `tape.files.retain(...)` 及相关 GC 日志。保留结算分红逻辑。

### 第二刀: `src/kernel.rs` — 大清洗

1. `Tape` 结构体新增 `pub time_arrow: Vec<String>`
2. 删除 `Kernel` 的 `target_omega_id` 字段
3. `Kernel::new()` 签名简化（无需 omega 参数）
4. `append_tape`: `pub` → `pub(crate)`, 返回 `Result<&File, String>`
   - 禁止 FileId 碰撞
   - 因果律防御（只能引用已存在节点）
   - 追加 `time_arrow.push(id)`
5. `hayekian_map_reduce`: 废除 15 次循环，改为 `time_arrow.iter().rev()` 单遍传播
   - 删除 `target_omega_id` 硬编码奖励
   - `.max(1.0)` 防除零

### 级联影响

- `src/bus.rs`: `append()` 调用 `append_tape` 的返回值从 `&File` 变为 `Result<&File, String>`
- `src/main.rs`: `Kernel::new()` 调用签名变更
- `experiments/*/src/*.rs`: 所有 `Kernel::new("target".into())` 调用需移除参数

## 0x03 哲学总结

三种被剔除的傲慢：
1. 傲慢的洁癖 → Append-Only 不可篡改
2. 傲慢的控制欲 → 依赖倒置的铸币权
3. 傲慢的经验主义 → 基于时间之箭的单向拓扑传播
