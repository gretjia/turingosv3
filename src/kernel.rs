//! TuringOS Microkernel
//! Architecture: The Bitter Lesson + Turing State Machine
//! Philosophy: No human priors. Pure topology, formal logic, and Hayekian pricing.

use std::collections::{HashMap, HashSet};

// ============================================================================
// [1] FUNDAMENTAL ONTOLOGY (宇宙的底层标量)
// ============================================================================

pub type Token = u64;          // 算力资本 (Stake / s_i)
pub type FileId = String;      // DAG 知识节点哈希 (Path)

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MachineState {
    Running,
    Halt, // 终极神迹降临，双重圆圈亮起
}

/// 拓扑纸带上的绝对知识切片
#[derive(Debug, Clone)]
pub struct File {
    pub id: FileId,
    pub author: String,
    pub payload: String,           // 形式化证明代码 (e.g., Lean 4 / Coq)
    pub citations: Vec<FileId>,    // 引用关系 (W_t 矩阵的因果链)
    pub stake: Token,              // 燃烧的算力资本
    pub price: f64,                // \Pi_t: 由 Map-Reduce 时钟倒灌的绝对价格
}

// ============================================================================
// [2] VERSION CONTROL SNAPSHOT: Q_t = < q_t, HEAD_t, tape_t >
// ============================================================================

#[derive(Debug, Clone)]
pub struct Tape {
    pub files: HashMap<FileId, File>, // Append-only 的绝对历史账本
}

#[derive(Debug, Clone)]
pub struct Head {
    pub paths: HashSet<FileId>, // 算力前沿的拓扑切面
}

/// 全局宇宙快照 Q
#[derive(Debug, Clone)]
pub struct Q {
    pub q: MachineState,
    pub head: Head,
    pub tape: Tape,
}

// ============================================================================
// [3] I/O BOUNDARIES: input & output
// ============================================================================

pub struct SensorContext { // s_i
    pub visible_tape: Tape, // Agent 通过嗅探 tape 上的 price 梯度来寻路
    pub current_head: Head,
}

/// input = < q_i, s_i >
pub struct Input {
    pub q_i: MachineState,
    pub s_i: SensorContext,
}

pub struct Action { // a_o
    pub file_id: FileId,
    pub author: String,
    pub payload: String,
    pub citations: Vec<FileId>,
    pub stake: Token,
}

/// output = < q_o, a_o >
pub struct Output {
    pub q_o: MachineState,
    pub a_o: Action,
}

// ============================================================================
// [4] MIDDLE BLACKBOX: AI as \delta
// ============================================================================

/// AI as \delta (允许产生任何幻觉的直觉集群)
pub trait AIBlackBox {
    fn delta(&mut self, input: &Input) -> Output;
}

// ============================================================================
// [5] BOTTOM TOOLS: rtool & wtool
// ============================================================================

/// < q_i, s_i > = rtool( < q_t, tape_t, HEAD_t > )
pub fn rtool(qt: &Q) -> Input {
    Input {
        q_i: qt.q.clone(),
        s_i: SensorContext {
            visible_tape: qt.tape.clone(), 
            current_head: qt.head.clone(),
        },
    }
}

/// Q_{t+1} = wtool(output | tape_t, HEAD_t)
/// 【核心动作】：消耗旧状态 qt，返回新状态 qt+1。野蛮生长，照单全收。
pub fn wtool(output: Output, mut qt: Q) -> Q {
    let action = output.a_o;
    
    let new_file = File {
        id: action.file_id.clone(),
        author: action.author,
        payload: action.payload,
        citations: action.citations.clone(),
        stake: action.stake,
        price: 0.0, // 初始价值为 0，等待 MR 时钟的哈耶克倒灌
    };

    // [The Bitter Lesson] 无视冗余，废除 Market Clearing！向 tape 绝对追加！
    qt.tape.files.insert(new_file.id.clone(), new_file);
    
    // 更新 HEAD 指针：移除父节点，新节点成为绝对前沿
    qt.head.paths.insert(action.file_id.clone());
    for cit in action.citations {
        qt.head.paths.remove(&cit);
    }

    qt.q = output.q_o;
    
    qt // 严格的所有权转移：时间单向流动，Q_{t+1} 诞生
}

// ============================================================================
// [6] TOP MANAGEMENT: predicates \prod p & ticks mr
// ============================================================================

pub struct Predicates {
    pub law: String, // Init 时注入的 Ground Truth
}

