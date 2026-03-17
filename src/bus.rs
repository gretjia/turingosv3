use crate::kernel::{Kernel, File};
use crate::sdk::skill::{TuringSkill, SkillSignal};

pub struct TuringBus {
    pub kernel: Kernel,
    pub skills: Vec<Box<dyn TuringSkill>>,
    pub clock: usize,
}

impl TuringBus {
    pub fn new(kernel: Kernel) -> Self {
        Self {
            kernel,
            skills: Vec::new(),
            clock: 0,
        }
    }

    pub fn mount_skill(&mut self, mut skill: Box<dyn TuringSkill>) {
        skill.on_boot();
        self.skills.push(skill);
    }

    pub fn append(&mut self, mut file: File) -> Result<(), String> {
        let mut final_reward = 0.0;
        
        // 1. Pre-append hooks
        for skill in &mut self.skills {
            match skill.on_pre_append(&file.payload) {
                SkillSignal::Pass => {}
                SkillSignal::Modify(new_payload) => {
                    file.payload = new_payload;
                }
                SkillSignal::Veto(reason) => {
                    return Err(reason);
                }
                SkillSignal::YieldReward { payload, reward } => {
                    file.payload = payload;
                    final_reward += reward;
                }
            }
        }

        // 2. Kernel append
        let node = self.kernel.append_tape(file.clone(), final_reward);

        // 3. Post-append hooks
        for skill in &mut self.skills {
            skill.on_post_append(node);
        }

        self.clock += 1;
        Ok(())
    }

    pub fn tick_map_reduce(&mut self) {
        let current_volume = self.kernel.tape.files.len();
        
        // Find current max price in the market
        let current_max_price = self.kernel.tape.files.values()
            .map(|f| f.price)
            .fold(0.0, f64::max);

        let mut skip = false;
        for skill in &mut self.skills {
            if skill.should_skip_reduce(current_volume) {
                skip = true;
            }
            if skill.should_skip_reduce_by_price(current_max_price) {
                skip = true;
            }
        }

        if !skip {
            println!(">>> [Event Bus] Triggering REDUCE (Volume: {}, MaxPrice: {:.2}) <<<", current_volume, current_max_price);
            self.kernel.hayekian_map_reduce();
        }
    }
}

pub struct ThermodynamicHeartbeatSkill {
    pub threshold: usize,
    pub last_mr_volume: usize,
}

impl ThermodynamicHeartbeatSkill {
    pub fn new(threshold: usize) -> Self {
        Self {
            threshold,
            last_mr_volume: 0,
        }
    }
}

impl TuringSkill for ThermodynamicHeartbeatSkill {
    fn manifest(&self) -> &'static str {
        "Thermodynamic Heartbeat Skill"
    }

    fn should_skip_reduce(&mut self, current_volume: usize) -> bool {
        if current_volume - self.last_mr_volume >= self.threshold {
            self.last_mr_volume = current_volume;
            false // Do not skip
        } else {
            true // Skip
        }
    }
}

pub struct MembraneGuardSkill;

impl TuringSkill for MembraneGuardSkill {
    fn manifest(&self) -> &'static str {
        "Membrane Guard Skill"
    }
    
    fn on_pre_append(&mut self, payload: &str) -> SkillSignal {
        if payload.contains("paradox") {
            SkillSignal::Veto("Membrane rejected payload".into())
        } else {
            SkillSignal::Pass
        }
    }
}

pub struct WalSnapshotSkill;

impl TuringSkill for WalSnapshotSkill {
    fn manifest(&self) -> &'static str {
        "WAL Snapshot Skill"
    }
}
