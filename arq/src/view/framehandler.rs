use tui::layout::Rect;

pub mod character;
pub mod console;
pub mod container;
pub mod container_choice;
pub mod util;
pub mod map_generation;

/*
    FrameHandlers are "dumb" views that simply draw themselves/given input to a terminal frame (the screen)
 */
pub trait FrameHandler<B: tui::backend::Backend, T> {
    /*
        When a FrameHandler "handles" a frame it essentially just draws it's input / content to the frame it is provided a frame by a View/Higher level UI component
     */
    fn handle_frame(&mut self, frame: &mut tui::terminal::Frame<B>, data: FrameData<T>);
}

pub struct FrameData<T> {
    pub data : T,
    pub frame_size : Rect // This is for size reference or to restrict the rendering area available
}

impl <T> FrameData<T> {
    pub fn unpack(&mut self) -> &mut T {
        &mut self.data
    }
    pub fn get_frame_size(&mut self) -> &Rect {
        &self.frame_size
    }
}
