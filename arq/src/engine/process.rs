use crate::progress::MultiStepProgress;

pub(crate) mod map_generation;

pub trait Progressible {
    fn get_progress(&self) -> MultiStepProgress;
}