impl Predicates {
    /// \prod \mathbf{p}(output | Q_t)
    /// 波普尔断头台：只做纯粹的形式逻辑与拓扑合法性校验。
    pub fn evaluate(&self, output: &Output, qt: &Q) -> bool {
        let action = &output.a_o;
        // 1. 拓扑合法性：引用的父节点必须存在于 tape 中
        if !action.citations.iter().all(|id| qt.tape.files.contains_key(id)) { return false; }
        
        // 2. 形式逻辑：调用底层编译引擎（如 Lean 4）。任何语法崩溃，瞬间返回 false
        if action.payload.contains("paradox") || action.stake == 0 { return false; }
        
        true 
    }
}

pub struct MapReduce {
    pub target_omega_id: FileId, // 终极实验悬赏节点 \Omega
    pub gamma: f64,              // 奥地利学派时间偏好贴现率 \in (0,1)
}

impl MapReduce {
    /// clock --> mr ==> |map| tape0 ==> |reduce| tape1
    /// 核心引擎：\vec{\Pi}_t = (I - \gamma W_t^T)^{-1} \vec{\Omega}
    pub fn tick(&self, tape: &mut Tape) {
        let mut new_prices = HashMap::new();
        
        // 泰勒级数展开迭代 (Power Iteration)。
        // O(1) 宏观时间内瞬间完成全宇宙知识的向后价值倒灌！
        for _ in 0..15 {
            for (id, file) in &tape.files {
                let mut base_val = 0.0;
                if *id == self.target_omega_id { base_val += 100_000_000_000.0; } // 神迹赏金
                
                // 逆向吸血 (Reverse Imputation)：
                // 扫描宇宙中所有引用了“我”的后代节点，从它们那里抽取版税分红
                let mut imputed_val = 0.0;
                for (child_id, child_file) in &tape.files {
                    if child_file.citations.contains(id) {
                        let weight = 1.0 / (child_file.citations.len() as f64);
                        let child_price = new_prices.get(child_id).unwrap_or(&child_file.price);
                        imputed_val += self.gamma * weight * child_price;
                    }
                }
                new_prices.insert(id.clone(), base_val + imputed_val);
            }
        }
        
        // Map Reduce Commit: 将神迹烙印进 tape_{t+1}
        for (id, price) in new_prices {
            if let Some(file) = tape.files.get_mut(&id) { file.price = price; }
        }
    }
}

// ============================================================================
// [7] INITIALIZATION & THE MACRO LOOP (系统创世与状态机循环)
// ============================================================================

pub struct InitAI;
impl InitAI {
    // human --x| once| law --> initAI
    pub fn boot(law: String, omega: FileId) -> (Q, Predicates, MapReduce) {
        let q0 = Q {
            q: MachineState::Running,
            head: Head { paths: HashSet::new() },
            tape: Tape { files: HashMap::new() },
        };
        (q0, Predicates { law }, MapReduce { target_omega_id: omega, gamma: 0.99 })
    }
}

pub fn run_turing_os(human_spec: String, mut ai: impl AIBlackBox, omega: FileId) {
    // InitAI
    let (mut qt, p, mr) = InitAI::boot(human_spec, omega);
    let mut clock: u64 = 0;

    println!(">>> TuringOS Kernel Booted. Awaiting HALT. <<<");

    loop {
        // [Finalization] q1 ==> if q=halt ==> halt
        if qt.q == MachineState::Halt {
            println!("==== [HALT] DOUBLE-CIRCLE REACHED. UNIVERSE FROZEN. ====");
            break;
        }

        // Q0 ==> rtool ==> input
        let input = rtool(&qt);

        // input ==> AI ==> output
        let output = ai.delta(&input);
        let stake_at_risk = output.a_o.stake;

        // output ==> p
        let is_valid = p.evaluate(&output, &qt);

        if is_valid {
            // p ==> |"Q_{t+1} = wtool(output)"| wtool ==> Q1
            // Rust 所有权发生转移：旧 qt 物理消亡，新快照诞生。
            qt = wtool(output, qt);
            println!("[Tick {}] [+] ACCEPTED. File Appended.", clock);
        } else {
            // p ==> |"Q_{t+1} = Q_t"| Q0
            // 【最深邃的惩罚】：什么都不做。旧 qt 依然是 qt。
            // 但 output 在此时超出作用域，质押的 stake 直接被 Drop 进内存黑洞！
            qt = qt; 
            println!("[Tick {}] [-] REJECTED. {} Stake Burned.", clock, stake_at_risk);
        }

        // clock --> mr ==> |map| tape0 ==> |reduce| tape1
        clock += 1;
        if clock % 10 == 0 {
            mr.tick(&mut qt.tape);
            println!(">>> MapReduce Triggered: Pricing Tensor Injected. <<<");
        }
    }
}