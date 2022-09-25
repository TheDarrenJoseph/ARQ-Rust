use tui::layout::Rect;

pub mod character;
pub mod console;
pub mod container;
pub mod container_choice;
pub mod util;

// FrameHandlers are "dumb" views that simply draw themselves to a terminal frame
pub trait FrameHandler<B: tui::backend::Backend, T> {
    fn handle_frame(&mut self, frame: &mut tui::terminal::Frame<B>, data: FrameData<T>);
}

pub struct FrameData<T> {
    pub data : T,
    pub frame_size : Rect
}

impl <T> FrameData<T> {
    pub fn unpack(&mut self) -> &mut T {
        &mut self.data
    }
    pub fn get_frame_size(&mut self) -> &Rect {
        &self.frame_size
    }
}