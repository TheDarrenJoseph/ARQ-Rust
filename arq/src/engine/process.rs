use crate::progress::StepProgress;

pub(crate) mod map_generation;

pub trait Progressible {
    fn get_progress(&self) -> StepProgress;
}