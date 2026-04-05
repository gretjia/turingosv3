# TuringOS v3 — Handover State
**Updated**: 2026-04-05
**Session Summary**: HTTP 代理架构部署 + 3 incident 记录 + Living Harness 升级 (14 rules, 9 incidents)

## Current State
- **云端 API 已通**: Mac → llm_proxy.py (localhost:8088) → DashScope qwen3-8b，3 agents × 59 appends/2min
- **llm_proxy.py**: Python OpenAI SDK 本地 HTTP 代理，ThreadingMixIn 支持 15+ 并发
- **llm_http.rs**: 纯 reqwest HTTP，不再有 TLS/subprocess/curl 逻辑
- **protocol.rs**: 三层容错 (JSON 前缀 + LaTeX 转义 + 裸标签回退)
- **Living Harness**: 14 rules (新增 R-013, R-014) + 9 incidents (新增 V-007, V-008, V-009)
- **constitutional_check.sh**: 新增 V-008/V-009 回归检查，14 PASS / 0 FAIL
- **四节点**: Mac (proxy + evaluator 已验证) + linux1/Win1 (待部署) + omega-vm (Git)

## Changes This Session

### HTTP Proxy Architecture (核心工程)
1. `src/drivers/llm_proxy.py` (NEW) — 本地 HTTP 代理 + ThreadingMixIn (`4916aee`, `1e76974`)
2. `src/drivers/llm_http.rs` — 完全重写为纯 reqwest HTTP (`4916aee`)
3. `evaluator.rs` — `proxy` provider + `DEEPSEEK_URL` env override (`4916aee`)

### Parser 三层修复 (V-009)
4. `src/sdk/protocol.rs` — JSON 前缀容错 find('{') (`db12049`)
5. `src/sdk/protocol.rs` — LaTeX 反斜杠 fix_json_escapes() (`1f18458`)
6. `src/sdk/protocol.rs` — 裸标签回退 preceding text (`0ffd94c`)

### Living Harness 升级
7. 3 incident records: V-007, V-008, V-009 (完整 meta/trace/root_cause/resolution)
8. 2 new rules: R-013 (format_contract), R-014 (proxy_concurrency)
9. VIA_NEGATIVA: +3 entries (#6, #7, #8)
10. constitutional_check.sh: +2 检查 sections (V-008 proxy concurrency, V-009 parser resilience)
11. LIVING_HARNESS.md: 更新数字 (12→14 rules, 6→9 incidents)
12. incidents/INDEX.yaml: 更新

### Cleanup
13. `wallet.rs` — 删除死代码 parse_transfer
14. `llm_http.rs` — 删除 unused info import

## Key Decisions
- **Python HTTP 代理 > reqwest 直连 HTTPS**: macOS pipe deadlock + Chinese TLS，代理是工业标准 (V-007)
- **ThreadingMixIn 强制**: 单线程 HTTP + N agents = (N-1) 502 (V-008)
- **Postel 法则**: LLM 输出是概率信号，解析器必须宽进严出 + 永不静默失败 (V-009)
- **fix_json_escapes 而非强制 JSON**: LaTeX + JSON 本质不兼容，修复解析器比约束 LLM 更可靠

## Architect Insights (本次会话)
本次会话无新架构洞察（纯工程执行 session）。

## Next Steps
1. **[IMMEDIATE] 启动模型规模筛查 AutoResearch** — Mac proxy 已就绪
2. **[OPEN SPRINT] bus.rs tick_map_reduce 重构**
3. **[OPEN SPRINT] math_membrane.rs 语义断头台对齐**
4. **[OPEN SPRINT] log_lib agent 工具**

## Warnings
- **Mac proxy 需手动启动**: `export $(grep -v '^#' .env | xargs) && nohup python3 src/drivers/llm_proxy.py --port 8088 > /tmp/llm_proxy.log 2>&1 &`
- **evaluator env**: `LLM_PROVIDER=proxy LLM_URL=http://127.0.0.1:8088/v1/chat/completions LLM_MODEL=qwen3-8b DEEPSEEK_URL=http://127.0.0.1:8088/v1/chat/completions`
- **SWARM_SIZE** (非 N_AGENTS) 控制 agent 数量
- **linux1 只能通过 Mac 跳板**: `ssh zephrymac-studio "ssh zepher@192.168.3.113 '...'"`
- **LIBRARIAN_INTERVAL=8 暂定** — 见 memory `project_librarian_interval.md`
