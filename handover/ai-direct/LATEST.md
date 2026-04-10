# TuringOS v3 — Handover State
**Updated**: 2026-04-10
**Session Summary**: PPUT 模型扫描 + Q3.5-122B 49x 提升 + 模型地板研究 + Oracle 修复方案

## Current State
- **PPUT 排行榜**（严格审计）:
  1. Q3.5-122B N=3: **9.87×10⁻³** (GENUINE, 4 步链, 2.5 min)
  2. Q3.5-27B N=3: 2.11×10⁻³ (BORDERLINE)
  3. Q3.5-35B-A3B N=3: 1.83×10⁻³ (GENUINE)
  4. V3.2 N=3 (baseline): 2.19×10⁻⁴ (GENUINE 8.5/10)
- **模型地板**: 27B 可靠，9B 产生 Oracle false positive
- **投资列表修复**: 5 recent + 5 top-priced (已改但未 commit) — 让 9B 突破 P=0.90
- **未提交**: evaluator.rs (投资列表修复)
- **进程状态**: 只有 DeepSeek proxy 在 8089（siliconflow）

## Changes This Session

### N Sweep 实验
1. **DeepSeek API N sweep** (V3.2): N=3 OMEGA (2/3), N≥5 全部失败。Brooks's Law 实证。
2. **Qwen3.5-122B N sweep**: N=3,5,7,10 全部 OMEGA (4/4, 100%)。N=3 PPUT=9.87×10⁻³ (49x V3.2)。
3. **模型地板 sweep**: 35B-A3B/27B OMEGA (genuine)，9B 卡在 P=0.887 (fix 前)，4B 崩溃

### 代码修复
4. **投资列表 = 5 recent + 5 top-priced** (未 commit)
   - 修复 attention dilution: 老的高价 [COMPLETE] 节点不再从视图中消失
   - 烟测: Q3.5-122B 从 P=0.835 突破到 0.914
   - 实测: 9B 从 P=0.887 → 0.907 (触发 Oracle，但后续被认定为 false positive)

### 审计发现
5. **14B Exp1 是 false positive**: 数值验证证明闭合公式 `-N²/4·(1+e^{-2/N})/(1-e^{-2/N})²` 差 16,000 倍
6. **9B post-fix OMEGA 是 false positive**: Step 4 空洞 "[COMPLETE] yield -1/12" 没有推导
7. **Oracle 系统性问题**: 3 次 false positive 都被 DeepSeek Reasoner 接受 — LLM Oracle 用训练知识脑补

### 数据保存
- `autoresearch/phase2_results/`: 20+ WAL/stderr/proxy 文件
- `POST_FIX_RESEARCH_SUMMARY.md`: 完整 PPUT 表 + 五阶段分析
- `nsweep_ds_analysis.md`: DeepSeek API N sweep 分析
- `qwen35_122b_sweep_analysis.md`: 122B 完整数据

## Key Decisions

### 1. 苦涩的教训 × 模型 vs 系统调优
**证据**: V3.2 上 5 个系统调优实验全部失败（0 valid OMEGA），换 Q3.5-122B 一次成功 49× 提升。
**结论**: Compute >> Engineering。停止 V3.2 系统调优。

### 2. Rule 20 (禁止 Over-Alignment) 实证
所有人工规则（quality instructions / SUBSTANCE / Bear / Oracle 结构化审计）都让 OMEGA 成功率从 2/3 降到 0/5。全部已回滚。

### 3. N=3 是所有模型的最优 swarm size
V3.2 和 Q3.5-122B 都在 N=3 取得最高 PPUT。N≥5 时 Brooks's Law 主导（注意力稀释 + 每 agent 步数变少）。

### 4. 投资列表修复是结构改进不是人工规则
当前 invest prompt 展示"最近 10 节点"有 recency 偏差。改为"5 recent + 5 top-priced"让市场信号可以持续表达，不因时间衰减。符合大宪章 — 不改变评价标准，只改变信息可见性。

### 5. LLM Oracle 是系统最弱环节
3 次 false positive 证明 DeepSeek Reasoner 会脑补不完整链。需要分离"文本提取"（LLM）和"数学验证"（机械）。

## Next Steps

### [OPEN SPRINT] 实现 Oracle 数值验证 Layer 1
目标: 抓到所有历史 false positive，保留所有真实 OMEGA
1. 新增 `ORACLE_NUMERIC_TARGET` 环境变量
2. 创建 `verify_proof.py`: LLM 提取最终公式 → Python 数值验证
3. evaluator.rs Oracle 分支: 先数值验证，通过后调 DeepSeek Reasoner 做最终确认
4. 回归测试: V3.2/Q3.5-122B/35B-A3B/27B OMEGA 应仍通过；14B/9B false positive 应被拒

### 高优先级
1. **Commit + push** 当前 evaluator.rs 修复 (投资列表)
2. **多次复跑 27B 和 35B-A3B** 验证 PPUT 稳定性
3. **修复 Oracle** (Layer 1 数值验证)
4. **用修复后的 Oracle 重测 9B/14B** 找真正的模型地板

### 中优先级
5. 跨问题测试（Basel 问题、简单积分等），验证 N=3 是否问题无关
6. MoE vs Dense 效率研究（35B-A3B 3B active 已接近 27B dense）

### NOT 推荐
- ❌ V3.2 系统调优 — 死路
- ❌ 加更多人工规则 — 违反 Rule 20
- ❌ 测 4B 以下 — LLM 延迟限制
- ❌ N≤3 加 Bear — 做空权重太大

## Warnings
- **evaluator.rs 未提交** - 投资列表修复需要 commit
- **9B OMEGA 是 false positive** - 不要把它当成真 OMEGA 用于 PPUT 比较
- **Oracle 不可靠** - 所有 9B/4B 附近的数据需要 Oracle 修复后重新验证
- **修复 Oracle 之前不要测更小模型** - 会产生更多 false positive
- **不要杀 proxy** - 实验之间只 reset stats
- **Mac 残留进程** - 本 session 开始时发现 Mac 上 40 小时残留消耗 ~3 亿 tokens (已清理)

## Architect Insights (本次会话)
- **苦涩的教训 × 原子步骤**: 人工规则（质量指令、SUBSTANCE、结构化审计）全部让 OMEGA 成功率从 100% 降到 0%。已通过 8 次对照实验实证。
- **价格信号两维度**: 高价格 = Ground Truth 合规 + 目标推进。市场自然编码这两个维度。
- **Compute >> Engineering**: V3.2 5次系统调优 vs 换 Q3.5-122B 一次 → 49× PPUT 提升。
- **Oracle LLM 不可替代形式验证**: 3 次 false positive 证明 LLM 无法可靠验证复杂数学。Lean 4 才是终极方案。
- （本次会话未创建新 architect-insights 归档文件 — 建议下次会话归档以上四条）
