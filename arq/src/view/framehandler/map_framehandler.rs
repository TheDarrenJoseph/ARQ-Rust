use crate::engine::level::Level;
use crate::view::framehandler::{FrameData, FrameHandler};

pub struct MapFrameHandler {
}

impl <B : tui::backend::Backend> FrameHandler<B, Level> for MapFrameHandler {
    fn handle_frame(&mut self, frame: &mut tui::terminal::Frame<B>, data: FrameData<Level>) {
    }
}