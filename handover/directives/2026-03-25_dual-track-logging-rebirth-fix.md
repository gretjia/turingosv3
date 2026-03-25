# 架构师指令：双轨日志体系 + 世代交替纠偏 + 3 个独立发现

**Date**: 2026-03-25
**Context**: Run 2/3 数学审计暴露日志截断 + 世代交替误触发
**Status**: 待实施

---

## 一、双轨日志体系与出版级导出

**设计理念**: 日志在物理存储层与认知交互层彻底分离。

- **Immutable Track (不可变绝对真理层)**: 写入 WAL 时 100% 原始、未修改、未截断
- **Summary Track (黑盒认知层)**: 终端 `log::info!` 和大模型上下文使用 `.take(150)` 截断
- **dump_golden_path**: OMEGA 后沿 DAG 逆向追溯，导出完整 Markdown 排版

截断标记格式: `... [TRUNCATED, original size: N chars]`

---

## 二、世代交替纠偏 (Rebirth Logic Fix)

**旧逻辑 (有 bug)**: `consecutive_timeouts >= 2` — Solvent 15/15 时仍触发 rebirth

**新逻辑**: 必须同时观测"耗资行为"和"免费行为"

| 条件 | 触发 Rebirth? | 理由 |
|------|-------------|------|
| solvent_count == 0 | ✅ 是 | 真正全体破产 |
| 投资超时 AND 免费动作也超时 | ✅ 是 | 绝对物理死锁 |
| 投资超时 BUT Search/View 活跃 | ❌ 否 | 大宪章 Law 1 保护免费阅读权 |

需要追踪 `last_free_action_time` (Search/View/Observe 的最后活跃时间)。

---

## 三、3 个独立发现

### 发现 1: WAL 写入撕裂 (I/O Tearing)
- **隐患**: 多 Agent 并发写同一 WAL → JSON 字节交错
- **评估**: ✅ 当前安全。Actor Model reactor 串行处理所有 append，WAL 写入在 reactor 内部，天然 MPSC。
- **行动**: 无需修复。

### 发现 2: 幽灵上下文污染 (Phantom Context Leakage)
- **隐患**: Rebirth 时未清理 agent 私有上下文 (private_ctx: Arc<Mutex<String>>)
- **影响**: 新世代 agent 可能接收到上一代的 Search/View 残留结果
- **行动**: rebirth 时通过 snapshot 广播重置，或追加清理机制

### 发现 3: 内存 DAG 爆炸 (OOM via Unbounded DAG)
- **隐患**: kernel.tape.files 存储完整 payload 在内存中
- **评估**: 当前规模 (数百节点，每节点 ~1KB) 约 1MB，无风险
- **行动**: Deferred。万级节点时需 mmap/offset 方案。
