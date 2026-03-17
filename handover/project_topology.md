# TuringOS v3 Project Topology: Star-Topology Microkernel & Event Bus

## Architectural Philosophy
Transition from "Layered/Onion Architecture" to a "Pure Microkernel" with "Bus-based Event-Driven" plugins (SKILLs).

**Core Principle:** Separation of Mechanism and Policy.
*   **Mechanism (Kernel):** Represents absolute truth. The kernel only records physical states (Tape DAG) and performs mathematical operations (MapReduce). It defaults to wanting to execute REDUCE every step.
*   **Policy (SKILLs):** Represents physical compromises (e.g., CPU limits). SKILLs are independent, equal-level plugins that interact via an event bus. They can intercept, modify, or veto actions, and decide when to skip resource-intensive operations like REDUCE.

## Topology Diagram

```text
                           [ Bootloader / 创世点火器 ]
                                      │ (按需组装注入 SKILL 矩阵)
                                      ▼
========================================================================================
   [ THE EVENT BUS (宇宙总线) ]  (钩子: on_boot, pre_append, post_append, should_skip_reduce)
========================================================================================
                 │                          │                          │
      [ SKILL: Membrane Guard ]    [ SKILL: WAL Snapshot ]    [ SKILL: Thermodynamic ]
       (正则斩杀幻觉、提纯基态)         (O(1) 无锁异步落盘)          (√V 自适应算力节流) 
                 │                          │                          │
                 +--------------------------+--------------------------+
                                            │ (极其干净的物理指令)
========================================================================================
                              [ 绝对封印的神圣微内核 Kernel ] 
                              - 机制 1：Tape (只记录绝对物理态，构建 DAG)
                              - 机制 2：MapReduce (纯粹哈耶克方程，无条件倒灌)
========================================================================================
```

## Key Components

1.  **Turing Skill Protocol (TSP):** A standard trait (`TuringSkill`) for all plugins, defining lifecycle hooks (`on_boot`, `on_pre_append`, `on_post_append`, `should_skip_reduce`).
2.  **Sacred Microkernel (`Kernel`):** Stripped of all engineering judgments and timing concepts. Contains only the `Tape` and pure mechanisms (`append_tape`, `hayekian_map_reduce`).
3.  **Turing Bus (`TuringBus`):** The orchestrator that holds the Kernel and a list of SKILLs. It drives the universe's "Tick", applying SKILL hooks before, during, and after Kernel operations.
4.  **Plugins (SKILLs):** Examples include `ThermodynamicHeartbeatSkill` (implements the $\sqrt{V}$ adaptive throttling), `MembraneGuardSkill`, `WalSnapshotSkill`, etc.

This topology ensures the kernel remains mathematically pure and frozen, while allowing infinite, non-hierarchical extensibility through the event bus.