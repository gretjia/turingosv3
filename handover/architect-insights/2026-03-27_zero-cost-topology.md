---
date: 2026-03-27
status: proposed
related_commits: []
---

## 原话
> "零成本拓扑（The Builder）：Agent 发现推理步骤，调用 append_node 提交。成本绝对为 0 Coins。节点瞬间上链。大模型拥有毫无金融风险的言论自由。没有任何 IP 版税，拒绝一切垄断。"

## 浓缩
建树零成本，投资独立，拓扑金融彻底剥离

## 架构含义
- 当前: append_node 必须经过 WalletTool 扣款 (stake >= 1.0)
- 新: append_node 完全免费，不经过 WalletTool
- invest 是独立动作，只在 Agent 主动下注时触发
- 彻底消除"写节点就要花钱"的门槛 → 鼓励充分试错
- intrinsic_reward 字段废除 (或恒定为 0)

## 行动项
- [ ] bus.rs: append 不调 WalletTool，直接 kernel.append_tape(file, 0.0)
- [ ] 新增独立 invest 动作路径 (不在 append 流程中)
- [ ] WalletTool on_pre_append: 仅对 invest 类 payload 生效
