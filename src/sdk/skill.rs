pub enum SkillSignal {
    Pass,
    Modify(String),
    Veto(String),
}

pub trait TuringSkill: Send + Sync {
    fn manifest(&self) -> &'static str;
    fn on_boot(&mut self) {}
    fn on_pre_append(&mut self, _payload: &str) -> SkillSignal { SkillSignal::Pass }
    fn on_post_append(&mut self, _node: &crate::kernel::File) {}
    fn should_skip_reduce(&mut self, _current_volume: usize) -> bool { false }
}
