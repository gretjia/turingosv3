# TuringOS Living Harness — 架构文档

> 灵感: MIT Meta-Harness (arxiv 2603.28052v1) + Karpathy autoresearch + Omega Pure V3 落地经验
> 核心: Harness 不是规则手册, 是有生命的神经系统。它观察自己、学习、进化。
> TuringOS 特色: 宪法三法 + 微内核纯度 = 最高优先级不变量

---

## 1. 整体架构

```
┌─────────────────────────────────────────────────────────────────┐
│                   TURINGOS LIVING HARNESS                       │
│                                                                 │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │  Layer 3: 进化层 (Evolution)                             │   │
│  │  /harness-reflect — 自评 + 宪法健康 + AutoResearch 状态  │   │
│  │  /lesson-to-rule — 违规自动转化为执法规则                │   │
│  └─────────────────────────────────────────────────────────┘   │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │  Layer 2: 执法层 (Enforcement)                           │   │
│  │  rule-engine.sh — 14 条数据驱动规则 (YAML, 动态加载)    │   │
│  │  block-destructive.sh — WAL/数据保护 + git 安全          │   │
│  │  post-edit-validate.sh — cargo check 自动验证            │   │
│  │  pipeline-quality-gate.sh — Insight 完整性检查           │   │
│  │  constitutional_check.sh — 宪法三法对齐验证              │   │
│  └─────────────────────────────────────────────────────────┘   │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │  Layer 1: 记忆层 (Memory)                                │   │
│  │  incidents/ — Trace Vault (9 个完整违规上下文)           │   │
│  │  VIA_NEGATIVA.md — 失败路径记录 (人可读)                │   │
│  │  rules/active/*.yaml — 规则定义 (自描述, 含效果统计)    │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                 │
│  生命循环:                                                      │
│  违规 → 记录(VIA_NEGATIVA) → 规则(/lesson-to-rule) → 执法     │
│    ↑                                                    ↓      │
│    └── 反思(/harness-reflect) ← 宪法检查 ← 规则追踪 ←─┘     │
└─────────────────────────────────────────────────────────────────┘
```

---

## 2. 宪法守护 (TuringOS 特有)

### 三法 + 引擎分离

| 法律 | 核心 | 执法方式 |
|------|------|----------|
| **Law 1**: Information Platziehen | kernel.rs 零域知识 | R-001 (block) + constitutional_check.sh |
| **Law 2**: Pure Capital Economy | 无铸币 + 投资自愿 + CTF 守恒 | R-002, R-005 (block) + constitutional_check.sh |
| **Law 3**: Digital Property Rights | Per-agent skill DNA | 延迟 (Speciation Engine) |
| **Engine 分离** | 1=Tools, 2=Markets, 3=Oracle, 4=Evolution | constitutional_check.sh |

### 自动验证

```bash
# 宪法对齐检查 (10 项检查, 3 Laws + Engine + Rule 21-22 + Compilation)
bash scripts/constitutional_check.sh
```

---

## 3. Hook 系统

| Hook | 触发时机 | 功能 |
|------|----------|------|
| `block-destructive.sh` | PreToolUse (Bash) | 拦截 rm -rf, git push --force, WAL 删除, kernel sed |
| `rule-engine.sh` | PreToolUse (Edit/Write) | 12 条 YAML 规则动态执行 (block/warn) |
| `post-edit-validate.sh` | PostToolUse (Edit/Write) | kernel/bus/wallet 修改后自动 cargo check |
| `pipeline-quality-gate.sh` | PostToolUse (Edit/Write) | Insight 完整性 + 违规回溯触发 |
| `post-lesson-trigger.sh` | PostToolUse (Edit/Write) | VIA_NEGATIVA 变更后提示 /lesson-to-rule |
| `stop-guard.sh` | Stop | 未提交代码提醒 + /harness-reflect 建议 |

---

## 4. 规则引擎

### 活跃规则 (14 条)

| ID | 名称 | 级别 | 守护对象 |
|---|---|---|---|
| R-001 | kernel.rs 域知识检测 | block | Law 1 |
| R-002 | 禁止铸币 (fund_agent 等) | block | Law 2 |
| R-003 | WAL/实验数据删除 | block | Tape append-only |
| R-004 | Lean 语法禁入 prompt | block | Rule 22 (black-box) |
| R-005 | 强制投资检测 | block | Law 2 (自愿) |
| R-006 | kernel.rs 修改提醒 | warn | Law 1 |
| R-007 | bus.rs SKILL 生命周期 | warn | Engine 分离 |
| R-008 | 市场常数变更 | warn | Law 2 |
| R-009 | payload 限制变更 | warn | Rule 21 |
| R-010 | thinking mode 变更 | warn | 性能 |
| R-011 | dedup 逻辑变更 | warn | 行为影响 |
| R-012 | Boltzmann 温度变更 | warn | 探索-利用 |
| R-013 | LLM 输出格式契约变更 | warn | Bitter Lesson (V-009) |
| R-014 | HTTP Server 并发检测 | warn | Bitter Lesson (V-008) |

