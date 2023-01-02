#[derive(Clone)]
pub struct StepProgress {
    pub step_name: String,
    pub current_step: u16,
    pub step_count: u16
}

impl StepProgress {
    pub fn is_done(&self) -> bool {
        self.current_step >= self.step_count
    }
}