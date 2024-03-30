
use log::error;


#[derive(Clone)]
pub struct MultiStepProgress {
    current_step_index: Option<usize>,
    steps: Vec<Step>
}

#[derive(Clone)]
pub struct Step {
    pub id: String,
    pub description: String
}

impl MultiStepProgress {
    pub fn for_steps_not_started(steps: Vec<Step>) -> MultiStepProgress {
        MultiStepProgress { current_step_index: None, steps }
    }

    pub fn get_current_step_value(&self) -> Option<&Step> {
        return if let Some(idx) = self.current_step_index {
            self.steps.get(idx)
        } else {
            None
        }
    }

    /*
    * Returns the current step, 1-indexed for human readability
    */
    pub fn get_current_step_number(&self) -> Option<usize> {
        self.current_step_index.map(|step_idx| step_idx+1)
    }

    pub fn next_step(&mut self) {
        if let Some(index) = self.current_step_index {
            let next_index = index+1;
            let step_count = self.steps.len();
            if step_count > next_index {
                self.current_step_index = Some(next_index);
            } else {
                error!("Cannot set next step, next stop would be: {} which is beyond the size of the steps: {}", index, step_count);
            }
        } else if self.steps.len() > 0 {
            // Start at the first step
            self.current_step_index = Some(0);
        }
    }

    pub fn step_count(&self) -> usize {
        self.steps.len()
    }

    pub fn steps(&self) -> &Vec<Step> {
        &self.steps
    }

    pub fn is_done(&self) -> bool {
        if let Some(idx) = self.current_step_index {
            idx == self.step_count() - 1
        } else {
            false
        }
    }

    pub fn get_progress_percentage(&self) -> usize {
        if let Some(idx) = self.current_step_index {
            (100 / self.steps.len()) * (idx+1)
        } else {
            0
        }
    }
}