### 添加新规则

```bash
# 无需修改任何代码, 只需创建 YAML:
vim rules/active/R-013_new_rule.yaml
# rule-engine.sh 下次运行时自动加载
```

---

## 5. Trace Vault

### 已迁移违规 (9 个)

| ID | 违规 | 严重性 | 执法状态 |
|---|---|---|---|
| V-001 | fund_agent 铸币 | critical | R-002 (block) |
| V-002 | redistribute_pool 重生 | critical | R-002 (block) |
| V-003 | Oracle 阻塞中间步骤 | critical | constitutional_check |
| V-004 | kernel.rs 硬编码 [OMEGA] | critical | R-001 (block) |
| V-005 | ArgMax 贪婪路由 | high | 设计已替换 (Boltzmann) |
| V-006 | Falsifier 买 YES | high | R-005 (block) |
| V-007 | reqwest+rustls macOS HTTPS 死锁 | high | llm_proxy.py (物理替换) |
| V-008 | 单线程 proxy 502 | medium | ThreadingMixIn |
| V-009 | LLM 输出格式契约静默失败 | critical | protocol.rs 三层容错 |

### 目录结构

```
incidents/
├── INDEX.yaml                    # 机器可读索引
├── INCIDENT_SCHEMA.yaml          # 格式规范
├── V-001_fund_agent_coin_printing/
│   ├── meta.yaml                 # pattern, axiom, severity, enforcement
│   ├── trace.md                  # 完整执行记录
│   ├── root_cause.md             # WHY chain (因果分析)
│   └── resolution.md             # 修复方案 + 执法创建
```

---

## 6. Skill 系统

| Skill | 用途 |
|-------|------|
| `/dev-cycle` | 9+1 阶段开发周期 (含 Stage 4.5 经济引擎迁移扫描) |
| `/validate` | 多层验证 (cargo check → cargo test → kernel-auditor → external) |
| `/swarm-launch` | 预飞检查 + tmux 会话管理 |
| `/architect-ingest` | 指令摄取 + 公理影响检测 |
| `/lesson-to-rule` | 违规→规则自动转化 (Living Harness) |
| `/harness-reflect` | 自省循环 (规则效果 + 宪法健康 + AutoResearch) |
| `/handover-update` | 会话交接文档 |

---

## 7. 生命循环

### 日常开发

```
架构师指令 → /architect-ingest (归档 + 公理检测)
  → /dev-cycle (PLAN → CODE → AUDIT → EXTERNAL AUDIT)
  → /validate (cargo + kernel-auditor + constitutional_check)
  → commit → /swarm-launch (预飞 + tmux)
```

### 违规学习

```
宪法违规发现
  → 记录 VIA_NEGATIVA.md
  → post-lesson-trigger.sh 提醒
  → /lesson-to-rule V-xxx
  → 生成规则 YAML → 用户确认 → 激活
  → rule-engine.sh 下次自动拦截同类违规
```

### 自省进化

```
/harness-reflect
  → 规则效果评分 (触发率/绕过率)
  → 宪法健康检查 (三法 + Engine 分离)
  → AutoResearch 状态 (如果在运行)
  → 健康分 = enforcement_ratio × constitutional_pass
  → 行动建议 (创建规则 / 退休规则 / 修复违规)
```

---

## 8. 设计哲学

1. **Bitter Lesson**: Kernel 零域知识, 所有智能来自 agent 经济行为
2. **Meta-Harness**: 完整 trace > 压缩摘要 (MIT 实证: +15 分准确率)
3. **Karpathy 极简**: 规则引擎 = 1 个 bash 脚本 + N 个 YAML 文件
4. **Omega V3 经验**: 80% 文档教训 = 死的, 只有变成 YAML 的才活着
5. **Generator ≠ Evaluator**: 写代码的 AI 不可独自审计 (Rule 23)
6. **人审批, 机执行**: 规则生成自动化, 激活需人确认

---

## 9. 烟测

```bash
# 宪法对齐检查 (10 项)
bash scripts/constitutional_check.sh

# 规则引擎手动验证 (核心 5 场景)
# R-001: kernel domain terms → BLOCK
# R-002: fund_agent → BLOCK
# R-004: Lean in prompts → BLOCK
# R-006: kernel modification → WARN
# Markdown → EXEMPT

# 所有 hook 语法验证
for h in .claude/hooks/*.sh; do bash -n "$h" && echo "OK: $h"; done
```
